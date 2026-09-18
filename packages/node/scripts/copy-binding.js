const fs = require("fs");
const path = require("path");

const rootDir = path.resolve(__dirname, "../../..");
const targetDirs = [
  path.join(rootDir, "target/release"),
  path.join(rootDir, "target/debug"),
];

const filenames = [
  "libpdf_template_napi.dylib",
  "libpdf_template_napi.so",
  "pdf_template_napi.dll",
  "pdf_template_napi.node",
];

let found = null;
for (const dir of targetDirs) {
  for (const name of filenames) {
    const full = path.join(dir, name);
    if (fs.existsSync(full)) {
      found = full;
      break;
    }
  }
  if (found) break;
}

const destDir = path.resolve(__dirname, "..");
const destFile = path.join(destDir, "pdf-template-napi.node");

if (found) {
  fs.copyFileSync(found, destFile);
  console.log(`Successfully copied ${found} -> ${destFile}`);
} else {
  console.warn("Warning: Compiled native binary not found in target/release or target/debug.");
  console.warn("Run 'cargo build -p pdf-template-napi' first.");
}
