use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OverflowMode {
    #[default]
    Shrink,
    Clip,
    Wrap,
    Error,
}

fn default_delimiters() -> (String, String) {
    ("{{".to_string(), "}}".to_string())
}

fn default_min_font_size() -> f64 {
    6.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOptions {
    #[serde(default = "default_delimiters")]
    pub delimiters: (String, String),
    #[serde(default)]
    pub overflow: OverflowMode,
    #[serde(default = "default_min_font_size")]
    pub min_font_size: f64,
    pub fallback_font: Option<String>,
    #[serde(default)]
    pub font_paths: HashMap<String, String>,
    #[serde(default)]
    pub custom_fonts: HashMap<String, Vec<u8>>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            delimiters: default_delimiters(),
            overflow: OverflowMode::Shrink,
            min_font_size: default_min_font_size(),
            fallback_font: None,
            font_paths: HashMap::new(),
            custom_fonts: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderInfo {
    pub expression: String,
    pub raw: String,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font: Option<String>,
    pub font_size: f64,
    pub rotation: f64,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub struct SpanMatchRef {
    pub op_index: usize,
    pub sub_index: Option<usize>,
    pub start_char: usize,
    pub end_char: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone)]
pub struct Placeholder {
    pub expression: String,
    pub raw: String,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_name: String,
    pub font_size: f64,
    pub rotation: f64,
    pub color: Option<[f32; 3]>,
    pub span_refs: Vec<SpanMatchRef>,
}

impl Placeholder {
    pub fn to_info(&self) -> PlaceholderInfo {
        PlaceholderInfo {
            expression: self.expression.clone(),
            raw: self.raw.clone(),
            page: self.page,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            font: Some(self.font_name.clone()),
            font_size: self.font_size,
            rotation: self.rotation,
            color: self.color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub raw_bytes: Vec<u8>,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub font_name: String,
    pub font_size: f64,
    pub rotation: f64,
    pub color: Option<[f32; 3]>,
    pub op_index: usize,
    pub sub_index: Option<usize>,
    pub char_widths: Vec<f64>,
    pub char_byte_ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRegion {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    pub placeholders: Vec<PlaceholderInfo>,
    pub pages_count: usize,
    pub has_text_layer: bool,
    pub regions: Vec<TemplateRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub expression: Option<String>,
    pub page: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub placeholders: Vec<PlaceholderInfo>,
}
