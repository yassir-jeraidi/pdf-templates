import { performance } from "perf_hooks";
import { renderPdfTemplate, inspectTemplate } from "../dist/index.js";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function runNodeBenchmark() {
  console.log("=========================================================================================");
  console.log("                         NODE.JS / TYPESCRIPT BENCHMARK SUITE");
  console.log("=========================================================================================");
  console.log(
    `${"Scenario".padEnd(32)} | ${"Pages".padStart(8)} | ${"Total PH".padStart(8)} | ${"Latency / Doc".padStart(14)} | ${"Throughput".padStart(12)}`
  );
  console.log("-----------------------------------------------------------------------------------------");

  // 1. Invoice
  const invoiceTemplate = path.join(__dirname, "../../../examples/invoice/invoice-template.pdf");
  const invoiceData = JSON.parse(fs.readFileSync(path.join(__dirname, "../../../examples/invoice/data.json"), "utf8"));
  await benchScenario("1-Page Invoice (Typical)", invoiceTemplate, invoiceData, 100);

  // 2. Certificate
  const certTemplate = path.join(__dirname, "../../../examples/certificate/certificate-template.pdf");
  const certData = JSON.parse(fs.readFileSync(path.join(__dirname, "../../../examples/certificate/data.json"), "utf8"));
  await benchScenario("1-Page Certificate", certTemplate, certData, 100);

  // 3. Contract
  const contractTemplate = path.join(__dirname, "../../../examples/multipage/contract-template.pdf");
  const contractData = JSON.parse(fs.readFileSync(path.join(__dirname, "../../../examples/multipage/data.json"), "utf8"));
  await benchScenario("3-Page Contract Agreement", contractTemplate, contractData, 50);

  console.log("=========================================================================================");
}

async function benchScenario(name, templatePath, data, iterations) {
  const inspectRes = await inspectTemplate(templatePath);
  const totalPh = inspectRes.placeholders.length;
  const pages = inspectRes.pagesCount;

  // Warmup
  await renderPdfTemplate(templatePath, data);

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    await renderPdfTemplate(templatePath, data);
  }
  const totalMs = performance.now() - start;
  const perDocMs = totalMs / iterations;
  const docsPerSec = (iterations / (totalMs / 1000)).toFixed(1);

  console.log(
    `${name.padEnd(32)} | ${String(pages).padStart(6)} pages | ${String(totalPh).padStart(6)} ph | ${(perDocMs.toFixed(2) + " ms").padStart(14)} | ${(docsPerSec + " docs/sec").padStart(12)}`
  );
}

runNodeBenchmark().catch(console.error);
