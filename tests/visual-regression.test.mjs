import { describe, it } from "node:test";
import assert from "node:assert/strict";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import * as pdfjs from "pdfjs-dist/legacy/build/pdf.mjs";
import { renderPdfTemplate } from "../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixturesDir = path.join(__dirname, "fixtures");

describe("Visual & Layout Regression Tests", () => {
  it("preserves exact page geometry and dimensions (MediaBox & Viewport)", async () => {
    const templatePath = path.join(fixturesDir, "invoice.pdf");
    const data = {
      customer: { name: "Yassir Jeraidi", email: "yassir@example.com" },
      invoice: { number: "INV-2026-001", date: "18/09/2026", total: 15000 },
    };

    const rendered = await renderPdfTemplate(templatePath, data);
    const renderedBytes = rendered.toUint8Array();

    const originalDoc = await pdfjs.getDocument({ data: new Uint8Array(fs.readFileSync(templatePath)) }).promise;
    const renderedDoc = await pdfjs.getDocument({ data: renderedBytes }).promise;

    assert.equal(renderedDoc.numPages, originalDoc.numPages, "Page count must match");

    const origPage = await originalDoc.getPage(1);
    const rendPage = await renderedDoc.getPage(1);

    const origViewport = origPage.getViewport({ scale: 1.0 });
    const rendViewport = rendPage.getViewport({ scale: 1.0 });

    assert.equal(rendViewport.width, origViewport.width, "Page width must be strictly preserved");
    assert.equal(rendViewport.height, origViewport.height, "Page height must be strictly preserved");
  });

  it("verifies unchanged text labels remain at identical visual coordinates", async () => {
    const templatePath = path.join(fixturesDir, "invoice.pdf");
    const data = {
      customer: { name: "Yassir Jeraidi", email: "yassir@example.com" },
      invoice: { number: "INV-2026-001", date: "18/09/2026", total: 15000 },
    };

    const rendered = await renderPdfTemplate(templatePath, data);
    const renderedBytes = rendered.toUint8Array();

    const origDoc = await pdfjs.getDocument({ data: new Uint8Array(fs.readFileSync(templatePath)) }).promise;
    const rendDoc = await pdfjs.getDocument({ data: renderedBytes }).promise;

    const origPage = await origDoc.getPage(1);
    const rendPage = await rendDoc.getPage(1);

    const origContent = await origPage.getTextContent();
    const rendContent = await rendPage.getTextContent();

    // Helper to find item by text substring
    const findItem = (items, text) => items.find((it) => it.str.includes(text));

    const origTitle = findItem(origContent.items, "INVOICE");
    const rendTitle = findItem(rendContent.items, "INVOICE");

    assert.ok(origTitle && rendTitle, "Title 'INVOICE' must exist in both");
    assert.equal(rendTitle.transform[4], origTitle.transform[4], "Title X must not change");
    assert.equal(rendTitle.transform[5], origTitle.transform[5], "Title Y must not change");

    const origBilled = findItem(origContent.items, "BILLED TO:");
    const rendBilled = findItem(rendContent.items, "BILLED TO:");
    assert.ok(origBilled && rendBilled);
    assert.equal(rendBilled.transform[4], origBilled.transform[4], "Billed To X must not change");
    assert.equal(rendBilled.transform[5], origBilled.transform[5], "Billed To Y must not change");
  });

  it("verifies replaced values are positioned correctly on the baseline", async () => {
    const templatePath = path.join(fixturesDir, "invoice.pdf");
    const data = {
      customer: { name: "Yassir Jeraidi", email: "yassir@example.com" },
      invoice: { number: "INV-2026-001", date: "18/09/2026", total: 15000 },
    };

    const rendered = await renderPdfTemplate(templatePath, data);
    const rendDoc = await pdfjs.getDocument({ data: rendered.toUint8Array() }).promise;
    const rendPage = await rendDoc.getPage(1);
    const rendContent = await rendPage.getTextContent();

    const customerItem = rendContent.items.find((it) => it.str.includes("Yassir Jeraidi"));
    assert.ok(customerItem, "Replacement 'Yassir Jeraidi' must exist in rendered text layer");
    assert.ok(customerItem.transform[4] > 0, "X position must be valid");
    assert.ok(customerItem.transform[5] > 0, "Y position must be valid");

    // The old placeholder must NOT exist in the text content
    const oldPlaceholder = rendContent.items.find((it) => it.str.includes("{{customer.name}}"));
    assert.equal(oldPlaceholder, undefined, "Old placeholder must be completely removed");
  });

  it("verifies vector graphics operators (borders, headers) are preserved", async () => {
    const templatePath = path.join(fixturesDir, "invoice.pdf");
    const data = {
      customer: { name: "Yassir Jeraidi", email: "yassir@example.com" },
      invoice: { number: "INV-2026-001", date: "18/09/2026", total: 15000 },
    };

    const rendered = await renderPdfTemplate(templatePath, data);
    const origDoc = await pdfjs.getDocument({ data: new Uint8Array(fs.readFileSync(templatePath)) }).promise;
    const rendDoc = await pdfjs.getDocument({ data: rendered.toUint8Array() }).promise;

    const origPage = await origDoc.getPage(1);
    const rendPage = await rendDoc.getPage(1);

    const origOps = await origPage.getOperatorList();
    const rendOps = await rendPage.getOperatorList();

    assert.ok(origOps.fnArray.length > 0);
    assert.ok(rendOps.fnArray.length >= origOps.fnArray.length, "Rendered PDF must preserve graphics operators");
  });
});
