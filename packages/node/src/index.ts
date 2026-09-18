import fs from "fs";
import fsp from "fs/promises";
import path from "path";
import {
  TemplateData,
  RenderOptions,
  RenderedPdf,
  InspectResult,
  ValidationResult,
  PlaceholderInfo,
} from "./types";
import { parseNativeError, PdfTemplateError } from "./errors";
import { getNativeBinding } from "./native";

export * from "./types";
export * from "./errors";

class RenderedPdfImpl implements RenderedPdf {
  private buffer: Buffer;

  constructor(buffer: Buffer) {
    this.buffer = buffer;
  }

  async toBuffer(): Promise<Buffer> {
    return Buffer.from(this.buffer);
  }

  toBufferSync(): Buffer {
    return Buffer.from(this.buffer);
  }

  toUint8Array(): Uint8Array {
    return new Uint8Array(this.buffer);
  }

  async save(filePath: string): Promise<void> {
    const resolved = path.resolve(filePath);
    await fsp.mkdir(path.dirname(resolved), { recursive: true });
    await fsp.writeFile(resolved, this.buffer);
  }
}

/**
 * Loads a template source (file path, Buffer, or Uint8Array) into a Buffer.
 */
function resolveTemplateBuffer(template: string | Buffer | Uint8Array): Buffer {
  if (typeof template === "string") {
    const resolved = path.resolve(template);
    if (!fs.existsSync(resolved)) {
      throw new PdfTemplateError(`Template file not found at: ${resolved}`);
    }
    return fs.readFileSync(resolved);
  }
  if (Buffer.isBuffer(template)) {
    return template;
  }
  if (template instanceof Uint8Array) {
    return Buffer.from(template);
  }
  throw new PdfTemplateError("Template must be a file path string, Buffer, or Uint8Array");
}

/**
 * Safely resolves nested properties and array indexes on an object without using eval.
 */
function safeGetProperty(data: any, expr: string): any {
  if (!data || typeof data !== "object") return undefined;
  const trimmed = expr.trim();
  if (!trimmed) return undefined;

  // Handle number literals
  if (/^-?\d+(\.\d+)?$/.test(trimmed)) {
    return Number(trimmed);
  }
  // Handle string literals
  if ((trimmed.startsWith('"') && trimmed.endsWith('"')) || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
    return trimmed.slice(1, -1);
  }

  // Tokenize property access: a.b[0].c
  const tokens = trimmed.replace(/\[(\d+)\]/g, ".$1").split(".");
  let current = data;
  for (const token of tokens) {
    if (current == null) return undefined;
    current = current[token];
  }
  return current;
}

/**
 * Pre-evaluates custom JavaScript helper functions on detected placeholder expressions.
 */
