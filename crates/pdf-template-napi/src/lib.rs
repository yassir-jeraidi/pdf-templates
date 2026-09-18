use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use pdf_template_core::{
    inspect_template, render_template, validate_template,
    PdfTemplateError, RenderOptions,
};
use serde_json::Value;

#[napi]
pub fn core_version() -> String {
    pdf_template_core::version().to_string()
}

#[napi]
pub fn render_pdf_native(
    template_buffer: Buffer,
    data_json: String,
    options_json: Option<String>,
    helpers_json: Option<String>,
) -> Result<Buffer> {
    let data: Value = serde_json::from_str(&data_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid JSON in data parameter: {}", e),
        )
    })?;

    let options: RenderOptions = match options_json {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Invalid JSON in options parameter: {}", e),
            )
        })?,
        _ => RenderOptions::default(),
    };

    let helpers: Option<HashMap<String, String>> = match helpers_json {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Invalid JSON in helpers parameter: {}", e),
            )
        })?,
        _ => None,
    };

    let result_bytes = render_template(
        template_buffer.as_ref(),
        &data,
        &options,
        helpers.as_ref(),
    )
    .map_err(format_core_error)?;

    Ok(Buffer::from(result_bytes))
}

#[napi]
pub fn inspect_pdf_native(
    template_buffer: Buffer,
    delimiters: Option<Vec<String>>,
) -> Result<String> {
    let delim_tuple = delimiters.and_then(|d| {
        if d.len() >= 2 {
            Some((d[0].clone(), d[1].clone()))
        } else {
            None
        }
    });

    let inspect_res = inspect_template(template_buffer.as_ref(), delim_tuple)
        .map_err(format_core_error)?;

    serde_json::to_string(&inspect_res).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to serialize inspection result: {}", e),
        )
    })
}

#[napi]
pub fn validate_pdf_native(
    template_buffer: Buffer,
    data_json: String,
    options_json: Option<String>,
    helpers_json: Option<String>,
) -> Result<String> {
    let data: Value = serde_json::from_str(&data_json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Invalid JSON in data parameter: {}", e),
        )
    })?;

    let options: RenderOptions = match options_json {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Invalid JSON in options parameter: {}", e),
            )
        })?,
        _ => RenderOptions::default(),
    };

    let helpers: Option<HashMap<String, String>> = match helpers_json {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Invalid JSON in helpers parameter: {}", e),
            )
        })?,
        _ => None,
    };

    let validation_res = validate_template(
        template_buffer.as_ref(),
        &data,
        &options,
        helpers.as_ref(),
    )
    .map_err(format_core_error)?;

    serde_json::to_string(&validation_res).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to serialize validation result: {}", e),
        )
    })
}

fn format_core_error(err: PdfTemplateError) -> Error {
    let json_str = serde_json::to_string(&err).unwrap_or_else(|_| format!("{}", err));
    Error::new(Status::GenericFailure, json_str)
}
