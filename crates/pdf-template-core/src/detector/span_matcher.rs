use crate::model::{Placeholder, SpanMatchRef, TextSpan};

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CharInfo {
    span_idx: usize,
    char_in_span: usize,
    x: f64,
    width: f64,
    byte_range: (usize, usize),
}

pub struct SpanMatcher {
    open_delim: String,
    close_delim: String,
}

impl SpanMatcher {
    pub fn new(open_delim: &str, close_delim: &str) -> Self {
        Self {
            open_delim: open_delim.to_string(),
            close_delim: close_delim.to_string(),
        }
    }

    pub fn detect_placeholders(&self, spans: &[TextSpan]) -> Vec<Placeholder> {
        let mut placeholders = Vec::new();
        if spans.is_empty() {
            return placeholders;
        }

        // Group spans into visual lines by baseline Y and rotation
        let line_groups = self.cluster_into_lines(spans);

        for group in line_groups {
            let mut line_text = String::new();
            let mut char_map: Vec<CharInfo> = Vec::new();

            for (group_span_idx, span) in group.iter().enumerate() {
                let mut char_x = span.x;
                for (char_idx, c) in span.text.chars().enumerate() {
                    let w = span.char_widths.get(char_idx).copied().unwrap_or(0.0);
                    let byte_range = span
                        .char_byte_ranges
                        .get(char_idx)
                        .copied()
                        .unwrap_or((0, 0));

                    char_map.push(CharInfo {
                        span_idx: group_span_idx,
                        char_in_span: char_idx,
                        x: char_x,
                        width: w,
                        byte_range,
                    });

                    char_x += w;
                    line_text.push(c);
                }
            }

            // Search for placeholders in line_text
            let mut search_start = 0;
            while let Some(open_idx) = line_text[search_start..].find(&self.open_delim) {
                let abs_open = search_start + open_idx;
                let after_open = abs_open + self.open_delim.len();

                if let Some(close_idx) = line_text[after_open..].find(&self.close_delim) {
                    let abs_close = after_open + close_idx + self.close_delim.len();
                    let raw_expr = &line_text[abs_open..abs_close];
                    let inner_expr = line_text[after_open..after_open + close_idx].trim().to_string();

                    let char_start = line_text[..abs_open].chars().count();
                    let char_end = line_text[..abs_close].chars().count();

                    if !inner_expr.is_empty() && char_end <= char_map.len() && char_start < char_end {
                        let start_char_info = &char_map[char_start];
                        let end_char_info = &char_map[char_end - 1];

                        let start_span = &group[start_char_info.span_idx];
                        let end_span = &group[end_char_info.span_idx];

                        let bbox_x = start_char_info.x;
                        let bbox_y = start_span.y;
                        let bbox_width = (end_char_info.x + end_char_info.width) - bbox_x;
                        let bbox_height = start_span.height.max(end_span.height);

                        // Collect span references for replacing/slicing
                        let mut span_refs = Vec::new();
                        let mut curr_span_idx = start_char_info.span_idx;
                        while curr_span_idx <= end_char_info.span_idx {
                            let span = &group[curr_span_idx];
                            // Find range of chars in this span that are part of the placeholder
                            let mut s_char = 0;
                            let mut e_char = span.text.chars().count();

                            if curr_span_idx == start_char_info.span_idx {
                                s_char = start_char_info.char_in_span;
                            }
                            if curr_span_idx == end_char_info.span_idx {
                                e_char = end_char_info.char_in_span + 1;
                            }

                            let b_start = span
                                .char_byte_ranges
                                .get(s_char)
                                .map(|r| r.0)
                                .unwrap_or(0);
                            let b_end = if e_char > 0 && e_char <= span.char_byte_ranges.len() {
                                span.char_byte_ranges[e_char - 1].1
                            } else {
                                span.text.len()
                            };

                            span_refs.push(SpanMatchRef {
                                op_index: span.op_index,
                                sub_index: span.sub_index,
                                start_char: s_char,
                                end_char: e_char,
                                byte_start: b_start,
                                byte_end: b_end,
                            });

                            curr_span_idx += 1;
                        }

                        placeholders.push(Placeholder {
                            expression: inner_expr,
                            raw: raw_expr.to_string(),
                            page: start_span.page,
                            x: bbox_x,
                            y: bbox_y,
                            width: bbox_width.max(1.0),
                            height: bbox_height.max(1.0),
                            font_name: start_span.font_name.clone(),
                            font_size: start_span.font_size,
                            rotation: start_span.rotation,
                            color: start_span.color,
                            span_refs,
                            scale_x: start_span.scale_x,
                            is_right_aligned: false,
                        });
                    }

                    search_start = abs_close;
                } else {
                    break;
                }
            }
        }

        // Post-process placeholders: automatically detect right-aligned text
        let max_page_right = spans.iter().map(|s| s.x + s.width).fold(0.0f64, f64::max);
        let min_page_left = spans.iter().map(|s| s.x).fold(f64::MAX, f64::min);
        let page_content_width = (max_page_right - min_page_left).max(100.0);
        let mid_x = min_page_left + page_content_width * 0.45;

        // Group spans by line (y rounded to nearest 2 pt) to find the rightmost edge of each line
        let mut line_rightmost: Vec<(f64, f64)> = Vec::new();
        for span in spans {
            let r = span.x + span.width;
            if let Some(entry) = line_rightmost.iter_mut().find(|(y, _)| (span.y - *y).abs() < 2.0) {
                if r > entry.1 {
                    entry.1 = r;
                }
            } else {
                line_rightmost.push((span.y, r));
            }
        }

        for ph in &mut placeholders {
            let ph_right = ph.x + ph.width;
            let in_right_half = ph.x > mid_x;
            let has_text_to_right = spans.iter().any(|s| {
                (s.y - ph.y).abs() < 3.0 && s.x > ph_right + 2.0
            });
            let near_margin = !has_text_to_right && (max_page_right - ph_right).abs() < 10.0;
            let matches_lines = if !has_text_to_right {
                line_rightmost.iter().any(|(y, r)| {
                    (y - ph.y).abs() > 3.0 && (y - ph.y).abs() < 250.0 && (r - ph_right).abs() < 3.0
                })
            } else {
                false
            };

            ph.is_right_aligned = in_right_half && !has_text_to_right && (near_margin || matches_lines);
        }

        placeholders
    }

    pub fn cluster_into_lines<'a>(&self, spans: &'a [TextSpan]) -> Vec<Vec<&'a TextSpan>> {
        let mut groups: Vec<Vec<&'a TextSpan>> = Vec::new();

        for span in spans {
            let mut matched = false;
            for group in groups.iter_mut() {
                let first = group[0];
                let y_diff = (span.y - first.y).abs();
                let rot_diff = (span.rotation - first.rotation).abs();
                let y_tol = (first.font_size * 0.35).max(1.5);

                if y_diff <= y_tol && rot_diff <= 1.0 {
                    group.push(span);
                    matched = true;
                    break;
                }
            }

            if !matched {
                groups.push(vec![span]);
            }
        }

        // Sort spans in each group by X coordinate
        for group in groups.iter_mut() {
            group.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        }

        // Sort groups by Y descending (top of page to bottom)
        groups.sort_by(|a, b| {
            b[0].y
                .partial_cmp(&a[0].y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        groups
    }
}
