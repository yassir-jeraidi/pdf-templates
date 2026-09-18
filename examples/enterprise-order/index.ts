import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import { renderPdfTemplate, inspectTemplate } from "../../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function run() {
  const dir = __dirname;
  const templatePath = path.join(dir, "template.pdf");
  const outputPath = path.join(dir, "rendered-order.pdf");
  const dataPath = path.join(dir, "data.json");

  console.log("1. Inspecting 2-Page Enterprise Template...");
  const inspection = await inspectTemplate(templatePath);
  console.log(`Found ${inspection.placeholders.length} placeholders on ${inspection.pagesCount} page(s).`);

  console.log("2. Loading dynamic dataset...");
  const data = JSON.parse(fs.readFileSync(dataPath, "utf-8"));

  console.log(`3. Rendering dynamic order with ${data.items.length} dynamic table rows...`);
  const result = await renderPdfTemplate(templatePath, data);

  await result.save(outputPath);
  console.log(`✓ Enterprise order rendered and saved to: ${outputPath}`);
}

run().catch(console.error);
