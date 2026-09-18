use std::collections::HashMap;

/// Parsed ToUnicode CMap for converting PDF character codes to Unicode strings.
#[derive(Debug, Clone, Default)]
pub struct ToUnicodeCMap {
    pub char_map: HashMap<u32, String>,
    pub range_map: Vec<(u32, u32, u32)>, // (start_code, end_code, unicode_start)
}

impl ToUnicodeCMap {
    pub fn parse(stream_data: &[u8]) -> Self {
        let mut cmap = ToUnicodeCMap::default();
        let text = String::from_utf8_lossy(stream_data);

        // Parse beginbfchar / endbfchar blocks
        let mut in_bfchar = false;
        let mut in_bfrange = false;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains("beginbfchar") {
                in_bfchar = true;
                in_bfrange = false;
                continue;
            } else if trimmed.contains("endbfchar") {
                in_bfchar = false;
                continue;
            } else if trimmed.contains("beginbfrange") {
                in_bfrange = true;
                in_bfchar = false;
                continue;
            } else if trimmed.contains("endbfrange") {
                in_bfrange = false;
                continue;
            }

            if in_bfchar {
                // Format: <srcCode> <dstUnicode>
                let parts: Vec<&str> = trimmed
                    .split_whitespace()
                    .filter(|p| p.starts_with('<') && p.ends_with('>'))
                    .collect();
                if parts.len() >= 2 {
                    if let (Some(code), Some(uni_str)) = (
                        parse_hex_u32(parts[0]),
                        parse_hex_to_string(parts[1]),
                    ) {
                        cmap.char_map.insert(code, uni_str);
                    }
                }
            } else if in_bfrange {
                // Format: <startCode> <endCode> <startUnicode>
                let parts: Vec<&str> = trimmed
                    .split_whitespace()
                    .filter(|p| p.starts_with('<') && p.ends_with('>'))
                    .collect();
                if parts.len() >= 3 {
                    if let (Some(start), Some(end), Some(uni_start)) = (
                        parse_hex_u32(parts[0]),
                        parse_hex_u32(parts[1]),
                        parse_hex_u32(parts[2]),
                    ) {
                        cmap.range_map.push((start, end, uni_start));
                    }
                }
            }
        }

        cmap
    }

    pub fn map_code(&self, code: u32) -> Option<String> {
        if let Some(s) = self.char_map.get(&code) {
            return Some(s.clone());
        }
        for &(start, end, uni_start) in &self.range_map {
            if code >= start && code <= end {
                let offset = code - start;
                let uni = uni_start + offset;
                if let Some(c) = char::from_u32(uni) {
                    return Some(c.to_string());
                }
            }
        }
        None
    }
}

fn parse_hex_u32(hex_str: &str) -> Option<u32> {
    let clean = hex_str.trim_matches(|c| c == '<' || c == '>');
    u32::from_str_radix(clean, 16).ok()
}

fn parse_hex_to_string(hex_str: &str) -> Option<String> {
    let clean = hex_str.trim_matches(|c| c == '<' || c == '>');
    let mut bytes = Vec::new();
    let mut chars = clean.chars();
    while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
        let byte_str = format!("{}{}", c1, c2);
        if let Ok(b) = u8::from_str_radix(&byte_str, 16) {
            bytes.push(b);
        }
    }
    if bytes.len() >= 2 && bytes.len() % 2 == 0 {
        let u16s: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&u16s).ok()
    } else {
        String::from_utf8(bytes).ok()
    }
}

/// Decodes a PDF raw string into a UTF-8 Rust String.
pub fn decode_pdf_string(
    bytes: &[u8],
    to_unicode: Option<&ToUnicodeCMap>,
    _encoding: Option<&str>,
) -> String {
    // 1. Check for UTF-16BE BOM
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        if let Ok(s) = String::from_utf16(&u16s) {
            return s;
        }
    }

    // 2. Check for UTF-8 BOM
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        if let Ok(s) = std::str::from_utf8(&bytes[3..]) {
            return s.to_string();
        }
    }

    // 3. If ToUnicode CMap is available, map characters
    if let Some(cmap) = to_unicode {
        let mut result = String::new();
        // Try 1-byte codes first or 2-byte if even length and not mostly ascii
        let is_2byte = bytes.len() >= 2
            && bytes.len() % 2 == 0
            && bytes.iter().step_by(2).all(|&b| b == 0);

        if is_2byte {
            for chunk in bytes.chunks_exact(2) {
                let code = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
                if let Some(mapped) = cmap.map_code(code) {
                    result.push_str(&mapped);
                } else if let Some(c) = char::from_u32(code) {
                    result.push(c);
                } else {
                    result.push('?');
                }
            }
            return result;
        } else {
            let mut mapped_all = true;
            for &b in bytes {
                if let Some(mapped) = cmap.map_code(b as u32) {
                    result.push_str(&mapped);
                } else {
                    mapped_all = false;
                    break;
                }
            }
            if mapped_all && !result.is_empty() {
                return result;
            }
        }
    }

    // 4. If UTF-8 valid directly, use it
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // 5. Windows-1252 / WinAnsiEncoding fallback
    let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    cow.into_owned()
}

/// Encodes a UTF-8 string into PDF bytes.
/// If pure ASCII/WinAnsi, returns bytes directly.
/// If non-ASCII chars are present, returns UTF-16BE with BOM `\xFE\xFF`.
pub fn encode_pdf_string(text: &str) -> Vec<u8> {
    let mut is_pure_ascii = true;
    for b in text.bytes() {
        if b >= 0x80 {
            is_pure_ascii = false;
            break;
        }
    }

    if is_pure_ascii {
        text.as_bytes().to_vec()
    } else {
        // Encode as UTF-16BE with BOM
        let mut bytes = vec![0xFE, 0xFF];
        for u in text.encode_utf16() {
            bytes.extend_from_slice(&u.to_be_bytes());
        }
        bytes
    }
}
