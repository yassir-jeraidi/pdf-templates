use crate::error::{PdfTemplateError, Result};
use crate::model::OverflowMode;
use crate::parser::font::FontInfo;

#[derive(Debug, Clone)]
pub struct ResolvedLayout {
    pub effective_font_size: f64,
    pub lines: Vec<String>,
    pub should_clip: bool,
    pub rendered_width: f64,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn resolve_layout(
        expression: &str,
        text: &str,
        available_width: f64,
        font_size: f64,
        font_info: &FontInfo,
        mode: OverflowMode,
        min_font_size: f64,
    ) -> Result<ResolvedLayout> {
        let text_width = font_info.measure_text_width(text, font_size);

        // If it fits within available width (or slightly within 0.5pt tolerance)
        if text_width <= available_width + 0.5 {
            return Ok(ResolvedLayout {
                effective_font_size: font_size,
                lines: vec![text.to_string()],
                should_clip: false,
                rendered_width: text_width,
            });
        }

        match mode {
            OverflowMode::Error => Err(PdfTemplateError::OverflowError {
                expression: expression.to_string(),
                required_width: text_width,
                available_width,
                font_size,
            }),
            OverflowMode::Clip => Ok(ResolvedLayout {
                effective_font_size: font_size,
                lines: vec![text.to_string()],
                should_clip: true,
                rendered_width: available_width,
            }),
            OverflowMode::Shrink => {
                let scale = available_width / text_width;
                let target_size = font_size * scale;
                if target_size < min_font_size {
                    // Check if below minimum font size
                    // We clamp to min_font_size and clip to prevent runaway microscopic text
                    let final_width = font_info.measure_text_width(text, min_font_size);
                    Ok(ResolvedLayout {
                        effective_font_size: min_font_size,
                        lines: vec![text.to_string()],
                        should_clip: final_width > available_width,
                        rendered_width: final_width.min(available_width),
                    })
                } else {
                    Ok(ResolvedLayout {
                        effective_font_size: target_size,
                        lines: vec![text.to_string()],
                        should_clip: false,
                        rendered_width: available_width,
                    })
                }
            }
            OverflowMode::Wrap => {
                let words: Vec<&str> = text.split_whitespace().collect();
                let mut lines = Vec::new();
                let mut current_line = String::new();

                for word in words {
                    let test_line = if current_line.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", current_line, word)
                    };

                    let test_width = font_info.measure_text_width(&test_line, font_size);
                    if test_width <= available_width {
                        current_line = test_line;
                    } else {
                        if !current_line.is_empty() {
                            lines.push(current_line);
                            current_line = word.to_string();
                        } else {
                            // Single word is wider than available width -> push as is
                            lines.push(word.to_string());
                            current_line = String::new();
                        }
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line);
                }

                Ok(ResolvedLayout {
                    effective_font_size: font_size,
                    lines: if lines.is_empty() { vec![text.to_string()] } else { lines },
                    should_clip: false,
                    rendered_width: available_width,
                })
            }
        }
    }
}
