import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { renderPdfTemplate, inspectTemplate } from "../../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function run() {
  const dir = __dirname;
  const templatePath = path.join(dir, "contract-template.pdf");
  const outputPath = path.join(dir, "contract.pdf");
  const data = JSON.parse(fs.readFileSync(path.join(dir, "data.json"), "utf8"));

  console.log("1. Inspecting multipage template...");
  const inspectRes = await inspectTemplate(templatePath);
  console.log(`Contract template has ${inspectRes.pagesCount} pages and ${inspectRes.placeholders.length} placeholders.`);

  console.log("2. Rendering 3-page contract...");
  const result = await renderPdfTemplate(templatePath, data);
  await result.save(outputPath);
  console.log(`✓ Multipage contract saved to: ${outputPath}`);
}

run().catch(console.error);
