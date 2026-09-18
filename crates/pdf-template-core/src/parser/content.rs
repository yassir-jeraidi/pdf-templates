use std::collections::HashMap;
use lopdf::{Document, Object, ObjectId};
use lopdf::content::Content;
use crate::model::TextSpan;
use crate::parser::encoding::decode_pdf_string;
use crate::parser::font::FontInfo;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Default for Matrix {
    fn default() -> Self {
        Self::identity()
    }
}

impl Matrix {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn from_array(arr: &[f64]) -> Self {
        if arr.len() >= 6 {
            Self {
                a: arr[0],
                b: arr[1],
                c: arr[2],
                d: arr[3],
                e: arr[4],
                f: arr[5],
            }
        } else {
            Self::identity()
        }
    }

    // Multiply: self * other
    pub fn multiply(&self, o: &Matrix) -> Matrix {
        Matrix {
            a: self.a * o.a + self.b * o.c,
            b: self.a * o.b + self.b * o.d,
            c: self.c * o.a + self.d * o.c,
            d: self.c * o.b + self.d * o.d,
            e: self.e * o.a + self.f * o.c + o.e,
            f: self.e * o.b + self.f * o.d + o.f,
        }
    }

    // Translate self by (tx, ty)
    fn translate(&self, tx: f64, ty: f64) -> Matrix {
        Matrix {
            a: self.a,
            b: self.b,
            c: self.c,
            d: self.d,
            e: tx * self.a + ty * self.c + self.e,
            f: tx * self.b + ty * self.d + self.f,
        }
    }

    #[allow(dead_code)]
    fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            x * self.a + y * self.c + self.e,
            x * self.b + y * self.d + self.f,
        )
    }

    fn rotation_degrees(&self) -> f64 {
        self.b.atan2(self.a).to_degrees()
    }
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: Matrix,
    color: Option<[f32; 3]>,
}

pub struct PageParser<'a> {
    #[allow(dead_code)]
    doc: &'a Document,
    page_index: usize,
    #[allow(dead_code)]
    page_id: ObjectId,
    pub fonts: HashMap<String, FontInfo>,
}

impl<'a> PageParser<'a> {
    pub fn new(doc: &'a Document, page_index: usize, page_id: ObjectId) -> Self {
        let fonts = Self::extract_page_fonts(doc, page_id);
        Self {
            doc,
            page_index,
            page_id,
            fonts,
        }
    }

    fn extract_page_fonts(doc: &Document, page_id: ObjectId) -> HashMap<String, FontInfo> {
        let mut result = HashMap::new();
        let page_dict = match doc.get_dictionary(page_id) {
            Ok(d) => d,
            Err(_) => return result,
        };

        let resources = match page_dict.get(b"Resources") {
            Ok(Object::Dictionary(d)) => Some(d),
            Ok(Object::Reference(id)) => match doc.get_object(*id) {
                Ok(Object::Dictionary(d)) => Some(d),
                Ok(Object::Stream(s)) => Some(&s.dict),
                _ => None,
            },
            _ => None,
        };

        if let Some(res) = resources {
            let font_dict_obj = match res.get(b"Font") {
                Ok(Object::Dictionary(d)) => Some(d),
                Ok(Object::Reference(id)) => match doc.get_object(*id) {
                    Ok(Object::Dictionary(d)) => Some(d),
                    Ok(Object::Stream(s)) => Some(&s.dict),
                    _ => None,
                },
                _ => None,
            };

            if let Some(font_dict) = font_dict_obj {
                for (key, val) in font_dict.iter() {
                    let font_name = String::from_utf8_lossy(key).to_string();
                    let font_subdict = match val {
                        Object::Dictionary(d) => Some(d),
                        Object::Reference(id) => match doc.get_object(*id) {
                            Ok(Object::Dictionary(d)) => Some(d),
                            Ok(Object::Stream(s)) => Some(&s.dict),
                            _ => None,
                        },
                        _ => None,
                    };

                    if let Some(sub) = font_subdict {
                        let info = FontInfo::from_dict(doc, sub, &font_name);
                        result.insert(font_name, info);
                    }
                }
            }
        }

        result
    }

