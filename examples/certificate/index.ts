import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { renderPdfTemplate } from "../../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function run() {
  const dir = __dirname;
  const templatePath = path.join(dir, "certificate-template.pdf");
  const outputPath = path.join(dir, "certificate.pdf");
  const data = JSON.parse(fs.readFileSync(path.join(dir, "data.json"), "utf8"));

  console.log("Rendering certificate...");
  const result = await renderPdfTemplate({
    template: templatePath,
    data,
    overflow: "shrink",
  });

  await result.save(outputPath);
  console.log(`✓ Certificate saved to: ${outputPath}`);
}

run().catch(console.error);
