use serde_json::Value;
use crate::error::{PdfTemplateError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    StringLit(String),
    NumberLit(f64),
    Member(Box<Expr>, String),
    Index(Box<Expr>, usize),
    Call(String, Vec<Expr>),
}

pub struct ExpressionParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> ExpressionParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse_expression(input: &str) -> Result<Expr> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(PdfTemplateError::InvalidExpressionError {
                expression: input.to_string(),
                message: "Empty expression".to_string(),
            });
        }
        let mut parser = ExpressionParser { input: trimmed, pos: 0 };
        let expr = parser.parse_expr()?;
        parser.skip_whitespace();
        if parser.pos < parser.input.len() {
            return Err(PdfTemplateError::InvalidExpressionError {
                expression: input.to_string(),
                message: format!("Unexpected characters after expression at index {}", parser.pos),
            });
        }
        Ok(expr)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.skip_whitespace();
        let mut primary = self.parse_primary()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('.') => {
                    self.pos += 1;
                    self.skip_whitespace();
                    let prop = self.parse_identifier()?;
                    primary = Expr::Member(Box::new(primary), prop);
                }
                Some('[') => {
                    self.pos += 1;
                    self.skip_whitespace();
                    let idx = self.parse_integer()?;
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        self.pos += 1;
                        primary = Expr::Index(Box::new(primary), idx);
                    } else {
                        return Err(PdfTemplateError::InvalidExpressionError {
                            expression: self.input.to_string(),
                            message: "Expected ']' after array index".to_string(),
                        });
                    }
                }
                _ => break,
            }
        }

        Ok(primary)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_whitespace();
        let c = self.peek().ok_or_else(|| PdfTemplateError::InvalidExpressionError {
            expression: self.input.to_string(),
            message: "Unexpected end of input".to_string(),
        })?;

        if c == '"' || c == '\'' {
            self.parse_string_lit(c)
        } else if c.is_ascii_digit() {
            self.parse_number_lit()
        } else if c.is_alphabetic() || c == '_' || c == '$' {
            let ident = self.parse_identifier()?;
            self.skip_whitespace();
            if self.peek() == Some('(') {
                // Function call: helper(arg1, arg2...)
                self.pos += 1;
                let mut args = Vec::new();
                self.skip_whitespace();
                if self.peek() != Some(')') {
                    loop {
                        let arg = self.parse_expr()?;
                        args.push(arg);
                        self.skip_whitespace();
                        if self.peek() == Some(',') {
                            self.pos += 1;
                            self.skip_whitespace();
                        } else {
                            break;
                        }
                    }
                }
                self.skip_whitespace();
                if self.peek() == Some(')') {
                    self.pos += 1;
                    Ok(Expr::Call(ident, args))
                } else {
                    Err(PdfTemplateError::InvalidExpressionError {
                        expression: self.input.to_string(),
                        message: "Expected ')' after function arguments".to_string(),
                    })
                }
            } else {
                Ok(Expr::Ident(ident))
            }
        } else {
            Err(PdfTemplateError::InvalidExpressionError {
                expression: self.input.to_string(),
                message: format!("Unexpected character '{}'", c),
            })
        }
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'$' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(PdfTemplateError::InvalidExpressionError {
                expression: self.input.to_string(),
                message: "Expected identifier".to_string(),
            });
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_string_lit(&mut self, quote: char) -> Result<Expr> {
        self.pos += 1; // skip opening quote
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c == quote {
                let s = self.input[start..self.pos].to_string();
                self.pos += c.len_utf8();
                return Ok(Expr::StringLit(s));
            }
            self.pos += c.len_utf8();
        }
        Err(PdfTemplateError::InvalidExpressionError {
            expression: self.input.to_string(),
            message: "Unterminated string literal".to_string(),
        })
    }

    fn parse_number_lit(&mut self) -> Result<Expr> {
        let start = self.pos;
        let mut has_dot = false;
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b.is_ascii_digit() {
                self.pos += 1;
            } else if b == b'.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        let num: f64 = num_str.parse().map_err(|_| PdfTemplateError::InvalidExpressionError {
            expression: self.input.to_string(),
            message: format!("Invalid number: {}", num_str),
        })?;
        Ok(Expr::NumberLit(num))
    }

    fn parse_integer(&mut self) -> Result<usize> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(PdfTemplateError::InvalidExpressionError {
                expression: self.input.to_string(),
                message: "Expected array index integer".to_string(),
            });
        }
        let num_str = &self.input[start..self.pos];
        num_str.parse::<usize>().map_err(|_| PdfTemplateError::InvalidExpressionError {
            expression: self.input.to_string(),
            message: format!("Invalid array index: {}", num_str),
        })
    }
}

/// Evaluates an expression against a JSON Value and optional helper mappings.
pub fn evaluate_expr(
    expr: &Expr,
    data: &Value,
    helpers: Option<&std::collections::HashMap<String, String>>, // pre-computed or standard helpers
) -> Option<Value> {
    match expr {
        Expr::Ident(name) => {
            if let Some(obj) = data.as_object() {
                obj.get(name).cloned()
            } else {
                None
            }
        }
        Expr::StringLit(s) => Some(Value::String(s.clone())),
        Expr::NumberLit(n) => serde_json::Number::from_f64(*n).map(Value::Number),
        Expr::Member(target, prop) => {
            let target_val = evaluate_expr(target, data, helpers)?;
            if let Some(obj) = target_val.as_object() {
                obj.get(prop).cloned()
            } else {
                None
            }
        }
        Expr::Index(target, idx) => {
            let target_val = evaluate_expr(target, data, helpers)?;
            if let Some(arr) = target_val.as_array() {
                arr.get(*idx).cloned()
            } else {
                None
            }
        }
        Expr::Call(fn_name, args) => {
            // Built-in standard helpers
            let evaluated_args: Vec<Option<Value>> = args
                .iter()
                .map(|a| evaluate_expr(a, data, helpers))
                .collect();

            if evaluated_args.iter().any(|a| a.is_none()) {
                return None;
            }
            let clean_args: Vec<Value> = evaluated_args.into_iter().flatten().collect();

            let key = format!("{}({})", fn_name, clean_args.iter().map(value_to_string).collect::<Vec<_>>().join(","));
            if let Some(h_map) = helpers {
                if let Some(res) = h_map.get(&key) {
                    return Some(Value::String(res.clone()));
                }
            }

            match fn_name.as_str() {
                "uppercase" => clean_args.first().map(|v| {
                    Value::String(value_to_string(v).to_uppercase())
                }),
                "lowercase" => clean_args.first().map(|v| {
                    Value::String(value_to_string(v).to_lowercase())
                }),
                "trim" => clean_args.first().map(|v| {
                    Value::String(value_to_string(v).trim().to_string())
                }),
                "currency" => clean_args.first().map(|v| {
                    let num = if let Some(n) = v.as_f64() {
                        n
                    } else if let Some(n) = v.as_i64() {
                        n as f64
                    } else if let Some(s) = v.as_str() {
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    Value::String(format!("{:.2} MAD", num))
                }),
                _ => {
                    // Check custom helper map if provided
                    if let Some(h_map) = helpers {
                        let key = format!("{}({})", fn_name, clean_args.iter().map(value_to_string).collect::<Vec<_>>().join(","));
                        if let Some(res) = h_map.get(&key) {
                            return Some(Value::String(res.clone()));
                        }
                    }
                    None
                }
            }
        }
    }
}

pub fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(_) => "[object Object]".to_string(),
    }
}
