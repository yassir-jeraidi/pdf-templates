use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum PdfTemplateError {
    #[error("TemplateParseError on page {page:?}: {message}")]
    TemplateParseError {
        message: String,
        page: Option<usize>,
    },

    #[error("PdfParseError: {message}")]
    PdfParseError {
        message: String,
    },

    #[error("MissingVariableError: Variable \"{expression}\" was not found on page {page} (placeholder: \"{placeholder}\")")]
    MissingVariableError {
        expression: String,
        page: usize,
        placeholder: String,
    },

    #[error("InvalidExpressionError in \"{expression}\": {message}")]
    InvalidExpressionError {
        expression: String,
        message: String,
    },

    #[error("FontError: {message}")]
    FontError {
        message: String,
    },

    #[error("RenderError: {message}")]
    RenderError {
        message: String,
    },

    #[error("OverflowError: Expression \"{expression}\" requires width {required_width:.2} which exceeds available width {available_width:.2} at font size {font_size:.2}")]
    OverflowError {
        expression: String,
        required_width: f64,
        available_width: f64,
        font_size: f64,
    },

    #[error("UnsupportedPdfError: {message}")]
    UnsupportedPdfError {
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, PdfTemplateError>;
