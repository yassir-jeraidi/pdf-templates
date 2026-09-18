import fs from "fs";
import path from "path";

interface NativeBinding {
  coreVersion(): string;
  renderPdfNative(
    templateBuffer: Buffer,
    dataJson: string,
    optionsJson?: string,
    helpersJson?: string
  ): Buffer;
  inspectPdfNative(
    templateBuffer: Buffer,
    delimiters?: string[]
  ): string;
  validatePdfNative(
    templateBuffer: Buffer,
    dataJson: string,
    optionsJson?: string,
    helpersJson?: string
  ): string;
}

let cachedBinding: NativeBinding | null = null;

export function getNativeBinding(): NativeBinding {
  if (cachedBinding) {
    return cachedBinding;
  }

  const candidatePaths = [
    // 1. In packages/node directory
    path.resolve(__dirname, "../pdf-template-napi.node"),
    path.resolve(__dirname, "../../pdf-template-napi.node"),
    // 2. Cargo target release
    path.resolve(__dirname, "../../../target/release/pdf_template_napi.node"),
    path.resolve(__dirname, "../../../target/release/libpdf_template_napi.dylib"),
    // 3. Cargo target debug
    path.resolve(__dirname, "../../../target/debug/pdf_template_napi.node"),
    path.resolve(__dirname, "../../../target/debug/libpdf_template_napi.dylib"),
  ];

  for (const p of candidatePaths) {
    if (fs.existsSync(p)) {
      try {
        cachedBinding = require(p) as NativeBinding;
        return cachedBinding;
      } catch {
        // try next
      }
    }
  }

  throw new Error(
    "Could not locate native addon binary 'pdf-template-napi.node'. Please run 'npm run build' first."
  );
}
