export class PdfTemplateError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "PdfTemplateError";
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export class TemplateParseError extends PdfTemplateError {
  public page?: number;

  constructor(message: string, page?: number) {
    super(message);
    this.name = "TemplateParseError";
    this.page = page;
  }
}

export class PdfParseError extends PdfTemplateError {
  constructor(message: string) {
    super(message);
    this.name = "PdfParseError";
  }
}

export class MissingVariableError extends PdfTemplateError {
  public expression: string;
  public page: number;
  public placeholder: string;

  constructor(expression: string, page: number, placeholder: string) {
    const msg = `Variable "${expression}" was not found on page ${page} (placeholder: "${placeholder}").`;
    super(msg);
    this.name = "MissingVariableError";
    this.expression = expression;
    this.page = page;
    this.placeholder = placeholder;
  }
}

export class InvalidExpressionError extends PdfTemplateError {
  public expression: string;

  constructor(expression: string, message: string) {
    super(`Invalid expression "${expression}": ${message}`);
    this.name = "InvalidExpressionError";
    this.expression = expression;
  }
}

export class FontError extends PdfTemplateError {
  constructor(message: string) {
    super(message);
    this.name = "FontError";
  }
}

export class RenderError extends PdfTemplateError {
  constructor(message: string) {
    super(message);
    this.name = "RenderError";
  }
}

export class OverflowError extends PdfTemplateError {
  public expression: string;
  public requiredWidth: number;
  public availableWidth: number;
  public fontSize: number;

  constructor(
    expression: string,
    requiredWidth: number,
    availableWidth: number,
    fontSize: number
  ) {
    super(
      `Text overflow for "${expression}": required width ${requiredWidth.toFixed(
        2
      )} exceeds available width ${availableWidth.toFixed(2)} at font size ${fontSize.toFixed(2)}.`
    );
    this.name = "OverflowError";
    this.expression = expression;
    this.requiredWidth = requiredWidth;
    this.availableWidth = availableWidth;
    this.fontSize = fontSize;
  }
}

export class UnsupportedPdfError extends PdfTemplateError {
  constructor(message: string) {
    super(message);
    this.name = "UnsupportedPdfError";
  }
}

interface RawErrorPayload {
  type?: string;
  details?: any;
}

export function parseNativeError(rawError: unknown): Error {
  if (!(rawError instanceof Error)) {
    return new PdfTemplateError(String(rawError));
  }

  const rawMsg = rawError.message;
  try {
    const parsed: RawErrorPayload = JSON.parse(rawMsg);
    if (parsed && typeof parsed.type === "string") {
      const d = parsed.details || {};
      switch (parsed.type) {
        case "MissingVariableError":
          return new MissingVariableError(
            d.expression || "",
            d.page ?? 0,
            d.placeholder || ""
          );
        case "InvalidExpressionError":
          return new InvalidExpressionError(
            d.expression || "",
            d.message || ""
          );
        case "OverflowError":
          return new OverflowError(
            d.expression || "",
            d.required_width || 0,
            d.available_width || 0,
            d.font_size || 0
          );
        case "UnsupportedPdfError":
          return new UnsupportedPdfError(d.message || rawMsg);
        case "TemplateParseError":
          return new TemplateParseError(d.message || rawMsg, d.page);
        case "PdfParseError":
          return new PdfParseError(d.message || rawMsg);
        case "FontError":
          return new FontError(d.message || rawMsg);
        case "RenderError":
          return new RenderError(d.message || rawMsg);
      }
    }
  } catch {
    // Message is not JSON, check substring
  }

  if (rawMsg.includes("no selectable text")) {
    return new UnsupportedPdfError(
      "Template contains no selectable text. OCR is required for scanned PDFs."
    );
  }

  return new PdfTemplateError(rawMsg);
}