    pub fn extract_spans(&self, content: &Content) -> Vec<TextSpan> {
        let mut spans = Vec::new();
        let mut gstate_stack: Vec<GraphicsState> = Vec::new();
        let mut current_gstate = GraphicsState {
            ctm: Matrix::identity(),
            color: None,
        };

        let mut text_matrix = Matrix::identity();
        let mut line_matrix = Matrix::identity();
        let mut active_font_name = "F1".to_string();
        let mut active_font_size = 12.0;
        let mut leading = 0.0;
        let mut char_spacing = 0.0;
        let mut word_spacing = 0.0;
        let mut horizontal_scaling = 100.0;

        for (op_idx, op) in content.operations.iter().enumerate() {
            match op.operator.as_str() {
                "q" => {
                    gstate_stack.push(current_gstate.clone());
                }
                "Q" => {
                    if let Some(st) = gstate_stack.pop() {
                        current_gstate = st;
                    }
                }
                "cm" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 6 {
                        let cm = Matrix::from_array(&nums);
                        current_gstate.ctm = cm.multiply(&current_gstate.ctm);
                    }
                }
                "rg" | "RG" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 3 {
                        current_gstate.color = Some([nums[0] as f32, nums[1] as f32, nums[2] as f32]);
                    }
                }
                "g" | "G" => {
                    let nums = extract_f64_args(&op.operands);
                    if !nums.is_empty() {
                        let gray = nums[0] as f32;
                        current_gstate.color = Some([gray, gray, gray]);
                    }
                }
                "k" | "K" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 4 {
                        let c = nums[0] as f32;
                        let m = nums[1] as f32;
                        let y = nums[2] as f32;
                        let k = nums[3] as f32;
                        current_gstate.color = Some([
                            (1.0 - c) * (1.0 - k),
                            (1.0 - m) * (1.0 - k),
                            (1.0 - y) * (1.0 - k),
                        ]);
                    }
                }
                "BT" => {
                    text_matrix = Matrix::identity();
                    line_matrix = Matrix::identity();
                }
                "ET" => {
                    text_matrix = Matrix::identity();
                    line_matrix = Matrix::identity();
                }
                "Tf" => {
                    if let Some(name_obj) = op.operands.get(0) {
                        if let Some(name) = crate::parser::font::obj_to_name(name_obj) {
                            active_font_name = name;
                        }
                    }
                    if let Some(size_obj) = op.operands.get(1) {
                        if let Some(sz) = crate::parser::font::obj_to_f64(size_obj) {
                            active_font_size = sz;
                        }
                    }
                }
                "Tm" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 6 {
                        text_matrix = Matrix::from_array(&nums);
                        line_matrix = text_matrix.clone();
                    }
                }
                "Td" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 2 {
                        line_matrix = line_matrix.translate(nums[0], nums[1]);
                        text_matrix = line_matrix.clone();
                    }
                }
                "TD" => {
                    let nums = extract_f64_args(&op.operands);
                    if nums.len() >= 2 {
                        leading = -nums[1];
                        line_matrix = line_matrix.translate(nums[0], nums[1]);
                        text_matrix = line_matrix.clone();
                    }
                }
                "T*" => {
                    line_matrix = line_matrix.translate(0.0, -leading);
                    text_matrix = line_matrix.clone();
                }
                "TL" => {
                    let nums = extract_f64_args(&op.operands);
                    if !nums.is_empty() {
                        leading = nums[0];
                    }
                }
                "Tc" => {
                    let nums = extract_f64_args(&op.operands);
                    if !nums.is_empty() {
                        char_spacing = nums[0];
                    }
                }
                "Tw" => {
                    let nums = extract_f64_args(&op.operands);
                    if !nums.is_empty() {
                        word_spacing = nums[0];
                    }
                }
                "Tz" => {
                    let nums = extract_f64_args(&op.operands);
                    if !nums.is_empty() {
                        horizontal_scaling = nums[0];
                    }
                }
                "Tj" => {
                    if let Some(str_obj) = op.operands.get(0) {
                        if let Ok(bytes) = str_obj.as_str() {
                            self.process_text_run(
                                bytes,
                                op_idx,
                                None,
                                &mut text_matrix,
                                &current_gstate,
                                &active_font_name,
                                active_font_size,
                                char_spacing,
                                word_spacing,
                                horizontal_scaling,
                                &mut spans,
                            );
                        }
                    }
                }
                "'" => {
                    line_matrix = line_matrix.translate(0.0, -leading);
                    text_matrix = line_matrix.clone();
                    if let Some(str_obj) = op.operands.get(0) {
                        if let Ok(bytes) = str_obj.as_str() {
                            self.process_text_run(
                                bytes,
                                op_idx,
                                None,
                                &mut text_matrix,
                                &current_gstate,
                                &active_font_name,
                                active_font_size,
                                char_spacing,
                                word_spacing,
                                horizontal_scaling,
                                &mut spans,
                            );
                        }
                    }
                }
                "\"" => {
                    let nums = extract_f64_args(&op.operands[0..2.min(op.operands.len())]);
                    if nums.len() >= 2 {
                        word_spacing = nums[0];
                        char_spacing = nums[1];
                    }
                    line_matrix = line_matrix.translate(0.0, -leading);
                    text_matrix = line_matrix.clone();
                    if let Some(str_obj) = op.operands.get(2) {
                        if let Ok(bytes) = str_obj.as_str() {
                            self.process_text_run(
                                bytes,
                                op_idx,
                                None,
                                &mut text_matrix,
                                &current_gstate,
                                &active_font_name,
                                active_font_size,
                                char_spacing,
                                word_spacing,
                                horizontal_scaling,
                                &mut spans,
                            );
                        }
                    }
                }
                "TJ" => {
                    if let Some(arr_obj) = op.operands.get(0) {
                        if let Ok(arr) = arr_obj.as_array() {
                            for (sub_idx, item) in arr.iter().enumerate() {
                                match item {
                                    Object::String(bytes, _) => {
                                        self.process_text_run(
                                            bytes,
                                            op_idx,
                                            Some(sub_idx),
                                            &mut text_matrix,
                                            &current_gstate,
                                            &active_font_name,
                                            active_font_size,
                                            char_spacing,
                                            word_spacing,
                                            horizontal_scaling,
                                            &mut spans,
                                        );
                                    }
                                    Object::Real(k) => {
                                        let displacement = -(*k as f64) / 1000.0 * active_font_size * (horizontal_scaling / 100.0);
                                        text_matrix = text_matrix.translate(displacement, 0.0);
                                    }
                                    Object::Integer(k) => {
                                        let displacement = -(*k as f64) / 1000.0 * active_font_size * (horizontal_scaling / 100.0);
                                        text_matrix = text_matrix.translate(displacement, 0.0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        spans
    }

    fn process_text_run(
        &self,
        bytes: &[u8],
        op_index: usize,
        sub_index: Option<usize>,
        text_matrix: &mut Matrix,
        current_gstate: &GraphicsState,
        font_name: &str,
        font_size: f64,
        char_spacing: f64,
        word_spacing: f64,
        horizontal_scaling: f64,
        spans: &mut Vec<TextSpan>,
    ) {
        let font_info = self.fonts.get(font_name);
        let decoded = decode_pdf_string(
            bytes,
            font_info.and_then(|f| f.to_unicode.as_ref()),
            font_info.and_then(|f| f.encoding.as_deref()),
        );

        if decoded.is_empty() {
            return;
        }

        // Effective transform = text_matrix * CTM
        let effective_matrix = text_matrix.multiply(&current_gstate.ctm);
        let (start_x, start_y) = (effective_matrix.e, effective_matrix.f);
        let rotation = effective_matrix.rotation_degrees();

        let scale_x = (effective_matrix.a * effective_matrix.a + effective_matrix.b * effective_matrix.b).sqrt();
        let scale_y = (effective_matrix.c * effective_matrix.c + effective_matrix.d * effective_matrix.d).sqrt();
        let scale_x = if scale_x > 1e-6 { scale_x } else { 1.0 };
        let scale_y = if scale_y > 1e-6 { scale_y } else { 1.0 };

        let user_font_size = font_size * scale_y;

        let mut char_widths = Vec::new();
        let mut char_byte_ranges = Vec::new();
        let mut total_run_width_user = 0.0;
        let mut total_text_space_advance = 0.0;

        let mut byte_idx = 0;
        for c in decoded.chars() {
            let char_len = c.len_utf8();
            char_byte_ranges.push((byte_idx, byte_idx + char_len));
            byte_idx += char_len;

            let byte_code = if (c as u32) < 256 { Some(c as u8) } else { None };
            let glyph_advance = font_info
                .map(|f| f.get_glyph_advance(c, byte_code))
                .unwrap_or_else(|| crate::parser::font::get_standard_helvetica_advance(c));

            let extra_space = if c == ' ' { word_spacing } else { 0.0 };
            let text_w = ((glyph_advance / 1000.0) * font_size + char_spacing + extra_space)
                * (horizontal_scaling / 100.0);
            let user_w = text_w * scale_x;

            char_widths.push(user_w);
            total_run_width_user += user_w;
            total_text_space_advance += text_w;
        }

        spans.push(TextSpan {
            text: decoded,
            raw_bytes: bytes.to_vec(),
            page: self.page_index,
            x: start_x,
            y: start_y,
            width: total_run_width_user,
            height: user_font_size,
            font_name: font_name.to_string(),
            font_size: user_font_size,
            rotation,
            color: current_gstate.color,
            op_index,
            sub_index,
            char_widths,
            char_byte_ranges,
            scale_x,
        });

        // Advance text matrix by text space advance
        *text_matrix = text_matrix.translate(total_text_space_advance, 0.0);
    }
}

pub fn extract_f64_args(operands: &[Object]) -> Vec<f64> {
    operands
        .iter()
        .filter_map(crate::parser::font::obj_to_f64)
        .collect()
}
