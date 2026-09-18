# pdf-template-engine

> A high-performance, production-ready PDF template engine for Node.js and TypeScript, powered by a native Rust core.

Like `docx-templates`, but for **PDFs**. Create your document visually in any software (Figma, Microsoft Word, LibreOffice, Adobe Acrobat, InDesign), insert placeholders like `{{customer.name}}`, export to PDF, and let **pdf-template-engine** render dynamic variables directly into the PDF while **100% preserving vector graphics, backgrounds, fonts, metadata, images, and layout**.

```
Existing PDF (+ Placeholders) + JSON/JS Data ➔ Pixel-Preserved Output PDF
```

---

## Key Highlights

* **The PDF is the Template:** No HTML/CSS conversions, no Headless Chrome/Puppeteer bloat, no rasterization, no OCR overhead.
* **Native Rust Performance:** Text extraction, coordinate detection, and content stream surgery run in compiled Rust via `napi-rs` (`> 1,500 docs/sec` per core).
* **True Content Stream Surgery:** Placeholders are cleanly excised from PDF content stream operators (`Tj` and `TJ`), not painted over or hidden with hacky covers.
* **Preserves Surrounding Design:** Vectors, lines, shapes, transparent elements, and embedded images remain pixel-perfect.
* **Safe Expression Engine:** Evaluates nested properties (`customer.address.city`), array items (`items[0].price`), and registered helper functions without unrestricted `eval()`.
* **Smart Text Overflow Handling:** Automatic font scaling (`shrink`), clipping (`clip`), multi-line wrapping (`wrap`), or strict exceptions (`error`).
* **Multi-Page Native:** Process multi-page agreements, invoices, certificates, and reports across hundreds of pages effortlessly.
* **CLI Included:** Inspect, validate, and render directly from the command line with `npx pdf-template`.

---

## Installation

```bash
npm install pdf-template-engine
```

*Pre-requisites for local compilation from source:* Rust toolchain (`cargo`, `rustc`) and Node.js `>= 18`.

---

## Quick Start

### Basic Usage

```typescript
import { renderPdfTemplate } from "pdf-template-engine";

const output = await renderPdfTemplate("./invoice-template.pdf", {
  customer: {
    name: "Yassir Jeraidi",
    email: "yassir@example.com",
    address: {
      city: "Casablanca",
      country: "Morocco",
    },
  },
  invoice: {
    number: "INV-2026-001",
    date: "18/09/2026",
    total: 15000,
  },
  items: [
    { name: "Cloud Infrastructure", price: 10000 },
    { name: "Consulting", price: 5000 },
  ],
});

// Save to disk
await output.save("./invoice.pdf");

// Or retrieve binary data
const buffer = await output.toBuffer();
const bytes = output.toUint8Array();
```

### Config Object Style

```typescript
const pdf = await renderPdfTemplate({
  template: "./invoice-template.pdf", // Path, Buffer, or Uint8Array
  data: {
    customer: { name: "Yassir Jeraidi" },
  },
  overflow: "shrink",
  minFontSize: 6,
});

await pdf.save("./output.pdf");
```

---

## Template Creation Guidelines

1. **Design visually:** Create your template in Figma, Canva, Microsoft Word, Google Docs, LibreOffice, or Adobe InDesign.
2. **Add placeholders:** Wherever dynamic content should appear, type:
   ```text
   {{customer.name}}
   {{invoice.total}}
   {{items[0].description}}
   ```
3. **Export directly to PDF:** Save or export as `.pdf`.
4. **Pass to the engine:** That's it! The engine finds the bounding box, detects the exact baseline and font metrics, excises the placeholder characters, and draws the replacement value.

---

## Supported Expression Syntax

### Nested Properties & Array Access
```text
{{customer.name}}
{{customer.address.city}}
{{items[0].name}}
{{items[0].price}}
```

### Whitespace Tolerance
Whitespace inside delimiters is automatically normalized:
```text
{{ customer.name }}      // Equivalent to {{customer.name}}
{{   invoice.total   }}  // Equivalent to {{invoice.total}}
```

### Custom Delimiters
If your document design or text contains double braces, configure custom delimiters:
```typescript
await renderPdfTemplate("./template.pdf", data, {
  delimiters: ["[[", "]]"],
});
```
Template:
```text
Item code: [[item.code]]
```

### Registered Helpers
Format variables with safe registered JavaScript helpers:
```typescript
await renderPdfTemplate("./template.pdf", data, {
  helpers: {
    currency: (value: number) => `${value.toFixed(2)} MAD`,
    formatDate: (val: string) => new Date(val).toLocaleDateString("en-US"),
  },
});
```
Template:
```text
Total: {{currency(invoice.total)}}
Due:   {{formatDate(invoice.dueDate)}}
```

