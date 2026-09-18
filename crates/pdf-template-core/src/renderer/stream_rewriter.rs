use lopdf::{Document, Object, ObjectId, Stream, Dictionary, StringFormat};
use lopdf::content::{Content, Operation};
use crate::error::{PdfTemplateError, Result};
use crate::layout::ResolvedLayout;
use crate::model::Placeholder;
use crate::parser::encoding::encode_pdf_string;

pub struct ReplacementTask {
    pub placeholder: Placeholder,
    pub layout: ResolvedLayout,
}

pub struct StreamRewriter<'a> {
    doc: &'a mut Document,
    page_id: ObjectId,
    page_index: usize,
}

impl<'a> StreamRewriter<'a> {
    pub fn new(doc: &'a mut Document, page_id: ObjectId, page_index: usize) -> Self {
        Self {
            doc,
            page_id,
            page_index,
        }
    }

    pub fn apply_replacements(&mut self, tasks: &[ReplacementTask]) -> Result<()> {
        if tasks.is_empty() {
            return Ok(());
        }

        // 1. Get current page content bytes and decode into Content
        let content_data = self.doc.get_page_content(self.page_id);

        let mut content = Content::decode(&content_data).map_err(|e| {
            PdfTemplateError::RenderError {
                message: format!("Failed to decode content for page {}: {}", self.page_index, e),
            }
        })?;

        // 2. Group all span slices to remove from original operations
        // Sort in descending order of (op_index, sub_index, byte_start) so earlier offsets are preserved
        let mut removals: Vec<(usize, Option<usize>, usize, usize)> = Vec::new();
        for task in tasks {
            for sref in &task.placeholder.span_refs {
                removals.push((sref.op_index, sref.sub_index, sref.byte_start, sref.byte_end));
            }
        }

        removals.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| b.2.cmp(&a.2))
        });

        // 3. Perform surgical removal on content.operations
        for (op_idx, sub_idx, byte_start, byte_end) in removals {
            if op_idx >= content.operations.len() {
                continue;
            }
            let op = &mut content.operations[op_idx];

            match op.operator.as_str() {
                "Tj" | "'" => {
                    if let Some(Object::String(bytes, _)) = op.operands.get_mut(0) {
                        slice_bytes_in_place(bytes, byte_start, byte_end);
                    }
                }
                "\"" => {
                    if let Some(Object::String(bytes, _)) = op.operands.get_mut(2) {
                        slice_bytes_in_place(bytes, byte_start, byte_end);
                    }
                }
                "TJ" => {
                    if let Some(Object::Array(arr)) = op.operands.get_mut(0) {
                        if let Some(sub) = sub_idx {
                            if sub < arr.len() {
                                if let Object::String(bytes, _) = &mut arr[sub] {
                                    slice_bytes_in_place(bytes, byte_start, byte_end);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 3b. Shift coordinates for reflow and alignment
        for task in tasks {
            let ph = &task.placeholder;
            let layout = &task.layout;

            if ph.scale_x < 1e-6 {
                continue;
            }

            if ph.is_right_aligned {
                // For right-aligned placeholders:
                // If the BT block containing the placeholder started before it (e.g. contained a currency symbol like '$'),
                // shift that BT block's Tm so the preceding symbol shifts with the right-aligned text.
                let delta_shift = (ph.width - layout.rendered_width) / ph.scale_x;
                if delta_shift.abs() > 0.01 {
                    if let Some(first_ref) = ph.span_refs.first() {
                        let first_op = first_ref.op_index;
                        for i in (0..first_op).rev() {
                            let op = &mut content.operations[i];
                            if op.operator == "BT" {
                                break;
                            }
                            if op.operator == "Tm" {
                                if let Some(operand) = op.operands.get_mut(4) {
                                    match operand {
                                        Object::Real(e) => *e += delta_shift as f32,
                                        Object::Integer(e) => *operand = Object::Real(*e as f32 + delta_shift as f32),
                                        _ => {}
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            } else {
                // For left-aligned or inline placeholders:
                let delta_w = layout.rendered_width - ph.width;
                if delta_w.abs() > 0.1 {
                    let delta_w_text = delta_w / ph.scale_x;

                    if let Some(last_ref) = ph.span_refs.last() {
                        let last_op = last_ref.op_index;

                        // A) If followed by text in the same BT block (e.g. comma in "{{city}},"),
                        // shift the succeeding Td operator
                        for i in (last_op + 1)..content.operations.len() {
                            let op = &mut content.operations[i];
                            if op.operator == "ET" || op.operator == "BT" || op.operator == "Tm" {
                                break;
                            }
                            if op.operator == "Td" || op.operator == "TD" {
                                if let Some(operand) = op.operands.get_mut(0) {
                                    match operand {
                                        Object::Real(dx) => *dx += delta_w_text as f32,
                                        Object::Integer(dx) => *operand = Object::Real(*dx as f32 + delta_w_text as f32),
                                        _ => {}
                                    }
                                }
                                break;
                            }
                        }

                        // B) If followed by text in subsequent BT blocks on the SAME visual line
                        // (e.g. "{{invoice.number}} within 15 calendar days..."):
                        let mut ph_line_f: Option<f64> = None;
                        for i in (0..=last_op).rev() {
                            let op = &content.operations[i];
                            if op.operator == "BT" {
                                break;
                            }
                            if op.operator == "Tm" {
                                if let Some(f_op) = op.operands.get(5) {
                                    ph_line_f = crate::parser::font::obj_to_f64(f_op);
                                }
                                break;
                            }
                        }

                        if let Some(target_f) = ph_line_f {
                            for i in (last_op + 1)..content.operations.len() {
                                let op = &mut content.operations[i];
                                if op.operator == "cm" {
                                    break;
                                }
                                if op.operator == "Tm" {
                                    let cur_f = op.operands.get(5).and_then(crate::parser::font::obj_to_f64);
                                    let cur_e = op.operands.get(4).and_then(crate::parser::font::obj_to_f64);
                                    if let (Some(f), Some(e)) = (cur_f, cur_e) {
                                        // If on the same visual line (within 2 pt) and positioned after placeholder
                                        if (f - target_f).abs() < 2.0 && e >= (ph.x / ph.scale_x) - 1.0 {
                                            if let Some(operand) = op.operands.get_mut(4) {
                                                match operand {
                                                    Object::Real(val) => *val += delta_w_text as f32,
                                                    Object::Integer(val) => *operand = Object::Real(*val as f32 + delta_w_text as f32),
                                                    _ => {}
                                                }
                                            }
                                        } else if (f - target_f).abs() >= 2.0 {
                                            // Different line reached, stop
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Reset graphics state to default user space
        // PDF content streams may have top-level `cm` transformations (e.g. 0.24 ... cm)
        // or unclosed `q` states. We wrap the entire original stream in `q ... Q` so that
        // our appended replacement operations run in pristine default user space (identity CTM).
        let mut depth: i32 = 0;
        for op in &content.operations {
            if op.operator == "q" {
                depth += 1;
            } else if op.operator == "Q" {
                depth -= 1;
            }
        }
        content.operations.insert(0, Operation::new("q", vec![]));
        let q_to_pop = (depth + 1).max(1);
        for _ in 0..q_to_pop {
            content.operations.push(Operation::new("Q", vec![]));
        }

        // 5. Append replacement text operations
        for task in tasks {
            let ph = &task.placeholder;
            let layout = &task.layout;

            let font_to_use = if self.has_font_resource(&ph.font_name) && self.is_font_safe_for_direct_ascii(&ph.font_name) {
                ph.font_name.clone()
            } else {
                let variant = self.get_font_variant(&ph.font_name);
                self.ensure_fallback_font(&variant)?
            };

            let render_x = if ph.is_right_aligned {
                (ph.x + ph.width) - layout.rendered_width
            } else {
                ph.x
            };

            // Begin isolated graphics state
            content.operations.push(Operation::new("q", vec![]));

            // If clipping is active
            if layout.should_clip {
                content.operations.push(Operation::new(
                    "re",
                    vec![
                        Object::Real(render_x as f32),
                        Object::Real((ph.y - ph.height * 0.2) as f32),
                        Object::Real(layout.rendered_width.max(ph.width) as f32),
                        Object::Real((ph.height * 1.2) as f32),
                    ],
                ));
                content.operations.push(Operation::new("W", vec![]));
                content.operations.push(Operation::new("n", vec![]));
            }

            // Set color if original placeholder had color
            if let Some(color) = ph.color {
                content.operations.push(Operation::new(
                    "rg",
                    vec![
                        Object::Real(color[0]),
                        Object::Real(color[1]),
                        Object::Real(color[2]),
                    ],
                ));
            }

            // Begin text
            content.operations.push(Operation::new("BT", vec![]));

            // Set font and effective size
            content.operations.push(Operation::new(
                "Tf",
                vec![
                    Object::Name(font_to_use.as_bytes().to_vec()),
                    Object::Real(layout.effective_font_size as f32),
                ],
            ));

            let leading = layout.effective_font_size * 1.2;

            for (line_idx, line) in layout.lines.iter().enumerate() {
                let line_y = ph.y - (line_idx as f64) * leading;

                // Text matrix: [cos, sin, -sin, cos, x, y]
                let rad = ph.rotation.to_radians();
                let cos = rad.cos();
                let sin = rad.sin();

                content.operations.push(Operation::new(
                    "Tm",
                    vec![
                        Object::Real(cos as f32),
                        Object::Real(sin as f32),
                        Object::Real(-sin as f32),
                        Object::Real(cos as f32),
                        Object::Real(render_x as f32),
                        Object::Real(line_y as f32),
                    ],
                ));

                let encoded_bytes = encode_pdf_string(line);
                let str_obj = Object::String(encoded_bytes, StringFormat::Literal);
                content.operations.push(Operation::new("Tj", vec![str_obj]));
            }

            content.operations.push(Operation::new("ET", vec![]));
            content.operations.push(Operation::new("Q", vec![]));
        }

        // 6. Encode modified content stream and save back to the document
        let encoded_stream = content.encode().map_err(|e| {
            PdfTemplateError::RenderError {
                message: format!("Failed to encode modified content for page {}: {}", self.page_index, e),
            }
        })?;

        self.update_page_contents(encoded_stream)?;

        Ok(())
    }

    fn has_font_resource(&self, font_name: &str) -> bool {
        if let Ok(page_dict) = self.doc.get_dictionary(self.page_id) {
            if let Ok(res) = page_dict.get(b"Resources") {
                let res_dict = match res {
                    Object::Dictionary(d) => Some(d),
                    Object::Reference(id) => match self.doc.get_object(*id) {
                        Ok(Object::Dictionary(d)) => Some(d),
                        Ok(Object::Stream(s)) => Some(&s.dict),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(r) = res_dict {
                    if let Ok(fonts) = r.get(b"Font") {
                        let fonts_dict = match fonts {
                            Object::Dictionary(d) => Some(d),
                            Object::Reference(id) => match self.doc.get_object(*id) {
                                Ok(Object::Dictionary(d)) => Some(d),
                                Ok(Object::Stream(s)) => Some(&s.dict),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(f) = fonts_dict {
                            return f.has(font_name.as_bytes());
                        }
                    }
                }
            }
        }
        false
    }

    fn is_font_safe_for_direct_ascii(&self, font_name: &str) -> bool {
        if let Ok(page_dict) = self.doc.get_dictionary(self.page_id) {
            if let Ok(res) = page_dict.get(b"Resources") {
                let res_dict = match res {
                    Object::Dictionary(d) => Some(d),
                    Object::Reference(id) => match self.doc.get_object(*id) {
                        Ok(Object::Dictionary(d)) => Some(d),
                        Ok(Object::Stream(s)) => Some(&s.dict),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(r) = res_dict {
                    if let Ok(fonts) = r.get(b"Font") {
                        let fonts_dict = match fonts {
                            Object::Dictionary(d) => Some(d),
                            Object::Reference(id) => match self.doc.get_object(*id) {
                                Ok(Object::Dictionary(d)) => Some(d),
                                Ok(Object::Stream(s)) => Some(&s.dict),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(f) = fonts_dict {
                            if let Ok(font_obj) = f.get(font_name.as_bytes()) {
                                let f_dict = match font_obj {
                                    Object::Dictionary(d) => Some(d),
                                    Object::Reference(id) => match self.doc.get_object(*id) {
                                        Ok(Object::Dictionary(d)) => Some(d),
                                        Ok(Object::Stream(s)) => Some(&s.dict),
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if let Some(fd) = f_dict {
                                    // If font has a ToUnicode mapping, it is almost certainly a subset font with custom glyph IDs
                                    if fd.has(b"ToUnicode") {
                                        return false;
                                    }
                                    if let Ok(st) = fd.get(b"Subtype") {
                                        if let Object::Name(name) = st {
                                            if name == b"Type0" || name == b"Type3" {
                                                return false;
                                            }
                                        }
                                    }
                                    if let Ok(enc) = fd.get(b"Encoding") {
                                        if let Object::Name(name) = enc {
                                            return name == b"WinAnsiEncoding" || name == b"StandardEncoding";
                                        }
                                    }
                                    if let Ok(bf) = fd.get(b"BaseFont") {
                                        if let Object::Name(name) = bf {
                                            let s = String::from_utf8_lossy(name);
                                            if s.contains('+') {
                                                return false;
                                            }
                                            return s.contains("Helvetica") || s.contains("Times") || s.contains("Courier") || s.contains("Arial");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn get_font_variant(&self, font_name: &str) -> String {
        let (is_bold, is_italic, is_serif, is_mono) = self.get_font_style(font_name);
        if is_mono {
            match (is_bold, is_italic) {
                (true, true) => "Courier-BoldOblique".to_string(),
                (true, false) => "Courier-Bold".to_string(),
                (false, true) => "Courier-Oblique".to_string(),
                (false, false) => "Courier".to_string(),
            }
        } else if is_serif {
            match (is_bold, is_italic) {
                (true, true) => "Times-BoldItalic".to_string(),
                (true, false) => "Times-Bold".to_string(),
                (false, true) => "Times-Italic".to_string(),
                (false, false) => "Times-Roman".to_string(),
            }
        } else {
            match (is_bold, is_italic) {
                (true, true) => "Helvetica-BoldOblique".to_string(),
                (true, false) => "Helvetica-Bold".to_string(),
                (false, true) => "Helvetica-Oblique".to_string(),
                (false, false) => "Helvetica".to_string(),
            }
        }
    }

    fn get_font_style(&self, font_name: &str) -> (bool, bool, bool, bool) {
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_serif = false;
        let mut is_mono = false;

        let check_name = |name: &str, b: &mut bool, i: &mut bool, s: &mut bool, m: &mut bool| {
            let lower = name.to_lowercase();
            if lower.contains("bold") || lower.contains("black") || lower.contains("heavy")
                || lower.contains("semibold") || lower.contains("demibold") || lower.contains("medium")
                || lower.contains("w6") || lower.contains("w7") || lower.contains("w8") || lower.contains("w9")
                || lower.contains("700") || lower.contains("800") || lower.contains("900") {
                *b = true;
            }
            if lower.contains("italic") || lower.contains("oblique") || lower.contains("slanted") {
                *i = true;
            }
            if lower.contains("times") || lower.contains("serif") || lower.contains("georgia")
                || lower.contains("minion") || lower.contains("garamond") || lower.contains("baskerville") {
                *s = true;
            }
            if lower.contains("courier") || lower.contains("mono") || lower.contains("console") || lower.contains("code") {
                *m = true;
            }
        };

        if let Ok(page_dict) = self.doc.get_dictionary(self.page_id) {
            if let Ok(res) = page_dict.get(b"Resources") {
                let res_dict = match res {
                    Object::Dictionary(d) => Some(d),
                    Object::Reference(id) => match self.doc.get_object(*id) {
                        Ok(Object::Dictionary(d)) => Some(d),
                        Ok(Object::Stream(s)) => Some(&s.dict),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(r) = res_dict {
                    if let Ok(fonts) = r.get(b"Font") {
                        let fonts_dict = match fonts {
                            Object::Dictionary(d) => Some(d),
                            Object::Reference(id) => match self.doc.get_object(*id) {
                                Ok(Object::Dictionary(d)) => Some(d),
                                Ok(Object::Stream(s)) => Some(&s.dict),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(f) = fonts_dict {
                            if let Ok(font_obj) = f.get(font_name.as_bytes()) {
                                let f_dict = match font_obj {
                                    Object::Dictionary(d) => Some(d),
                                    Object::Reference(id) => match self.doc.get_object(*id) {
                                        Ok(Object::Dictionary(d)) => Some(d),
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if let Some(fd) = f_dict {
                                    if let Ok(bf) = fd.get(b"BaseFont") {
                                        if let Ok(name_bytes) = bf.as_name() {
                                            let name = String::from_utf8_lossy(name_bytes);
                                            check_name(&name, &mut is_bold, &mut is_italic, &mut is_serif, &mut is_mono);
                                        }
                                    }
                                    if let Ok(desc_obj) = fd.get(b"FontDescriptor") {
                                        let desc_dict = match desc_obj {
                                            Object::Dictionary(d) => Some(d),
                                            Object::Reference(id) => match self.doc.get_object(*id) {
                                                Ok(Object::Dictionary(d)) => Some(d),
                                                _ => None,
                                            },
                                            _ => None,
                                        };
                                        if let Some(dd) = desc_dict {
                                            if let Ok(fn_obj) = dd.get(b"FontName") {
                                                if let Ok(name_bytes) = fn_obj.as_name() {
                                                    let name = String::from_utf8_lossy(name_bytes);
                                                    check_name(&name, &mut is_bold, &mut is_italic, &mut is_serif, &mut is_mono);
                                                }
                                            }
                                            if let Ok(fw) = dd.get(b"FontWeight") {
                                                if let Some(w) = crate::parser::font::obj_to_f64(fw) {
                                                    if w >= 600.0 { is_bold = true; }
                                                }
                                            }
                                            if let Ok(flags_obj) = dd.get(b"Flags") {
                                                if let Some(flags) = crate::parser::font::obj_to_f64(flags_obj) {
                                                    let fl = flags as u32;
                                                    if (fl & (1 << 6)) != 0 { is_italic = true; }
                                                    if (fl & (1 << 18)) != 0 { is_bold = true; }
                                                    if (fl & (1 << 1)) != 0 { is_serif = true; }
                                                    if (fl & (1 << 0)) != 0 { is_mono = true; }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        (is_bold, is_italic, is_serif, is_mono)
    }

    fn ensure_fallback_font(&mut self, variant: &str) -> Result<String> {
        let font_key = format!("F_PTE_{}", variant.replace('-', "_"));
        if self.has_font_resource(&font_key) {
            return Ok(font_key);
        }

        // Create standard Type 1 font object
        let mut font_dict = Dictionary::new();
        font_dict.set("Type", Object::Name(b"Font".to_vec()));
        font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        font_dict.set("BaseFont", Object::Name(variant.as_bytes().to_vec()));
        font_dict.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));

        let font_id = self.doc.add_object(Object::Dictionary(font_dict));

        // Insert into page's Resources -> Font dictionary
        let page_dict = self.doc.get_dictionary_mut(self.page_id).map_err(|e| {
            PdfTemplateError::RenderError {
                message: format!("Failed to get page dictionary: {}", e),
            }
        })?;

        if !page_dict.has(b"Resources") {
            page_dict.set("Resources", Object::Dictionary(Dictionary::new()));
        }

        // Get or create Resources dictionary
        let resources_id = match page_dict.get(b"Resources") {
            Ok(Object::Reference(id)) => Some(*id),
            _ => None,
        };

        if let Some(res_id) = resources_id {
            if let Ok(res_dict) = self.doc.get_dictionary_mut(res_id) {
                Self::add_font_to_resources(res_dict, &font_key, font_id);
            }
        } else if let Ok(Object::Dictionary(res_dict)) = page_dict.get_mut(b"Resources") {
            Self::add_font_to_resources(res_dict, &font_key, font_id);
        }

        Ok(font_key)
    }

    fn add_font_to_resources(res_dict: &mut Dictionary, font_key: &str, font_id: ObjectId) {
        if !res_dict.has(b"Font") {
            res_dict.set("Font", Object::Dictionary(Dictionary::new()));
        }
        if let Ok(Object::Dictionary(fonts_dict)) = res_dict.get_mut(b"Font") {
            fonts_dict.set(font_key.as_bytes().to_vec(), Object::Reference(font_id));
        }
    }

    fn update_page_contents(&mut self, new_content_bytes: Vec<u8>) -> Result<()> {
        let existing_contents = {
            let page_dict = self.doc.get_dictionary(self.page_id).map_err(|e| {
                PdfTemplateError::RenderError {
                    message: format!("Failed to access page dict: {}", e),
                }
            })?;
            page_dict.get(b"Contents").cloned()
        };

        match existing_contents {
            Ok(Object::Reference(old_id)) => {
                if let Some(Object::Stream(old_stream)) = self.doc.objects.get_mut(&old_id) {
                    old_stream.set_plain_content(new_content_bytes);
                    return Ok(());
                }
            }
            Ok(Object::Array(arr)) => {
                let mut first_id = None;
                for (i, item) in arr.iter().enumerate() {
                    if let Object::Reference(id) = item {
                        if i == 0 {
                            first_id = Some(*id);
                        } else {
                            self.doc.objects.remove(id);
                        }
                    }
                }
                if let Some(first_id) = first_id {
                    if let Some(Object::Stream(first_stream)) = self.doc.objects.get_mut(&first_id) {
                        first_stream.set_plain_content(new_content_bytes);
                        let page_dict = self.doc.get_dictionary_mut(self.page_id).map_err(|e| {
                            PdfTemplateError::RenderError {
                                message: format!("Failed to access page dict: {}", e),
                            }
                        })?;
                        page_dict.set("Contents", Object::Reference(first_id));
                        return Ok(());
                    }
                }
            }
            _ => {}
        }

        let stream_obj = Stream::new(Dictionary::new(), new_content_bytes);
        let new_stream_id = self.doc.add_object(stream_obj);

        let page_dict = self.doc.get_dictionary_mut(self.page_id).map_err(|e| {
            PdfTemplateError::RenderError {
                message: format!("Failed to access page dict: {}", e),
            }
        })?;

        page_dict.set("Contents", Object::Reference(new_stream_id));
        Ok(())
    }
}

fn slice_bytes_in_place(bytes: &mut Vec<u8>, start: usize, end: usize) {
    if start >= bytes.len() {
        return;
    }
    let actual_end = end.min(bytes.len());
    if start < actual_end {
        bytes.drain(start..actual_end);
    }
}
