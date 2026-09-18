use lopdf::{Document, Object, Dictionary};
use crate::parser::encoding::ToUnicodeCMap;

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub name: String,
    pub base_font: String,
    pub subtype: String,
    pub first_char: u32,
    pub last_char: u32,
    pub widths: Vec<f64>,
    pub missing_width: f64,
    pub ascent: f64,
    pub descent: f64,
    pub to_unicode: Option<ToUnicodeCMap>,
    pub encoding: Option<String>,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            name: "F1".to_string(),
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            first_char: 0,
            last_char: 255,
            widths: Vec::new(),
            missing_width: 500.0,
            ascent: 750.0,
            descent: -200.0,
            to_unicode: None,
            encoding: None,
        }
    }
}

impl FontInfo {
    pub fn from_dict(doc: &Document, font_dict: &Dictionary, font_name: &str) -> Self {
        let mut info = FontInfo::default();
        info.name = font_name.to_string();

        if let Ok(bf) = font_dict.get(b"BaseFont") {
            if let Some(name) = obj_to_name(bf) {
                info.base_font = name;
            }
        }

        if let Ok(st) = font_dict.get(b"Subtype") {
            if let Some(name) = obj_to_name(st) {
                info.subtype = name;
            }
        }

        if let Ok(fc) = font_dict.get(b"FirstChar") {
            if let Some(n) = obj_to_f64(fc) {
                info.first_char = n as u32;
            }
        }

        if let Ok(lc) = font_dict.get(b"LastChar") {
            if let Some(n) = obj_to_f64(lc) {
                info.last_char = n as u32;
            }
        }

        // Widths array
        if let Ok(w_obj) = font_dict.get(b"Widths") {
            let actual_w = resolve_object(doc, w_obj);
            if let Some(arr) = actual_w.and_then(|o| o.as_array().ok()) {
                info.widths = arr.iter().filter_map(obj_to_f64).collect();
            }
        }

        // FontDescriptor for ascent, descent, missing_width
        if let Ok(fd_obj) = font_dict.get(b"FontDescriptor") {
            let actual_fd = resolve_object(doc, fd_obj);
            if let Some(fd_dict) = actual_fd.and_then(|o| o.as_dict().ok()) {
                if let Ok(asc) = fd_dict.get(b"Ascent") {
                    if let Some(f) = obj_to_f64(asc) {
                        info.ascent = f;
                    }
                }
                if let Ok(desc) = fd_dict.get(b"Descent") {
                    if let Some(f) = obj_to_f64(desc) {
                        info.descent = f;
                    }
                }
                if let Ok(mw) = fd_dict.get(b"MissingWidth") {
                    if let Some(f) = obj_to_f64(mw) {
                        info.missing_width = f;
                    }
                }
            }
        }

        // ToUnicode CMap
        if let Ok(tu_obj) = font_dict.get(b"ToUnicode") {
            let actual_tu = resolve_object(doc, tu_obj);
            if let Some(tu_stream) = actual_tu.and_then(|o| o.as_stream().ok()) {
                if let Ok(data) = tu_stream.decompressed_content() {
                    info.to_unicode = Some(ToUnicodeCMap::parse(&data));
                }
            }
        }

        // Encoding
        if let Ok(enc_obj) = font_dict.get(b"Encoding") {
            if let Some(enc_name) = obj_to_name(enc_obj) {
                info.encoding = Some(enc_name);
            }
        }

        info
    }

    /// Measures the advance width of a character in glyph space (per 1000 units).
    pub fn get_glyph_advance(&self, c: char, byte_code: Option<u8>) -> f64 {
        // 1. Try explicit Widths array
        if let Some(b) = byte_code {
            let code = b as u32;
            if code >= self.first_char && code <= self.last_char && !self.widths.is_empty() {
                let idx = (code - self.first_char) as usize;
                if idx < self.widths.len() {
                    return self.widths[idx];
                }
            }
        }

        // 2. Monospace check (Courier)
        let base = self.base_font.to_lowercase();
        if base.contains("courier") || base.contains("mono") {
            return 600.0;
        }

        // 3. Helvetica / Standard proportional metrics lookup
        get_standard_helvetica_advance(c)
    }

    /// Measures the total advance width of a string at a given font size.
    pub fn measure_text_width(&self, text: &str, font_size: f64) -> f64 {
        let mut total_advance = 0.0;
        for c in text.chars() {
            let byte_code = if (c as u32) < 256 { Some(c as u8) } else { None };
            let glyph_advance = self.get_glyph_advance(c, byte_code);
            total_advance += (glyph_advance / 1000.0) * font_size;
        }
        total_advance
    }
}

fn resolve_object<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(id) => doc.objects.get(id),
        _ => Some(obj),
    }
}

pub fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(f) => Some(*f as f64),
        Object::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

pub fn obj_to_name(obj: &Object) -> Option<String> {
    match obj {
        Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        _ => None,
    }
}

pub fn get_standard_helvetica_advance(c: char) -> f64 {
    match c {
        ' ' => 278.0,
        '!' => 278.0,
        '"' => 355.0,
        '#' => 556.0,
        '$' => 556.0,
        '%' => 889.0,
        '&' => 667.0,
        '\'' => 222.0,
        '(' | ')' => 333.0,
        '*' => 389.0,
        '+' => 584.0,
        ',' => 278.0,
        '-' => 333.0,
        '.' | '/' => 278.0,
        '0'..='9' => 556.0,
        ':' | ';' => 278.0,
        '<' | '=' | '>' => 584.0,
        '?' => 556.0,
        '@' => 1015.0,
        'A' | 'B' | 'E' | 'H' | 'K' | 'P' | 'R' | 'S' | 'V' | 'X' | 'Y' => 667.0,
        'C' | 'D' | 'N' | 'U' => 722.0,
        'F' | 'T' | 'Z' => 611.0,
        'G' | 'O' | 'Q' => 778.0,
        'I' => 278.0,
        'J' => 500.0,
        'L' => 556.0,
        'M' => 833.0,
        'W' => 944.0,
        '[' | '\\' | ']' => 278.0,
        '^' => 469.0,
        '_' => 556.0,
        '`' => 222.0,
        'a' | 'b' | 'd' | 'e' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' => 556.0,
        'c' | 'k' | 's' | 'v' | 'x' | 'y' | 'z' => 500.0,
        'f' | 't' => 278.0,
        'i' | 'j' | 'l' => 222.0,
        'm' => 833.0,
        'r' => 333.0,
        'w' => 722.0,
        '{' | '}' => 334.0,
        '|' => 260.0,
        '~' => 584.0,
        _ => 500.0,
    }
}