function evaluateCustomHelpers(
  placeholders: PlaceholderInfo[],
  data: TemplateData,
  helpers: Record<string, Function>
): Record<string, string> {
  const result: Record<string, string> = {};
  const fnCallRegex = /^([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\((.*)\)$/;

  for (const ph of placeholders) {
    const match = ph.expression.match(fnCallRegex);
    if (match) {
      const [, fnName, argExpr] = match;
      if (typeof helpers[fnName] === "function") {
        const argVal = safeGetProperty(data, argExpr);
        try {
          const helperOutput = helpers[fnName](argVal);
          const key = `${fnName}(${argVal})`;
          result[key] = String(helperOutput ?? "");
          result[ph.expression] = String(helperOutput ?? "");
        } catch (err: any) {
          throw new PdfTemplateError(
            `Error executing helper "${fnName}" for placeholder "${ph.raw}": ${err?.message || err}`
          );
        }
      }
    }
  }

  return result;
}

/**
 * Normalize overloaded arguments for renderPdfTemplate, inspectTemplate, validateTemplate.
 */
function normalizeRenderArgs(
  arg1: string | Buffer | Uint8Array | RenderOptions,
  arg2?: TemplateData,
  arg3?: RenderOptions
): { template: Buffer; data: TemplateData; options: RenderOptions } {
  if (typeof arg1 === "object" && !Buffer.isBuffer(arg1) && !(arg1 instanceof Uint8Array)) {
    const opts = arg1 as RenderOptions;
    if (!opts.template) {
      throw new PdfTemplateError("Missing required 'template' in render options");
    }
    const templateBuf = resolveTemplateBuffer(opts.template);
    return {
      template: templateBuf,
      data: opts.data || {},
      options: opts,
    };
  }

  const templateBuf = resolveTemplateBuffer(arg1 as string | Buffer | Uint8Array);
  const data = arg2 || {};
  const options = arg3 || {};
  return { template: templateBuf, data, options };
}

/**
 * Inspects a PDF template and returns detected placeholders, page count, and text layer metadata.
 */
export async function inspectTemplate(
  templateOrOptions: string | Buffer | Uint8Array | { template: string | Buffer | Uint8Array; delimiters?: [string, string] },
  options?: { delimiters?: [string, string] }
): Promise<InspectResult> {
  let templateBuf: Buffer;
  let delimiters: [string, string] | undefined;

  if (typeof templateOrOptions === "object" && !Buffer.isBuffer(templateOrOptions) && !(templateOrOptions instanceof Uint8Array)) {
    templateBuf = resolveTemplateBuffer(templateOrOptions.template);
    delimiters = templateOrOptions.delimiters;
  } else {
    templateBuf = resolveTemplateBuffer(templateOrOptions as string | Buffer | Uint8Array);
    delimiters = options?.delimiters;
  }

  const native = getNativeBinding();
  try {
    const jsonStr = native.inspectPdfNative(
      templateBuf,
      delimiters ? [delimiters[0], delimiters[1]] : undefined
    );
    const parsed = JSON.parse(jsonStr);
    return {
      placeholders: parsed.placeholders || [],
      pagesCount: parsed.pages_count ?? 0,
      hasTextLayer: parsed.has_text_layer ?? false,
      regions: parsed.regions || [],
    };
  } catch (err) {
    throw parseNativeError(err);
  }
}

/**
 * Validates a PDF template against the provided data and returns structured error diagnostics.
 */
export async function validateTemplate(
  arg1: string | Buffer | Uint8Array | RenderOptions,
  arg2?: TemplateData,
  arg3?: RenderOptions
): Promise<ValidationResult> {
  const { template, data, options } = normalizeRenderArgs(arg1, arg2, arg3);
  const native = getNativeBinding();

  let helpersMap: Record<string, string> = {};
  if (options.helpers && Object.keys(options.helpers).length > 0) {
    const inspected = await inspectTemplate(template, { delimiters: options.delimiters });
    helpersMap = evaluateCustomHelpers(inspected.placeholders, data, options.helpers);
  }

  try {
    const jsonStr = native.validatePdfNative(
      template,
      JSON.stringify(data),
      JSON.stringify({
        delimiters: options.delimiters || ["{{", "}}"],
        overflow: options.overflow || "shrink",
        min_font_size: options.minFontSize || 6.0,
        fallback_font: options.fallbackFont,
      }),
      JSON.stringify(helpersMap)
    );

    const parsed = JSON.parse(jsonStr);
    return {
      valid: parsed.valid ?? true,
      errors: parsed.errors || [],
      placeholders: parsed.placeholders || [],
    };
  } catch (err) {
    throw parseNativeError(err);
  }
}

/**
 * Primary API: Renders a PDF template by evaluating and replacing placeholders while preserving layout.
 */
export async function renderPdfTemplate(
  arg1: string | Buffer | Uint8Array | RenderOptions,
  arg2?: TemplateData,
  arg3?: RenderOptions
): Promise<RenderedPdf> {
  const { template, data, options } = normalizeRenderArgs(arg1, arg2, arg3);
  const native = getNativeBinding();

  let helpersMap: Record<string, string> = {};
  if (options.helpers && Object.keys(options.helpers).length > 0) {
    const inspected = await inspectTemplate(template, { delimiters: options.delimiters });
    helpersMap = evaluateCustomHelpers(inspected.placeholders, data, options.helpers);
  }

  try {
    const outputBuffer = native.renderPdfNative(
      template,
      JSON.stringify(data),
      JSON.stringify({
        delimiters: options.delimiters || ["{{", "}}"],
        overflow: options.overflow || "shrink",
        min_font_size: options.minFontSize || 6.0,
        fallback_font: options.fallbackFont,
      }),
      JSON.stringify(helpersMap)
    );

    return new RenderedPdfImpl(outputBuffer);
  } catch (err) {
    throw parseNativeError(err);
  }
}