Built-in helpers available out of the box:
* `currency(val)`: Formats number to 2 decimal places with `MAD` suffix.
* `uppercase(val)`: Transforms string to uppercase.
* `lowercase(val)`: Transforms string to lowercase.
* `trim(val)`: Trims leading and trailing whitespace.

---

## Text Overflow Policies

When the replacement value is wider than the original placeholder bounding box, configure how the engine handles the overflow:

```typescript
renderPdfTemplate(template, data, {
  overflow: "shrink", // "shrink" | "clip" | "wrap" | "error"
  minFontSize: 6,     // Minimum font size for "shrink" (default: 6)
});
```

* **`shrink` (default):** Proportionally scales down the font size so the text fits within the placeholder's original bounding box. Clamps to `minFontSize`.
* **`clip`:** Keeps the font size and clips text at the bounding box boundaries using PDF graphics clipping paths (`re W n`).
* **`wrap`:** Breaks the string across multiple lines if vertical space permits.
* **`error`:** Throws an `OverflowError` specifying the required width, available width, and font size.

---

## Inspecting & Validating Templates

### `inspectTemplate(template, options?)`
Discovers all placeholders in a template without rendering:

```typescript
import { inspectTemplate } from "pdf-template-engine";

const result = await inspectTemplate("./invoice-template.pdf");

console.log(`Pages: ${result.pagesCount}`);
console.log(`Has Text Layer: ${result.hasTextLayer}`);
console.log(result.placeholders);
```

Output:
```json
[
  {
    "expression": "customer.name",
    "raw": "{{customer.name}}",
    "page": 0,
    "x": 93.8,
    "y": 690.0,
    "width": 90.5,
    "height": 11.0,
    "font": "F1",
    "fontSize": 11.0,
    "rotation": 0.0,
    "color": [0.1, 0.1, 0.1]
  }
]
```

### `validateTemplate(template, data, options?)`
Performs dry-run validation against your dataset before generating PDFs:

```typescript
import { validateTemplate } from "pdf-template-engine";

const result = await validateTemplate("./invoice-template.pdf", data);

if (!result.valid) {
  console.error("Validation errors found:", result.errors);
}
```

Diagnostics output:
```json
{
  "valid": false,
  "errors": [
    {
      "type": "MISSING_VARIABLE",
      "message": "Variable \"customer.email\" was not found",
      "expression": "customer.email",
      "page": 0
    }
  ]
}
```

---

## Command Line Interface (CLI)

The package includes a command-line tool for continuous integration, scripts, and debugging:

```bash
# Render a template
npx pdf-template render invoice-template.pdf data.json -o invoice.pdf

# Inspect detected placeholders
npx pdf-template inspect invoice-template.pdf

# Validate template against data
npx pdf-template validate invoice-template.pdf data.json
```

---

## Architecture & Technology Decision

