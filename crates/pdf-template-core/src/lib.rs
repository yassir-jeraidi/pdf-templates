pub mod error;
pub mod model;
pub mod parser;
pub mod detector;
pub mod expression;
pub mod layout;
pub mod renderer;

use std::collections::HashMap;
use lopdf::Document;
use serde_json::Value;

pub use error::{PdfTemplateError, Result};
pub use model::{
    InspectResult, OverflowMode, Placeholder, PlaceholderInfo, RenderOptions,
    TemplateRegion, ValidationError, ValidationResult,
};
pub use parser::{FontInfo, PageParser};
pub use detector::SpanMatcher;
pub use expression::{ExpressionParser, evaluate_expr, value_to_string};
pub use layout::{LayoutEngine, ResolvedLayout};
pub use renderer::{StreamRewriter, ReplacementTask, BackgroundDecoration};

fn strip_array_prefix<'a>(expr: &'a str, array_key: &str) -> Option<&'a str> {
    if expr.starts_with(array_key) {
        let rest = &expr[array_key.len()..];
        if let Some(after_dot) = rest.strip_prefix('.') {
            if let Some(after_zero) = after_dot.strip_prefix("0.") {
                Some(after_zero)
            } else if after_dot == "0" {
                Some("")
            } else {
                Some(after_dot)
            }
        } else if let Some(after_bracket) = rest.strip_prefix("[]") {
            Some(after_bracket.strip_prefix('.').unwrap_or(after_bracket))
        } else if let Some(after_bracket) = rest.strip_prefix("[*]") {
            Some(after_bracket.strip_prefix('.').unwrap_or(after_bracket))
        } else if let Some(after_zero_bracket) = rest.strip_prefix("[0]") {
            Some(after_zero_bracket.strip_prefix('.').unwrap_or(after_zero_bracket))
        } else if rest.is_empty() {
            Some("")
        } else {
            None
        }
    } else {
        None
    }
}

fn has_explicit_higher_index(expr: &str, array_key: &str) -> bool {
    if expr.starts_with(array_key) {
        let rest = &expr[array_key.len()..];
        if let Some(after_dot) = rest.strip_prefix('.') {
            if let Some(first_char) = after_dot.chars().next() {
                if first_char >= '1' && first_char <= '9' {
                    return true;
                }
            }
        } else if let Some(after_bracket) = rest.strip_prefix('[') {
            if let Some(first_char) = after_bracket.chars().next() {
                if first_char >= '1' && first_char <= '9' {
                    return true;
                }
            }
        }
    }
    false
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Inspects a PDF template and returns all detected placeholders, page count, and text layer status.
pub fn inspect_template(
    pdf_bytes: &[u8],
    delimiters: Option<(String, String)>,
) -> Result<InspectResult> {
    let doc = Document::load_mem(pdf_bytes).map_err(|e| PdfTemplateError::PdfParseError {
        message: format!("Failed to load PDF document: {}", e),
    })?;

    let (open_delim, close_delim) = delimiters.unwrap_or_else(|| ("{{".to_string(), "}}".to_string()));
    let matcher = SpanMatcher::new(&open_delim, &close_delim);

    let pages = doc.get_pages();
    let mut all_placeholders = Vec::new();
    let mut total_spans = 0;

    let mut sorted_pages: Vec<(u32, lopdf::ObjectId)> = pages.into_iter().collect();
    sorted_pages.sort_by_key(|p| p.0);

    for (page_num, (_page_no, page_id)) in sorted_pages.iter().enumerate() {
        let parser = PageParser::new(&doc, page_num, *page_id);
        let content_data = doc.get_page_content(*page_id);
        if let Ok(content) = lopdf::content::Content::decode(&content_data) {
            let spans = parser.extract_spans(&content);
            total_spans += spans.len();
            let page_placeholders = matcher.detect_placeholders(&spans);
            for p in page_placeholders {
                all_placeholders.push(p.to_info());
            }
        }
    }

    let has_text_layer = total_spans > 0;

    Ok(InspectResult {
        placeholders: all_placeholders,
        pages_count: sorted_pages.len(),
        has_text_layer,
        regions: Vec::new(),
    })
}

/// Validates a PDF template against the provided data.
pub fn validate_template(
    pdf_bytes: &[u8],
    data: &Value,
    options: &RenderOptions,
    helpers: Option<&HashMap<String, String>>,
) -> Result<ValidationResult> {
    let inspect_res = inspect_template(pdf_bytes, Some(options.delimiters.clone()))?;
    let mut errors = Vec::new();

    if !inspect_res.has_text_layer {
        errors.push(ValidationError {
            error_type: "UNSUPPORTED_PDF".to_string(),
            message: "Template contains no selectable text. OCR is required for scanned PDFs.".to_string(),
            expression: None,
            page: None,
        });
        return Ok(ValidationResult {
            valid: false,
            errors,
            placeholders: inspect_res.placeholders,
        });
    }

    for ph in &inspect_res.placeholders {
        let text_opt = if let Some(val) = helpers.and_then(|h| h.get(&ph.expression)) {
            Some(val.clone())
        } else {
            let ast = match ExpressionParser::parse_expression(&ph.expression) {
                Ok(expr) => expr,
                Err(e) => {
                    errors.push(ValidationError {
                        error_type: "INVALID_EXPRESSION".to_string(),
                        message: format!("Malformed expression: {}", e),
                        expression: Some(ph.expression.clone()),
                        page: Some(ph.page),
                    });
                    continue;
                }
            };
            evaluate_expr(&ast, data, helpers).map(|v| value_to_string(&v))
        };

        match text_opt {
            Some(text) => {
                let font_info = FontInfo::default();
                if let Err(e) = LayoutEngine::resolve_layout(
                    &ph.expression,
                    &text,
                    ph.width,
                    ph.font_size,
                    &font_info,
                    options.overflow,
                    options.min_font_size,
                ) {
                    errors.push(ValidationError {
                        error_type: "OVERFLOW".to_string(),
                        message: format!("{}", e),
                        expression: Some(ph.expression.clone()),
                        page: Some(ph.page),
                    });
                }
            }
            None => {
                errors.push(ValidationError {
                    error_type: "MISSING_VARIABLE".to_string(),
                    message: format!("Variable \"{}\" was not found", ph.expression),
                    expression: Some(ph.expression.clone()),
                    page: Some(ph.page),
                });
            }
        }
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        placeholders: inspect_res.placeholders,
    })
}

