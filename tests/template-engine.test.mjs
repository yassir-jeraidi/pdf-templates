import { describe, it } from "node:test";
import assert from "node:assert/strict";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import {
  renderPdfTemplate,
  inspectTemplate,
  validateTemplate,
  MissingVariableError,
  OverflowError,
  UnsupportedPdfError,
  PdfTemplateError,
} from "../packages/node/dist/index.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixturesDir = path.join(__dirname, "fixtures");

describe("PDF Template Engine Test Suite", () => {

  describe("1. Placeholder Inspection", () => {
    it("detects all placeholders with bounding boxes and page numbers", async () => {
      const templatePath = path.join(fixturesDir, "invoice.pdf");
      const result = await inspectTemplate(templatePath);

      assert.equal(result.pagesCount, 1);
      assert.equal(result.hasTextLayer, true);
      assert.equal(result.placeholders.length, 5);

      const expressions = result.placeholders.map((p) => p.expression);
      assert.ok(expressions.includes("customer.name"));
      assert.ok(expressions.includes("customer.email"));
      assert.ok(expressions.includes("invoice.number"));
      assert.ok(expressions.includes("invoice.date"));
      assert.ok(expressions.includes("currency(invoice.total)"));

      // Check coordinates validity
      for (const ph of result.placeholders) {
        assert.ok(ph.x > 0, `X must be > 0, got ${ph.x}`);
        assert.ok(ph.y > 0, `Y must be > 0, got ${ph.y}`);
        assert.ok(ph.width > 0, `Width must be > 0, got ${ph.width}`);
        assert.ok(ph.height > 0, `Height must be > 0, got ${ph.height}`);
      }
    });

    it("supports custom delimiters for inspection", async () => {
      const templatePath = path.join(fixturesDir, "custom_delimiters.pdf");
      const result = await inspectTemplate(templatePath, {
        delimiters: ["[[", "]]"],
      });

      assert.equal(result.placeholders.length, 2);
      assert.equal(result.placeholders[0].expression, "item.code");
      assert.equal(result.placeholders[1].expression, "item.title");
    });
  });

  describe("2. Variable Replacement & Preservation", () => {
    it("renders simple placeholders and completely removes original placeholder tokens", async () => {
      const templatePath = path.join(fixturesDir, "simple.pdf");
      const outPath = path.join(__dirname, "tmp_simple_output.pdf");

      const rendered = await renderPdfTemplate(templatePath, {
        name: "Yassir",
        place: "Casablanca",
      });

      await rendered.save(outPath);
      assert.ok(fs.existsSync(outPath));

      const raw = fs.readFileSync(outPath, "utf8");
      assert.ok(raw.includes("Yassir"), "Must contain replacement 'Yassir'");
      assert.ok(raw.includes("Casablanca"), "Must contain replacement 'Casablanca'");
      assert.ok(!raw.includes("{{name}}"), "Must NOT contain '{{name}}'");
      assert.ok(!raw.includes("{{place}}"), "Must NOT contain '{{place}}'");

      // Re-inspect output: 0 placeholders must remain
      const postInspect = await inspectTemplate(outPath);
      assert.equal(postInspect.placeholders.length, 0);

      fs.unlinkSync(outPath);
    });

    it("handles placeholders split across multiple text runs and kerning displacements (TJ)", async () => {
      const templatePath = path.join(fixturesDir, "split_spans.pdf");
      const outPath = path.join(__dirname, "tmp_split_output.pdf");

      const rendered = await renderPdfTemplate(templatePath, {
        customer: {
          name: "Dr. Alan Turing",
        },
      });

      await rendered.save(outPath);
      const raw = fs.readFileSync(outPath, "utf8");

      assert.ok(raw.includes("Dr. Alan Turing"));
      assert.ok(!raw.includes("customer.name"));

      fs.unlinkSync(outPath);
    });

    it("supports buffer input and returns buffer and uint8array outputs", async () => {
      const templateBuf = fs.readFileSync(path.join(fixturesDir, "simple.pdf"));
      const rendered = await renderPdfTemplate({
        template: templateBuf,
        data: { name: "Alice", place: "Wonderland" },
      });

      const buf = await rendered.toBuffer();
      const syncBuf = rendered.toBufferSync();
      const u8 = rendered.toUint8Array();

      assert.ok(Buffer.isBuffer(buf));
      assert.ok(Buffer.isBuffer(syncBuf));
      assert.ok(u8 instanceof Uint8Array);
      assert.equal(buf.length, syncBuf.length);
      assert.equal(buf.length, u8.length);
    });
  });

  describe("3. Expressions & Custom Helpers", () => {
    it("evaluates custom registered JavaScript helpers safely without eval()", async () => {
      const templatePath = path.join(fixturesDir, "invoice.pdf");
      const outPath = path.join(__dirname, "tmp_helpers_output.pdf");

      const rendered = await renderPdfTemplate(templatePath, {
        customer: {
          name: "Yassir Jeraidi",
          email: "yassir@example.com",
        },
        invoice: {
          number: "INV-999",
          date: "18/09/2026",
          total: 25000,
        },
      }, {
        helpers: {
          currency: (val) => `${val.toLocaleString("en-US", { minimumFractionDigits: 2 })} MAD`,
        },
      });

      await rendered.save(outPath);
      const raw = fs.readFileSync(outPath, "utf8");

      assert.ok(raw.includes("25,000.00 MAD"));
      assert.ok(raw.includes("Yassir Jeraidi"));
      assert.ok(raw.includes("INV-999"));

      fs.unlinkSync(outPath);
    });

    it("supports custom delimiters in rendering", async () => {
      const templatePath = path.join(fixturesDir, "custom_delimiters.pdf");
      const outPath = path.join(__dirname, "tmp_custom_delim_output.pdf");

      const rendered = await renderPdfTemplate(templatePath, {
        item: {
          code: "SKU-440",
          title: "Mechanical Keyboard",
        },
      }, {
        delimiters: ["[[", "]]"],
      });

      await rendered.save(outPath);
      const raw = fs.readFileSync(outPath, "utf8");

      assert.ok(raw.includes("SKU-440"));
      assert.ok(raw.includes("Mechanical Keyboard"));
      assert.ok(!raw.includes("[[item.code]]"));

      fs.unlinkSync(outPath);
    });
  });

  describe("4. Error Handling & Validation", () => {
    it("throws MissingVariableError with structured details when a variable is absent", async () => {
      const templatePath = path.join(fixturesDir, "invoice.pdf");

      try {
        await renderPdfTemplate(templatePath, {
          customer: { name: "Yassir" },
          // missing email, invoice.number, etc.
        });
        assert.fail("Should have thrown MissingVariableError");
      } catch (err) {
        assert.ok(err instanceof MissingVariableError);
        assert.equal(typeof err.expression, "string");
        assert.equal(typeof err.page, "number");
        assert.ok(err.message.includes("was not found"));
      }
    });

    it("validates template and returns diagnostic error objects without throwing", async () => {
      const templatePath = path.join(fixturesDir, "invoice.pdf");
      const validation = await validateTemplate(templatePath, {
        customer: { name: "Yassir" },
      });

      assert.equal(validation.valid, false);
      assert.ok(validation.errors.length >= 1);
      const missingVarErrors = validation.errors.filter((e) => e.type === "MISSING_VARIABLE");
      assert.ok(missingVarErrors.length >= 1);
    });

    it("handles text overflow with shrink policy without error", async () => {
      const templatePath = path.join(fixturesDir, "simple.pdf");
      const rendered = await renderPdfTemplate(templatePath, {
        name: "Alexandru Constantin Dimitrescu von Hohenzollern-Sigmaringen",
        place: "A Very Long Place Name That Should Scale Down",
      }, {
        overflow: "shrink",
        minFontSize: 4,
      });

      const buf = await rendered.toBuffer();
      assert.ok(buf.length > 0);
    });

    it("throws OverflowError when overflow is set to error and text is too long", async () => {
      const templatePath = path.join(fixturesDir, "simple.pdf");

      try {
        await renderPdfTemplate(templatePath, {
          name: "Alexandru Constantin Dimitrescu von Hohenzollern-Sigmaringen The Third",
          place: "Everywhere",
        }, {
          overflow: "error",
        });
        assert.fail("Should have thrown OverflowError");
      } catch (err) {
        assert.ok(err instanceof OverflowError);
      }
    });

    it("detects scanned image-only PDFs and throws UnsupportedPdfError", async () => {
      const templatePath = path.join(fixturesDir, "scanned.pdf");

      try {
        await renderPdfTemplate(templatePath, { any: "data" });
        assert.fail("Should have thrown UnsupportedPdfError");
      } catch (err) {
        assert.ok(err instanceof UnsupportedPdfError);
        assert.ok(err.message.includes("OCR is required for scanned PDFs"));
      }
    });
  });

  describe("5. Multi-Page Document Support", () => {
    it("renders across multi-page documents while keeping page count and untouched pages intact", async () => {
      const templatePath = path.join(__dirname, "../examples/multipage/contract-template.pdf");
      const outPath = path.join(__dirname, "tmp_contract_output.pdf");

      const initialInspect = await inspectTemplate(templatePath);
      assert.equal(initialInspect.pagesCount, 3);
      assert.equal(initialInspect.placeholders.length, 10);

      const rendered = await renderPdfTemplate(templatePath, {
        provider: {
          name: "ACME Cloud Solutions",
          signatory: "Yassir Jeraidi",
        },
        client: {
          name: "Global Enterprise",
          signatory: "Sarah Connor",
        },
        contract: {
          effective_date: "01/10/2026",
          payment_terms: "Net 30 Days",
          signed_date: "18/09/2026",
        },
        project: {
          name: "Core Platform",
          milestone_1: "Alpha Milestone",
          milestone_2: "Beta Milestone",
        },
      });

      await rendered.save(outPath);

      // Verify page count remains 3 and all placeholders were replaced
      const postInspect = await inspectTemplate(outPath);
      assert.equal(postInspect.pagesCount, 3);
      assert.equal(postInspect.placeholders.length, 0);

      const raw = fs.readFileSync(outPath, "utf8");
      assert.ok(raw.includes("ACME Cloud Solutions"));
      assert.ok(raw.includes("Global Enterprise"));
      assert.ok(raw.includes("Alpha Milestone"));
      assert.ok(raw.includes("Sarah Connor"));

      fs.unlinkSync(outPath);
    });
  });

});
