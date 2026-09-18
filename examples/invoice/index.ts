import path from "path";
import { fileURLToPath } from "url";
import { renderPdfTemplate, inspectTemplate } from "../../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function run() {
  const dir = __dirname;
  const templatePath = path.join(dir, "invoice-template.pdf");
  const outputPath = path.join(dir, "invoice.pdf");

  console.log("1. Inspecting template...");
  const inspection = await inspectTemplate(templatePath);
  console.log(`Found ${inspection.placeholders.length} placeholders on ${inspection.pagesCount} page(s).`);

  console.log("2. Rendering PDF...");
  const result = await renderPdfTemplate(templatePath, {
    customer: {
      name: "Yassir Jeraidi",
      email: "yassir@example.com",
    },
    invoice: {
      number: "INV-2026-001",
      date: "18/09/2026",
      total: 15000,
    },
  }, {
    helpers: {
      currency: (value: number) => `${value.toFixed(2)} MAD`,
    }
  });

  await result.save(outputPath);
  console.log(`✓ Invoice saved to: ${outputPath}`);
}

run().catch(console.error);