```
┌─────────────────────────────────────────────────────────┐
│                 TypeScript / Node.js API                │
│         renderPdfTemplate | inspectTemplate             │
└────────────────────────────┬────────────────────────────┘
                             │ NAPI-RS (Zero-Copy)
┌────────────────────────────▼────────────────────────────┐
│                    Rust Core Engine                     │
│  ┌───────────────────────┐   ┌────────────────────────┐ │
│  │   PageParser (lopdf)  │   │  SpanMatcher (Cluster) │ │
│  └───────────┬───────────┘   └───────────┬────────────┘ │
│              │                           │              │
│  ┌───────────▼───────────┐   ┌───────────▼────────────┐ │
│  │     LayoutEngine      │   │     StreamRewriter     │ │
│  │ (Font Metrics/Scaling)│   │ (Content Stream Surgery│ │
│  └───────────────────────┘   └────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

1. **Why `lopdf` was selected:**
   * **100% Pure Rust:** Compiles to a self-contained native binary without C/C++ dynamic shared library dependencies (unlike `pdfium` which requires distributing platform `.dylib`/`.so`/`.dll` runtime binaries).
   * **Direct Content Stream Surgery:** Decodes content stream operations (`BT`, `Tf`, `Tm`, `Td`, `Tj`, `TJ`, `cm`, `ET`), surgically slices placeholder text tokens, and injects replacement operations while keeping all other objects, vectors, colors, and image streams untouched.
2. **Text Coordinate Extraction:**
   Tracks graphics state transforms ($CTM$) and text matrices ($T_m$) across affine matrix multiplications to compute exact page coordinates $(x, y)$ and character advances.
3. **Split Span Detection:**
   PDF generators routinely split text across operators or inside `TJ` arrays with kerning offsets (e.g. `[(Customer: {{cust) -5 (omer.name}})] TJ`). Our engine groups text runs into visual lines using baseline proximity and monotonic X progression, mapping character offsets back to exact operator slices.

---

## Benchmarks

Measured on macOS (Apple Silicon M-series):

### Rust Core (`cargo run --release --bin benchmark`)
| Scenario | Pages | Total Placeholders | Latency / Doc | Throughput |
| :--- | :---: | :---: | :---: | :---: |
| **1-Page Invoice (Typical)** | 1 | 5 | **246.8 µs** | **4,051 docs/sec** |
| **10-Page Document** | 10 | 50 | **1.06 ms** | **947 docs/sec** |
| **100-Page Large Document** | 100 | 500 | **7.64 ms** | **131 docs/sec** |
| **1-Page High Density** | 1 | 100 | **1.27 ms** | **786 docs/sec** |
| **10-Page Ultra Density** | 10 | 1,000 | **11.36 ms** | **88 docs/sec** |

### Node.js / TypeScript API (`npm run test:node`)
| Scenario | Pages | Total Placeholders | Latency / Doc | Throughput |
| :--- | :---: | :---: | :---: | :---: |
| **1-Page Invoice** | 1 | 5 | **0.67 ms** | **1,486 docs/sec** |
| **1-Page Certificate** | 1 | 4 | **0.52 ms** | **1,915 docs/sec** |
| **3-Page Agreement** | 3 | 10 | **0.72 ms** | **1,390 docs/sec** |

---

## Honest Limitations (v1 MVP)

* **Complex RTL & Arabic Ligature Shaping:** Basic UTF-8 and French accents are supported. Advanced bidirectional text and cursive ligature shaping (e.g. Arabic with HarfBuzz) require custom OpenType shaper integration planned for v2.
* **Embedded Font Subsetting:** If a template font was heavily subsetted during export and lacks glyphs required by the new value, the engine seamlessly uses standard fallback fonts (`Helvetica`, `Times`, `Courier`).
* **Scanned / Image-Only PDFs:** PDFs without a selectable text layer cannot be matched and immediately throw `UnsupportedPdfError: Template contains no selectable text. OCR is required for scanned PDFs.`
* **Dynamic Table Expansion:** In-place replacement at the exact bounding box is supported in v1. Block reflow and repeating table regions are planned for v2 (AST `TemplateRegion` structures are already incorporated).

---

## Project Structure

```
pdf-template-engine/
├── crates/
│   ├── pdf-template-core/         # Pure Rust PDF parsing, detection & rendering
│   │   ├── src/
│   │   │   ├── parser/            # Content stream, font metrics & encoding
│   │   │   ├── detector/          # Line clustering & split span matcher
│   │   │   ├── expression/        # Safe AST expression parser & evaluator
│   │   │   ├── layout/            # Font scaling & overflow policies
│   │   │   ├── renderer/          # Surgical stream rewriter & injector
│   │   │   ├── bin/               # Template generator & benchmark binaries
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── pdf-template-napi/         # High-performance NAPI-RS Node binding
│       ├── src/lib.rs
│       └── Cargo.toml
├── packages/
│   └── node/                      # User-facing npm package
│       ├── src/
│       │   ├── index.ts           # Primary TypeScript API
│       │   ├── types.ts           # Type definitions
│       │   ├── errors.ts          # Structured error classes
│       │   ├── native.ts          # Native binary loader
│       │   └── cli.ts             # CLI implementation
│       ├── bin/cli.js             # Executable CLI wrapper
│       └── package.json
├── examples/
│   ├── invoice/                   # Sample invoice template, data & runner
│   ├── certificate/               # Sample certificate template & runner
│   └── multipage/                 # Sample 3-page contract agreement
├── tests/
│   ├── fixtures/                  # Real PDF test fixtures
│   ├── template-engine.test.mjs   # Comprehensive integration tests
│   └── visual-regression.test.mjs # Visual geometry & layout regression tests
├── Cargo.toml                     # Workspace configuration
├── package.json                   # Root monorepo configuration
├── LICENSE                        # MIT License
└── README.md
```

---

## Building from Clean Checkout

```bash
# 1. Clone repository
git clone https://github.com/yassirjr/pdf-template-engine.git
cd pdf-template-engine

# 2. Install dependencies
npm install

# 3. Build native Rust addon and TypeScript package
npm run build:rust
npm run build

# 4. Run test suite (Rust unit tests + TypeScript integration + visual regression)
npm test
```

---

## Publishing to npm

```bash
# 1. Build optimized release binary
npm run build:rust

# 2. Compile TypeScript distribution
npm run build

# 3. Publish from packages/node
cd packages/node
npm publish --access public
```

---

## License

MIT © [Yassir Jeraidi](https://github.com/yassirjr)
