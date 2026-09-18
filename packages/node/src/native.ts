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

  const { platform, arch } = process;
  const platformKey = `${platform}-${arch}`;

  // 1. Try optional platform package if installed
  const optionalPackages = [
    `@pdf-templates/${platformKey}`,
    `pdf-templates-${platformKey}`,
  ];
  for (const pkg of optionalPackages) {
    try {
      cachedBinding = require(pkg) as NativeBinding;
      return cachedBinding;
    } catch {
      // ignore
    }
  }

  const candidatePaths = [
    // 2. In package root or binaries directory
    path.resolve(__dirname, "../pdf-template-napi.node"),
    path.resolve(__dirname, `../pdf-template-napi.${platformKey}.node`),
    path.resolve(__dirname, `../binaries/pdf-template-napi.${platformKey}.node`),
    path.resolve(__dirname, "../../pdf-template-napi.node"),
    // 3. Cargo target release
    path.resolve(__dirname, "../../../target/release/pdf_template_napi.node"),
    path.resolve(__dirname, "../../../target/release/libpdf_template_napi.dylib"),
    path.resolve(__dirname, "../../../target/release/libpdf_template_napi.so"),
    path.resolve(__dirname, "../../../target/release/pdf_template_napi.dll"),
    // 4. Cargo target debug
    path.resolve(__dirname, "../../../target/debug/pdf_template_napi.node"),
    path.resolve(__dirname, "../../../target/debug/libpdf_template_napi.dylib"),
    path.resolve(__dirname, "../../../target/debug/libpdf_template_napi.so"),
    path.resolve(__dirname, "../../../target/debug/pdf_template_napi.dll"),
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
    `Could not locate native addon binary for platform '${platformKey}'. Please verify that 'pdf-templates' was installed correctly for your platform, or run 'npm run build'.`
  );
}