fn compute_available_width(ph: &Placeholder, spans: &[crate::model::TextSpan]) -> f64 {
    let max_page_right = spans.iter().map(|s| s.x + s.width).fold(0.0f64, f64::max);
    if ph.is_right_aligned {
        // Right aligned: available width is distance to the nearest span to the left on the same line
        let left_barrier = spans.iter()
            .filter(|s| (s.y - ph.y).abs() < 4.0 && (s.x + s.width) < ph.x + 1.0)
            .map(|s| s.x + s.width)
            .fold(0.0f64, f64::max);
        let space = (ph.x + ph.width) - left_barrier - 5.0;
        ph.width.max(space)
    } else {
        // Left aligned: find nearest non-inline span to the right on the same line
        let right_barrier = spans.iter()
            .filter(|s| (s.y - ph.y).abs() < 4.0 && s.x > ph.x + ph.width + 10.0)
            .map(|s| s.x)
            .fold(f64::MAX, f64::min);
        let space = if right_barrier < f64::MAX {
            right_barrier - ph.x - 5.0
        } else {
            max_page_right - ph.x
        };
        ph.width.max(space)
    }
}

/// Renders a PDF template by replacing all detected placeholders with evaluated values.
pub fn render_template(
    pdf_bytes: &[u8],
    data: &Value,
    options: &RenderOptions,
    helpers: Option<&HashMap<String, String>>,
) -> Result<Vec<u8>> {
    let mut doc = Document::load_mem(pdf_bytes).map_err(|e| PdfTemplateError::PdfParseError {
        message: format!("Failed to load PDF document: {}", e),
    })?;

    let (open_delim, close_delim) = &options.delimiters;
    let matcher = SpanMatcher::new(open_delim, close_delim);

    let pages = doc.get_pages();
    let mut sorted_pages: Vec<(u32, lopdf::ObjectId)> = pages.into_iter().collect();
    sorted_pages.sort_by_key(|p| p.0);

    let mut total_spans = 0;

    for (page_idx, (_page_no, page_id)) in sorted_pages.iter().enumerate() {
        let parser = PageParser::new(&doc, page_idx, *page_id);
        let content_data = doc.get_page_content(*page_id);

        let content = match lopdf::content::Content::decode(&content_data) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let spans = parser.extract_spans(&content);
        total_spans += spans.len();

        let placeholders = matcher.detect_placeholders(&spans);
        if placeholders.is_empty() {
            continue;
        }

        let mut tasks = Vec::new();
        let mut decorations = Vec::new();

        // Check for repeating array prototype groups
        let mut repeating_keys: Vec<(String, Vec<Value>, Vec<Placeholder>)> = Vec::new();
        let mut handled_ph_indices = std::collections::HashSet::new();

        if let Some(obj) = data.as_object() {
            for (k, v) in obj {
                if let Some(arr) = v.as_array() {
                    // Check if there is any placeholder with an explicit higher index (e.g. items.1.* or items[1].*)
                    let has_multi_rows = placeholders.iter().any(|ph| has_explicit_higher_index(&ph.expression, k));
                    if !has_multi_rows {
                        let proto_indices: Vec<usize> = placeholders.iter().enumerate().filter_map(|(idx, ph)| {
                            if strip_array_prefix(&ph.expression, k).is_some() {
                                Some(idx)
                            } else {
                                None
                            }
                        }).collect();

                        if !proto_indices.is_empty() {
                            let mut proto_phs = Vec::new();
                            for idx in proto_indices {
                                handled_ph_indices.insert(idx);
                                proto_phs.push(placeholders[idx].clone());
                            }
                            repeating_keys.push((k.clone(), arr.clone(), proto_phs));
                        }
                    }
                }
            }
        }

        // Process dynamic repeating rows
        for (arr_key, arr, proto_phs) in repeating_keys {
            if proto_phs.is_empty() {
                continue;
            }

            let y_0 = proto_phs.iter().map(|p| p.y).fold(0.0f64, f64::max);
            let min_y = proto_phs.iter().map(|p| p.y).fold(f64::MAX, f64::min);
            let row_span = (y_0 - min_y).max(0.0);
            let row_pitch = if proto_phs.len() == 1 {
                (proto_phs[0].font_size * 2.3).max(15.0)
            } else {
                (row_span + 17.0).max(28.0)
            };

            let min_x = proto_phs.iter().map(|p| p.x).fold(f64::MAX, f64::min);
            let max_x = proto_phs.iter().map(|p| p.x + p.width).fold(0.0f64, f64::max);
            let table_left = (min_x - 8.0).max(40.0);
            let table_width = (max_x - table_left + 15.0).min(595.28 - table_left);

            let n_items = arr.len();
            if n_items == 0 {
                // Erase prototype text if array is empty
                for ph in proto_phs {
                    let font_info = parser.fonts.get(&ph.font_name).cloned().unwrap_or_default();
                    let layout = LayoutEngine::resolve_layout(
                        &ph.expression,
                        "",
                        ph.width,
                        ph.font_size,
                        &font_info,
                        options.overflow,
                        options.min_font_size,
                    )?;
                    tasks.push(ReplacementTask {
                        placeholder: ph,
                        layout,
                    });
                }
                continue;
            }

            for r in 0..n_items {
                let y_r = y_0 - (r as f64) * row_pitch;

                // Add zebra background / row borders if this is a multi-column table row
                if table_width > 200.0 && proto_phs.len() > 1 {
                    let box_y = min_y - (r as f64) * row_pitch - 7.0;
                    if r % 2 == 0 {
                        decorations.push(BackgroundDecoration {
                            x: table_left,
                            y: box_y,
                            width: table_width,
                            height: row_pitch,
                            fill_color: Some([0.97, 0.98, 0.99]),
                            border_color: Some([0.90, 0.92, 0.94]),
                            border_width: 0.5,
                        });
                    } else {
                        decorations.push(BackgroundDecoration {
                            x: table_left,
                            y: box_y,
                            width: table_width,
                            height: 0.0,
                            fill_color: None,
                            border_color: Some([0.90, 0.92, 0.94]),
                            border_width: 0.5,
                        });
                    }
                }

                for proto_ph in &proto_phs {
                    let field = strip_array_prefix(&proto_ph.expression, &arr_key).unwrap_or("");
                    let text = if field.is_empty() {
                        value_to_string(&arr[r])
                    } else if let Some(item_obj) = arr[r].as_object() {
                        item_obj.get(field).map(value_to_string).unwrap_or_default()
                    } else {
                        value_to_string(&arr[r])
                    };

                    let font_info = parser.fonts.get(&proto_ph.font_name).cloned().unwrap_or_default();
                    let available_width = compute_available_width(proto_ph, &spans);

                    let ph_for_task = if r == 0 {
                        proto_ph.clone()
                    } else {
                        let mut dynamic_ph = proto_ph.clone();
                        dynamic_ph.expression = format!("{}.{}.{}", arr_key, r, field);
                        dynamic_ph.y = y_r + (proto_ph.y - y_0);
                        dynamic_ph.span_refs = Vec::new();
                        dynamic_ph
                    };

                    let layout = LayoutEngine::resolve_layout(
                        &ph_for_task.expression,
                        &text,
                        available_width,
                        ph_for_task.font_size,
                        &font_info,
                        options.overflow,
                        options.min_font_size,
                    )?;

                    tasks.push(ReplacementTask {
                        placeholder: ph_for_task,
                        layout,
                    });
                }
            }
        }

        // Process all remaining regular placeholders
        for (idx, ph) in placeholders.into_iter().enumerate() {
            if handled_ph_indices.contains(&idx) {
                continue;
            }

            let text = if let Some(val) = helpers.and_then(|h| h.get(&ph.expression)) {
                val.clone()
            } else {
                let ast = ExpressionParser::parse_expression(&ph.expression)?;
                let evaluated_val = evaluate_expr(&ast, data, helpers).ok_or_else(|| {
                    PdfTemplateError::MissingVariableError {
                        expression: ph.expression.clone(),
                        page: ph.page,
                        placeholder: ph.raw.clone(),
                    }
                })?;
                value_to_string(&evaluated_val)
            };

            let font_info = parser.fonts.get(&ph.font_name).cloned().unwrap_or_default();
            let available_width = compute_available_width(&ph, &spans);

            let layout = LayoutEngine::resolve_layout(
                &ph.expression,
                &text,
                available_width,
                ph.font_size,
                &font_info,
                options.overflow,
                options.min_font_size,
            )?;

            tasks.push(ReplacementTask {
                placeholder: ph,
                layout,
            });
        }

        let mut rewriter = StreamRewriter::new(&mut doc, *page_id, page_idx);
        rewriter.apply_replacements_with_decorations(&tasks, &decorations)?;
    }

    if total_spans == 0 {
        return Err(PdfTemplateError::UnsupportedPdfError {
            message: "Template contains no selectable text. OCR is required for scanned PDFs.".to_string(),
        });
    }

    let mut output = Vec::new();
    doc.save_to(&mut output).map_err(|e| PdfTemplateError::RenderError {
        message: format!("Failed to save rendered PDF: {}", e),
    })?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{Document, Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};
    use serde_json::json;

    fn create_test_pdf(text_runs: &[&str]) -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let font_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let mut font = Dictionary::new();
        font.set("Type", Object::Name(b"Font".to_vec()));
        font.set("Subtype", Object::Name(b"Type1".to_vec()));
        font.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        doc.objects.insert(font_id, Object::Dictionary(font));

        let mut resources = Dictionary::new();
        let mut fonts = Dictionary::new();
        fonts.set("F1", Object::Reference(font_id));
        resources.set("Font", Object::Dictionary(fonts));

        let mut operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
            Operation::new("Tm", vec![
                Object::Real(1.0), Object::Real(0.0),
                Object::Real(0.0), Object::Real(1.0),
                Object::Real(100.0), Object::Real(700.0),
            ]),
        ];

        for (idx, run) in text_runs.iter().enumerate() {
            if idx > 0 {
                operations.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]));
            }
            operations.push(Operation::new("Tj", vec![Object::string_literal(*run)]));
        }

        operations.push(Operation::new("ET", vec![]));

        let content = Content { operations };
        let content_bytes = content.encode().unwrap();
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), content_bytes));

        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set("Resources", Object::Dictionary(resources));
        page.set("MediaBox", vec![0.into(), 0.into(), 600.into(), 800.into()]);
        page.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(page));

        let mut pages = Dictionary::new();
        pages.set("Type", Object::Name(b"Pages".to_vec()));
        pages.set("Kids", vec![Object::Reference(page_id)]);
        pages.set("Count", Object::Integer(1));
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn test_inspect_placeholders() {
        let pdf = create_test_pdf(&[
            "Customer: {{customer.name}}",
            "Email: {{customer.email}}",
        ]);

        let res = inspect_template(&pdf, None).unwrap();
        assert_eq!(res.placeholders.len(), 2);
        assert_eq!(res.placeholders[0].expression, "customer.name");
        assert_eq!(res.placeholders[1].expression, "customer.email");
        assert_eq!(res.pages_count, 1);
        assert!(res.has_text_layer);
    }

    #[test]
    fn test_render_placeholders() {
        let pdf = create_test_pdf(&[
            "Customer: {{customer.name}}",
            "Email: {{customer.email}}",
        ]);

        let data = json!({
            "customer": {
                "name": "Yassir Jeraidi",
                "email": "yassir@example.com"
            }
        });

        let rendered = render_template(&pdf, &data, &RenderOptions::default(), None).unwrap();
        assert!(!rendered.is_empty());

        // Reopen rendered PDF and inspect placeholders - should now be empty!
        let post_inspect = inspect_template(&rendered, None).unwrap();
        assert_eq!(post_inspect.placeholders.len(), 0);

        // Verify that the replacement text exists in the PDF content stream!
        let doc = Document::load_mem(&rendered).unwrap();
        let pages = doc.get_pages();
        let page_id = pages.values().next().unwrap();
        let content_data = doc.get_page_content(*page_id);
        let content_str = String::from_utf8_lossy(&content_data);

        assert!(content_str.contains("Yassir Jeraidi"));
        assert!(content_str.contains("yassir@example.com"));
        assert!(!content_str.contains("{{customer.name}}"));
        assert!(!content_str.contains("{{customer.email}}"));
    }

    #[test]
    fn test_custom_delimiters() {
        let pdf = create_test_pdf(&["Hello [[user.name]]"]);
        let options = RenderOptions {
            delimiters: ("[[".to_string(), "]]".to_string()),
            ..Default::default()
        };

        let data = json!({ "user": { "name": "Alice" } });
        let rendered = render_template(&pdf, &data, &options, None).unwrap();

        let doc = Document::load_mem(&rendered).unwrap();
        let page_id = doc.get_pages().values().next().copied().unwrap();
        let content_str = String::from_utf8_lossy(&doc.get_page_content(page_id)).to_string();

        assert!(content_str.contains("Alice"));
        assert!(!content_str.contains("[[user.name]]"));
    }

    #[test]
    fn test_missing_variable_error() {
        let pdf = create_test_pdf(&["Hello {{missing.var}}"]);
        let data = json!({ "user": {} });

        let err = render_template(&pdf, &data, &RenderOptions::default(), None).unwrap_err();
        match err {
            PdfTemplateError::MissingVariableError { expression, page, .. } => {
                assert_eq!(expression, "missing.var");
                assert_eq!(page, 0);
            }
            _ => panic!("Expected MissingVariableError, got {:?}", err),
        }
    }

    #[test]
    fn test_split_placeholder_across_spans() {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let font_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let mut font = Dictionary::new();
        font.set("Type", Object::Name(b"Font".to_vec()));
        font.set("Subtype", Object::Name(b"Type1".to_vec()));
        font.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        doc.objects.insert(font_id, Object::Dictionary(font));

        let mut resources = Dictionary::new();
        let mut fonts = Dictionary::new();
        fonts.set("F1", Object::Reference(font_id));
        resources.set("Font", Object::Dictionary(fonts));

        // Use TJ with split fragments: [ (Customer: {{cust) -2 (omer.name}}) ]
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
                Operation::new("Tm", vec![
                    Object::Real(1.0), Object::Real(0.0),
                    Object::Real(0.0), Object::Real(1.0),
                    Object::Real(50.0), Object::Real(500.0),
                ]),
                Operation::new("TJ", vec![Object::Array(vec![
                    Object::string_literal("Customer: {{cust"),
                    Object::Real(-2.0),
                    Object::string_literal("omer.name}}"),
                ])]),
                Operation::new("ET", vec![]),
            ],
        };

        let content_bytes = content.encode().unwrap();
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), content_bytes));

        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set("Resources", Object::Dictionary(resources));
        page.set("MediaBox", vec![0.into(), 0.into(), 600.into(), 800.into()]);
        page.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(page));

        let mut pages = Dictionary::new();
        pages.set("Type", Object::Name(b"Pages".to_vec()));
        pages.set("Kids", vec![Object::Reference(page_id)]);
        pages.set("Count", Object::Integer(1));
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer).unwrap();

        // Inspect and verify detection across split spans!
        let inspected = inspect_template(&buffer, None).unwrap();
        assert_eq!(inspected.placeholders.len(), 1);
        assert_eq!(inspected.placeholders[0].expression, "customer.name");

        // Render replacement
        let data = json!({ "customer": { "name": "Yassir Jeraidi" } });
        let rendered = render_template(&buffer, &data, &RenderOptions::default(), None).unwrap();

        let doc2 = Document::load_mem(&rendered).unwrap();
        let page_id2 = doc2.get_pages().values().next().copied().unwrap();
        let content_str = String::from_utf8_lossy(&doc2.get_page_content(page_id2)).to_string();

        assert!(content_str.contains("Yassir Jeraidi"));
        assert!(!content_str.contains("customer.name"));
    }

    #[test]
    fn test_nested_properties_and_arrays_and_whitespace() {
        let pdf = create_test_pdf(&[
            "City: {{ customer.address.city }}",
            "Item: {{ items[0].name }}",
            "Price: {{ items[0].price }}",
        ]);

        let data = json!({
            "customer": {
                "address": {
                    "city": "Casablanca"
                }
            },
            "items": [
                { "name": "Cloud Hosting", "price": 450 }
            ]
        });

        let rendered = render_template(&pdf, &data, &RenderOptions::default(), None).unwrap();
        let doc = Document::load_mem(&rendered).unwrap();
        let page_id = doc.get_pages().values().next().copied().unwrap();
        let content_str = String::from_utf8_lossy(&doc.get_page_content(page_id)).to_string();

        assert!(content_str.contains("Casablanca"));
        assert!(content_str.contains("Cloud Hosting"));
        assert!(content_str.contains("450"));
    }

    #[test]
    fn test_helpers_evaluation() {
        let pdf = create_test_pdf(&[
            "Total: {{ currency(invoice.total) }}",
            "Name: {{ uppercase(customer.name) }}",
        ]);

        let data = json!({
            "customer": { "name": "yassir" },
            "invoice": { "total": 15000 }
        });

        let rendered = render_template(&pdf, &data, &RenderOptions::default(), None).unwrap();
        let doc = Document::load_mem(&rendered).unwrap();
        let page_id = doc.get_pages().values().next().copied().unwrap();
        let content_str = String::from_utf8_lossy(&doc.get_page_content(page_id)).to_string();

        assert!(content_str.contains("15000.00 MAD"));
        assert!(content_str.contains("YASSIR"));
    }

    #[test]
    fn test_overflow_modes() {
        let pdf = create_test_pdf(&["Short: {{val}}"]);
        let data = json!({ "val": "Very long text that will exceed bounding box by a lot" });

        // Error mode should fail
        let err_options = RenderOptions {
            overflow: OverflowMode::Error,
            ..Default::default()
        };
        let res_err = render_template(&pdf, &data, &err_options, None);
        assert!(res_err.is_err());
        match res_err.unwrap_err() {
            PdfTemplateError::OverflowError { expression, .. } => {
                assert_eq!(expression, "val");
            }
            other => panic!("Expected OverflowError, got {:?}", other),
        }

        // Shrink mode should succeed
        let shrink_options = RenderOptions {
            overflow: OverflowMode::Shrink,
            ..Default::default()
        };
        let res_shrink = render_template(&pdf, &data, &shrink_options, None);
        assert!(res_shrink.is_ok());

        // Clip mode should succeed
        let clip_options = RenderOptions {
            overflow: OverflowMode::Clip,
            ..Default::default()
        };
        let res_clip = render_template(&pdf, &data, &clip_options, None);
        assert!(res_clip.is_ok());
    }

    #[test]
    fn test_scanned_pdf_error() {
        // PDF with empty content stream (no text layer)
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let content_id = doc.add_object(Stream::new(Dictionary::new(), vec![]));

        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set("Contents", Object::Reference(content_id));
        page.set("MediaBox", vec![0.into(), 0.into(), 600.into(), 800.into()]);
        doc.objects.insert(page_id, Object::Dictionary(page));

        let mut pages = Dictionary::new();
        pages.set("Type", Object::Name(b"Pages".to_vec()));
        pages.set("Kids", vec![Object::Reference(page_id)]);
        pages.set("Count", Object::Integer(1));
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer).unwrap();

        let data = json!({ "user": "test" });
        let res = render_template(&buffer, &data, &RenderOptions::default(), None);
        assert!(res.is_err());
        match res.unwrap_err() {
            PdfTemplateError::UnsupportedPdfError { message } => {
                assert!(message.contains("no selectable text"));
            }
            other => panic!("Expected UnsupportedPdfError, got {:?}", other),
        }
    }

    #[test]
    fn test_multipage_pdf() {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let font_id = doc.new_object_id();
        let page1_id = doc.new_object_id();
        let page2_id = doc.new_object_id();

        let mut font = Dictionary::new();
        font.set("Type", Object::Name(b"Font".to_vec()));
        font.set("Subtype", Object::Name(b"Type1".to_vec()));
        font.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        doc.objects.insert(font_id, Object::Dictionary(font));

        let mut resources = Dictionary::new();
        let mut fonts = Dictionary::new();
        fonts.set("F1", Object::Reference(font_id));
        resources.set("Font", Object::Dictionary(fonts));

        // Page 1 content
        let c1 = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
                Operation::new("Tm", vec![
                    Object::Real(1.0), Object::Real(0.0),
                    Object::Real(0.0), Object::Real(1.0),
                    Object::Real(100.0), Object::Real(700.0),
                ]),
                Operation::new("Tj", vec![Object::string_literal("Page 1: {{page1_title}}")]),
                Operation::new("ET", vec![]),
            ],
        };
        let s1 = doc.add_object(Stream::new(Dictionary::new(), c1.encode().unwrap()));

        // Page 2 content
        let c2 = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
                Operation::new("Tm", vec![
                    Object::Real(1.0), Object::Real(0.0),
                    Object::Real(0.0), Object::Real(1.0),
                    Object::Real(100.0), Object::Real(700.0),
                ]),
                Operation::new("Tj", vec![Object::string_literal("Page 2: {{page2_title}}")]),
                Operation::new("ET", vec![]),
            ],
        };
        let s2 = doc.add_object(Stream::new(Dictionary::new(), c2.encode().unwrap()));

        let mut p1 = Dictionary::new();
        p1.set("Type", Object::Name(b"Page".to_vec()));
        p1.set("Parent", Object::Reference(pages_id));
        p1.set("Resources", Object::Dictionary(resources.clone()));
        p1.set("MediaBox", vec![0.into(), 0.into(), 600.into(), 800.into()]);
        p1.set("Contents", Object::Reference(s1));
        doc.objects.insert(page1_id, Object::Dictionary(p1));

        let mut p2 = Dictionary::new();
        p2.set("Type", Object::Name(b"Page".to_vec()));
        p2.set("Parent", Object::Reference(pages_id));
        p2.set("Resources", Object::Dictionary(resources));
        p2.set("MediaBox", vec![0.into(), 0.into(), 600.into(), 800.into()]);
        p2.set("Contents", Object::Reference(s2));
        doc.objects.insert(page2_id, Object::Dictionary(p2));

        let mut pages = Dictionary::new();
        pages.set("Type", Object::Name(b"Pages".to_vec()));
        pages.set("Kids", vec![Object::Reference(page1_id), Object::Reference(page2_id)]);
        pages.set("Count", Object::Integer(2));
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let mut catalog = Dictionary::new();
        catalog.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buffer = Vec::new();
        doc.save_to(&mut buffer).unwrap();

        // Inspect: 2 placeholders, across 2 pages
        let inspect_res = inspect_template(&buffer, None).unwrap();
        assert_eq!(inspect_res.pages_count, 2);
        assert_eq!(inspect_res.placeholders.len(), 2);
        assert_eq!(inspect_res.placeholders[0].page, 0);
        assert_eq!(inspect_res.placeholders[1].page, 1);

        // Render
        let data = json!({
            "page1_title": "Invoice Details",
            "page2_title": "Terms & Conditions"
        });
        let rendered = render_template(&buffer, &data, &RenderOptions::default(), None).unwrap();

        let doc_rendered = Document::load_mem(&rendered).unwrap();
        assert_eq!(doc_rendered.get_pages().len(), 2);

        let p1_content = String::from_utf8_lossy(&doc_rendered.get_page_content(page1_id)).to_string();
        let p2_content = String::from_utf8_lossy(&doc_rendered.get_page_content(page2_id)).to_string();

        assert!(p1_content.contains("Invoice Details"));
        assert!(!p1_content.contains("page1_title"));
        assert!(p2_content.contains("Terms & Conditions"));
        assert!(!p2_content.contains("page2_title"));
    }
}
