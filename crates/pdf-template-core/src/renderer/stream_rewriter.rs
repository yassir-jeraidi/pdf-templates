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

        // 4. Ensure standard font resource is registered on the page
        let fallback_font_name = self.ensure_fallback_font()?;

        // 5. Append replacement text operations
        for task in tasks {
            let ph = &task.placeholder;
            let layout = &task.layout;

            let font_to_use = if self.has_font_resource(&ph.font_name) {
                ph.font_name.clone()
            } else {
                fallback_font_name.clone()
            };

            // Begin isolated graphics state
            content.operations.push(Operation::new("q", vec![]));

            // If clipping is active
            if layout.should_clip {
                content.operations.push(Operation::new(
                    "re",
                    vec![
                        Object::Real(ph.x as f32),
                        Object::Real((ph.y - ph.height * 0.2) as f32),
                        Object::Real(ph.width as f32),
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
                        Object::Real(ph.x as f32),
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
                    Object::Reference(id) => self.doc.get_dictionary(*id).ok(),
                    _ => None,
                };
                if let Some(r) = res_dict {
                    if let Ok(fonts) = r.get(b"Font") {
                        let fonts_dict = match fonts {
                            Object::Dictionary(d) => Some(d),
                            Object::Reference(id) => self.doc.get_dictionary(*id).ok(),
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

    fn ensure_fallback_font(&mut self) -> Result<String> {
        let font_key = "F_PTE_Helvetica";
        if self.has_font_resource(font_key) {
            return Ok(font_key.to_string());
        }

        // Create standard Helvetica Type 1 font object
        let mut font_dict = Dictionary::new();
        font_dict.set("Type", Object::Name(b"Font".to_vec()));
        font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        font_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
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
                Self::add_font_to_resources(res_dict, font_key, font_id);
            }
        } else if let Ok(Object::Dictionary(res_dict)) = page_dict.get_mut(b"Resources") {
            Self::add_font_to_resources(res_dict, font_key, font_id);
        }

        Ok(font_key.to_string())
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
