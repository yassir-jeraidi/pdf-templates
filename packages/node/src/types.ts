/// <reference types="node" />

export type TemplateData = Record<string, unknown>;

export type HelperFunction = (...args: any[]) => any;

export type OverflowMode = "clip" | "shrink" | "wrap" | "error";

export interface RenderOptions {
  /** Path to template PDF or Buffer / Uint8Array */
  template?: string | Uint8Array | Buffer;
  /** Template variables object */
  data?: TemplateData;
  /** Custom delimiters, default is ["{{", "}}"] */
  delimiters?: [string, string];
  /** Custom helper functions */
  helpers?: Record<string, HelperFunction>;
  /** Text overflow handling strategy, default is "shrink" */
  overflow?: OverflowMode;
  /** Minimum font size when shrinking, default is 6.0 */
  minFontSize?: number;
  /** Fallback font family name, e.g. "Helvetica" */
  fallbackFont?: string;
  /** Map of font names to .ttf/.otf file paths */
  fonts?: Record<string, string>;
}

export interface PlaceholderInfo {
  expression: string;
  raw: string;
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  font?: string;
  fontSize: number;
  rotation: number;
  color?: [number, number, number];
}

export interface TemplateRegion {
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
  name?: string;
}

export interface InspectResult {
  placeholders: PlaceholderInfo[];
  pagesCount: number;
  hasTextLayer: boolean;
  regions: TemplateRegion[];
}

export interface ValidationError {
  type: string;
  message: string;
  expression?: string;
  page?: number;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  placeholders: PlaceholderInfo[];
}

export interface RenderedPdf {
  /** Returns the rendered PDF as a Buffer asynchronously */
  toBuffer(): Promise<Buffer>;
  /** Returns the rendered PDF as a Buffer synchronously */
  toBufferSync(): Buffer;
  /** Returns the rendered PDF as a Uint8Array */
  toUint8Array(): Uint8Array;
  /** Saves the rendered PDF to the specified file path */
  save(filePath: string): Promise<void>;
}
