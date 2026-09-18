#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { renderPdfTemplate, inspectTemplate, validateTemplate } from "./index";

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === "--help" || command === "-h") {
    printHelp();
    process.exit(0);
  }

  if (command === "--version" || command === "-v") {
    const pkg = require("../package.json");
    console.log(`pdf-template-engine v${pkg.version}`);
    process.exit(0);
  }

  try {
    switch (command) {
      case "render": {
        const templatePath = args[1];
        const dataPath = args[2];
        let outputPath = "output.pdf";

        const outIdx = args.indexOf("-o") !== -1 ? args.indexOf("-o") : args.indexOf("--output");
        if (outIdx !== -1 && args[outIdx + 1]) {
          outputPath = args[outIdx + 1];
        }

        if (!templatePath || !dataPath) {
          console.error("Usage: pdf-template render <template.pdf> <data.json> [-o output.pdf]");
          process.exit(1);
        }

        const dataContent = fs.readFileSync(path.resolve(dataPath), "utf8");
        const data = JSON.parse(dataContent);

        console.log(`Rendering "${templatePath}" with data from "${dataPath}"...`);
        const result = await renderPdfTemplate(templatePath, data);
        await result.save(outputPath);
        console.log(`✓ Generated PDF saved to "${outputPath}"`);
        break;
      }

      case "inspect": {
        const templatePath = args[1];
        if (!templatePath) {
          console.error("Usage: pdf-template inspect <template.pdf>");
          process.exit(1);
        }

        console.log(`Inspecting template "${templatePath}"...\n`);
        const result = await inspectTemplate(templatePath);

        console.log(`Pages: ${result.pagesCount}`);
        console.log(`Selectable Text Layer: ${result.hasTextLayer ? "Yes" : "No"}`);
        console.log(`Placeholders Found: ${result.placeholders.length}\n`);

        if (result.placeholders.length > 0) {
          console.log("Detected Placeholders:");
          for (const [idx, ph] of result.placeholders.entries()) {
            console.log(
              `  [${idx + 1}] Expression: "${ph.expression}" | Page: ${ph.page + 1} | Box: (${ph.x.toFixed(1)}, ${ph.y.toFixed(1)}, ${ph.width.toFixed(1)}x${ph.height.toFixed(1)})`
            );
          }
        }
        break;
      }

      case "validate": {
        const templatePath = args[1];
        const dataPath = args[2];
        if (!templatePath || !dataPath) {
          console.error("Usage: pdf-template validate <template.pdf> <data.json>");
          process.exit(1);
        }

        const dataContent = fs.readFileSync(path.resolve(dataPath), "utf8");
        const data = JSON.parse(dataContent);

        console.log(`Validating "${templatePath}" against "${dataPath}"...\n`);
        const result = await validateTemplate(templatePath, data);

        if (result.valid) {
          console.log(`✓ Template is VALID (${result.placeholders.length} placeholder(s) matched successfully).`);
        } else {
          console.error(`✗ Validation FAILED with ${result.errors.length} error(s):`);
          for (const err of result.errors) {
            console.error(`  - [${err.type}] ${err.message} (Page: ${err.page != null ? err.page + 1 : "N/A"})`);
          }
          process.exit(1);
        }
        break;
      }

      default:
        console.error(`Unknown command: "${command}"`);
        printHelp();
        process.exit(1);
    }
  } catch (err: any) {
    console.error(`Error: ${err?.message || err}`);
    process.exit(1);
  }
}

function printHelp() {
  console.log(`
pdf-template - Production-ready PDF template engine CLI

Usage:
  pdf-template <command> [options]

Commands:
  render <template.pdf> <data.json> [-o output.pdf]   Render template with data
  inspect <template.pdf>                              Inspect detected placeholders
  validate <template.pdf> <data.json>                 Validate variables against template

Options:
  -o, --output <file>    Output destination file (default: output.pdf)
  -v, --version          Show CLI version
  -h, --help             Show this help message
`);
}

main();
