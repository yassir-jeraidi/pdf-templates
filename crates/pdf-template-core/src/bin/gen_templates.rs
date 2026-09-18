use std::fs;
use std::path::Path;
use lopdf::{Document, Object, ObjectId, Stream, Dictionary};
use lopdf::content::{Content, Operation};

fn main() {
    println!("Generating sample PDF templates and test fixtures...");

    fs::create_dir_all("examples/invoice").unwrap();
    fs::create_dir_all("examples/certificate").unwrap();
    fs::create_dir_all("examples/multipage").unwrap();
    fs::create_dir_all("tests/fixtures").unwrap();

    generate_invoice_template("examples/invoice/invoice-template.pdf");
    generate_certificate_template("examples/certificate/certificate-template.pdf");
    generate_contract_multipage_template("examples/multipage/contract-template.pdf");

    // Test fixtures
    generate_invoice_template("tests/fixtures/invoice.pdf");
    generate_simple_template("tests/fixtures/simple.pdf");
    generate_split_spans_template("tests/fixtures/split_spans.pdf");
    generate_custom_delimiters_template("tests/fixtures/custom_delimiters.pdf");
    generate_scanned_pdf("tests/fixtures/scanned.pdf");

    println!("All sample templates generated successfully!");
}

fn create_base_doc() -> (Document, ObjectId, ObjectId, ObjectId) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_helvetica_id = doc.new_object_id();
    let font_bold_id = doc.new_object_id();

    let mut font_h = Dictionary::new();
    font_h.set("Type", Object::Name(b"Font".to_vec()));
    font_h.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_h.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
    font_h.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    doc.objects.insert(font_helvetica_id, Object::Dictionary(font_h));

    let mut font_b = Dictionary::new();
    font_b.set("Type", Object::Name(b"Font".to_vec()));
    font_b.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_b.set("BaseFont", Object::Name(b"Helvetica-Bold".to_vec()));
    font_b.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    doc.objects.insert(font_bold_id, Object::Dictionary(font_b));

    (doc, pages_id, font_helvetica_id, font_bold_id)
}

fn generate_invoice_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, font_b_id) = create_base_doc();
    let page_id = doc.new_object_id();

    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    fonts.set("F2", Object::Reference(font_b_id));
    resources.set("Font", Object::Dictionary(fonts));

    let mut ops = Vec::new();

    // 1. Dark Blue Header Background: (0.12, 0.16, 0.23)
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("rg", vec![Object::Real(0.12), Object::Real(0.16), Object::Real(0.23)]));
    ops.push(Operation::new("re", vec![Object::Real(0.0), Object::Real(760.0), Object::Real(595.0), Object::Real(82.0)]));
    ops.push(Operation::new("f", vec![]));
    ops.push(Operation::new("Q", vec![]));

    // Header Title (white text)
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(24.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(40.0), Object::Real(795.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("INVOICE")]));
    ops.push(Operation::new("ET", vec![]));

    // Company branding on top right
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(10.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.9), Object::Real(0.9), Object::Real(0.9)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(420.0), Object::Real(805.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("ACME Cloud Solutions")]));
    ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-14.0)]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Casablanca, Morocco")]));
    ops.push(Operation::new("ET", vec![]));

    // Decorative line rule
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("w", vec![Object::Real(1.5)]));
    ops.push(Operation::new("RG", vec![Object::Real(0.2), Object::Real(0.4), Object::Real(0.8)]));
    ops.push(Operation::new("m", vec![Object::Real(40.0), Object::Real(740.0)]));
    ops.push(Operation::new("l", vec![Object::Real(555.0), Object::Real(740.0)]));
    ops.push(Operation::new("S", vec![]));
    ops.push(Operation::new("Q", vec![]));

    // Customer & Invoice Details
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(12.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.1), Object::Real(0.1)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(40.0), Object::Real(710.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("BILLED TO:")]));

    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(11.0)]));
    ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Customer: {{customer.name}}")]));
    ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-18.0)]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Email:    {{customer.email}}")]));
    ops.push(Operation::new("ET", vec![]));

    // Invoice Meta on Right
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(12.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.1), Object::Real(0.1)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(350.0), Object::Real(710.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("INVOICE DETAILS:")]));

    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(11.0)]));
    ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Invoice: {{invoice.number}}")]));
    ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-18.0)]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Date:    {{invoice.date}}")]));
    ops.push(Operation::new("ET", vec![]));

    // Table Header bar
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("rg", vec![Object::Real(0.92), Object::Real(0.94), Object::Real(0.97)]));
    ops.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(610.0), Object::Real(515.0), Object::Real(26.0)]));
    ops.push(Operation::new("f", vec![]));
    ops.push(Operation::new("Q", vec![]));

    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(11.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.2), Object::Real(0.2), Object::Real(0.3)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(50.0), Object::Real(618.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Description")]));

    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(450.0), Object::Real(618.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Amount")]));
    ops.push(Operation::new("ET", vec![]));

    // Table Row 1
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(11.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.1), Object::Real(0.1)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(50.0), Object::Real(575.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Enterprise Cloud Infrastructure")]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(450.0), Object::Real(575.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("10,000.00 MAD")]));
    ops.push(Operation::new("ET", vec![]));

    // Table Row 2
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(11.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.1), Object::Real(0.1)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(50.0), Object::Real(545.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Professional Integration & Consulting")]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(450.0), Object::Real(545.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("5,000.00 MAD")]));
    ops.push(Operation::new("ET", vec![]));

    // Total Box
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("w", vec![Object::Real(1.0)]));
    ops.push(Operation::new("RG", vec![Object::Real(0.8), Object::Real(0.8), Object::Real(0.8)]));
    ops.push(Operation::new("m", vec![Object::Real(40.0), Object::Real(510.0)]));
    ops.push(Operation::new("l", vec![Object::Real(555.0), Object::Real(510.0)]));
    ops.push(Operation::new("S", vec![]));
    ops.push(Operation::new("Q", vec![]));

    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(14.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.2), Object::Real(0.6)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(300.0), Object::Real(475.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Total: {{currency(invoice.total)}}")]));
    ops.push(Operation::new("ET", vec![]));

    // Footer note
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(9.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.5), Object::Real(0.5), Object::Real(0.5)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(200.0), Object::Real(50.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Thank you for your business!")]));
    ops.push(Operation::new("ET", vec![]));

    let content = Content { operations: ops };
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), content.encode().unwrap()));

    let mut page = Dictionary::new();
    page.set("Type", Object::Name(b"Page".to_vec()));
    page.set("Parent", Object::Reference(pages_id));
    page.set("Resources", Object::Dictionary(resources));
    page.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
    page.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(page));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn generate_certificate_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, font_b_id) = create_base_doc();
    let page_id = doc.new_object_id();

    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    fonts.set("F2", Object::Reference(font_b_id));
    resources.set("Font", Object::Dictionary(fonts));

    let mut ops = Vec::new();

    // Elegant gold border
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("w", vec![Object::Real(4.0)]));
    ops.push(Operation::new("RG", vec![Object::Real(0.85), Object::Real(0.65), Object::Real(0.13)]));
    ops.push(Operation::new("re", vec![Object::Real(30.0), Object::Real(30.0), Object::Real(782.0), Object::Real(535.0)]));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("w", vec![Object::Real(1.0)]));
    ops.push(Operation::new("re", vec![Object::Real(36.0), Object::Real(36.0), Object::Real(770.0), Object::Real(523.0)]));
    ops.push(Operation::new("S", vec![]));
    ops.push(Operation::new("Q", vec![]));

    // Certificate Title
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(28.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.12), Object::Real(0.18), Object::Real(0.3)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(230.0), Object::Real(480.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("CERTIFICATE OF ACHIEVEMENT")]));
    ops.push(Operation::new("ET", vec![]));

    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(14.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.4), Object::Real(0.4), Object::Real(0.4)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(320.0), Object::Real(430.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("PROUDLY PRESENTED TO")]));
    ops.push(Operation::new("ET", vec![]));

    // Recipient Placeholder
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(24.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.1), Object::Real(0.3), Object::Real(0.7)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(280.0), Object::Real(370.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("{{recipient.name}}")]));
    ops.push(Operation::new("ET", vec![]));

    // Underline
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("w", vec![Object::Real(1.5)]));
    ops.push(Operation::new("RG", vec![Object::Real(0.85), Object::Real(0.65), Object::Real(0.13)]));
    ops.push(Operation::new("m", vec![Object::Real(220.0), Object::Real(355.0)]));
    ops.push(Operation::new("l", vec![Object::Real(622.0), Object::Real(355.0)]));
    ops.push(Operation::new("S", vec![]));
    ops.push(Operation::new("Q", vec![]));

    // Course & Description
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(13.0)]));
    ops.push(Operation::new("rg", vec![Object::Real(0.25), Object::Real(0.25), Object::Real(0.25)]));
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(200.0), Object::Real(310.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("For successfully mastering {{course.title}} with honor.")]));

    // Date & Instructor
    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(100.0), Object::Real(150.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Date: {{issue.date}}")]));

    ops.push(Operation::new("Tm", vec![
        Object::Real(1.0), Object::Real(0.0),
        Object::Real(0.0), Object::Real(1.0),
        Object::Real(550.0), Object::Real(150.0),
    ]));
    ops.push(Operation::new("Tj", vec![Object::string_literal("Instructor: {{instructor.name}}")]));
    ops.push(Operation::new("ET", vec![]));

    let content = Content { operations: ops };
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), content.encode().unwrap()));

    let mut page = Dictionary::new();
    page.set("Type", Object::Name(b"Page".to_vec()));
    page.set("Parent", Object::Reference(pages_id));
    page.set("Resources", Object::Dictionary(resources));
    page.set("MediaBox", vec![0.into(), 0.into(), 842.into(), 595.into()]);
    page.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(page));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn generate_contract_multipage_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, font_b_id) = create_base_doc();
    let mut page_ids = Vec::new();

    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    fonts.set("F2", Object::Reference(font_b_id));
    resources.set("Font", Object::Dictionary(fonts));

    // Page 1: Overview
    {
        let page_id = doc.new_object_id();
        let ops = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(20.0)]),
            Operation::new("Tm", vec![
                Object::Real(1.0), Object::Real(0.0),
                Object::Real(0.0), Object::Real(1.0),
                Object::Real(50.0), Object::Real(760.0),
            ]),
            Operation::new("Tj", vec![Object::string_literal("MASTER SERVICES AGREEMENT")]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-40.0)]),
            Operation::new("Tj", vec![Object::string_literal("This agreement is entered into between:")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-25.0)]),
            Operation::new("Tj", vec![Object::string_literal("Provider: {{provider.name}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]),
            Operation::new("Tj", vec![Object::string_literal("Client:   {{client.name}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-30.0)]),
            Operation::new("Tj", vec![Object::string_literal("Effective Date: {{contract.effective_date}}")]),
            Operation::new("ET", vec![]),
        ];
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));
        let mut p = Dictionary::new();
        p.set("Type", Object::Name(b"Page".to_vec()));
        p.set("Parent", Object::Reference(pages_id));
        p.set("Resources", Object::Dictionary(resources.clone()));
        p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
        p.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(p));
        page_ids.push(page_id);
    }

    // Page 2: Scope
    {
        let page_id = doc.new_object_id();
        let ops = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(18.0)]),
            Operation::new("Tm", vec![
                Object::Real(1.0), Object::Real(0.0),
                Object::Real(0.0), Object::Real(1.0),
                Object::Real(50.0), Object::Real(760.0),
            ]),
            Operation::new("Tj", vec![Object::string_literal("SCHEDULE A: SCOPE OF SERVICES")]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-35.0)]),
            Operation::new("Tj", vec![Object::string_literal("Project Name: {{project.name}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-25.0)]),
            Operation::new("Tj", vec![Object::string_literal("Milestone 1: {{project.milestone_1}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]),
            Operation::new("Tj", vec![Object::string_literal("Milestone 2: {{project.milestone_2}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-30.0)]),
            Operation::new("Tj", vec![Object::string_literal("Payment Terms: {{contract.payment_terms}}")]),
            Operation::new("ET", vec![]),
        ];
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));
        let mut p = Dictionary::new();
        p.set("Type", Object::Name(b"Page".to_vec()));
        p.set("Parent", Object::Reference(pages_id));
        p.set("Resources", Object::Dictionary(resources.clone()));
        p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
        p.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(p));
        page_ids.push(page_id);
    }

    // Page 3: Execution & Signatures
    {
        let page_id = doc.new_object_id();
        let ops = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(18.0)]),
            Operation::new("Tm", vec![
                Object::Real(1.0), Object::Real(0.0),
                Object::Real(0.0), Object::Real(1.0),
                Object::Real(50.0), Object::Real(760.0),
            ]),
            Operation::new("Tj", vec![Object::string_literal("EXECUTION & SIGNATURES")]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-40.0)]),
            Operation::new("Tj", vec![Object::string_literal("IN WITNESS WHEREOF, the parties execute this agreement on:")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-20.0)]),
            Operation::new("Tj", vec![Object::string_literal("Execution Date: {{contract.signed_date}}")]),
            Operation::new("Td", vec![Object::Real(0.0), Object::Real(-60.0)]),
            Operation::new("Tj", vec![Object::string_literal("Provider Signatory: {{provider.signatory}}")]),
            Operation::new("Td", vec![Object::Real(250.0), Object::Real(0.0)]),
            Operation::new("Tj", vec![Object::string_literal("Client Signatory: {{client.signatory}}")]),
            Operation::new("ET", vec![]),
        ];
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));
        let mut p = Dictionary::new();
        p.set("Type", Object::Name(b"Page".to_vec()));
        p.set("Parent", Object::Reference(pages_id));
        p.set("Resources", Object::Dictionary(resources));
        p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
        p.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(p));
        page_ids.push(page_id);
    }

    finish_and_save_doc(doc, pages_id, page_ids, out_path);
}

fn generate_simple_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, _) = create_base_doc();
    let page_id = doc.new_object_id();
    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    resources.set("Font", Object::Dictionary(fonts));

    let ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
        Operation::new("Tm", vec![
            Object::Real(1.0), Object::Real(0.0),
            Object::Real(0.0), Object::Real(1.0),
            Object::Real(100.0), Object::Real(700.0),
        ]),
        Operation::new("Tj", vec![Object::string_literal("Hello, {{name}}! Welcome to {{place}}!")]),
        Operation::new("ET", vec![]),
    ];
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));

    let mut p = Dictionary::new();
    p.set("Type", Object::Name(b"Page".to_vec()));
    p.set("Parent", Object::Reference(pages_id));
    p.set("Resources", Object::Dictionary(resources));
    p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
    p.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(p));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn generate_split_spans_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, _) = create_base_doc();
    let page_id = doc.new_object_id();
    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    resources.set("Font", Object::Dictionary(fonts));

    let ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
        Operation::new("Tm", vec![
            Object::Real(1.0), Object::Real(0.0),
            Object::Real(0.0), Object::Real(1.0),
            Object::Real(100.0), Object::Real(700.0),
        ]),
        Operation::new("TJ", vec![Object::Array(vec![
            Object::string_literal("Customer: {{cust"),
            Object::Real(-5.0),
            Object::string_literal("omer."),
            Object::Real(10.0),
            Object::string_literal("name}}"),
        ])]),
        Operation::new("ET", vec![]),
    ];
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));

    let mut p = Dictionary::new();
    p.set("Type", Object::Name(b"Page".to_vec()));
    p.set("Parent", Object::Reference(pages_id));
    p.set("Resources", Object::Dictionary(resources));
    p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
    p.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(p));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn generate_custom_delimiters_template(out_path: &str) {
    let (mut doc, pages_id, font_h_id, _) = create_base_doc();
    let page_id = doc.new_object_id();
    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    resources.set("Font", Object::Dictionary(fonts));

    let ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(12.0)]),
        Operation::new("Tm", vec![
            Object::Real(1.0), Object::Real(0.0),
            Object::Real(0.0), Object::Real(1.0),
            Object::Real(100.0), Object::Real(700.0),
        ]),
        Operation::new("Tj", vec![Object::string_literal("Item: [[item.code]] - [[item.title]]")]),
        Operation::new("ET", vec![]),
    ];
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));

    let mut p = Dictionary::new();
    p.set("Type", Object::Name(b"Page".to_vec()));
    p.set("Parent", Object::Reference(pages_id));
    p.set("Resources", Object::Dictionary(resources));
    p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
    p.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(p));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn generate_scanned_pdf(out_path: &str) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let page_id = doc.new_object_id();

    // Only vector shape simulating scanned page, no text BT/ET
    let ops = vec![
        Operation::new("re", vec![Object::Real(10.0), Object::Real(10.0), Object::Real(500.0), Object::Real(700.0)]),
        Operation::new("f", vec![]),
    ];
    let stream_id = doc.add_object(Stream::new(Dictionary::new(), Content { operations: ops }.encode().unwrap()));

    let mut p = Dictionary::new();
    p.set("Type", Object::Name(b"Page".to_vec()));
    p.set("Parent", Object::Reference(pages_id));
    p.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
    p.set("Contents", Object::Reference(stream_id));
    doc.objects.insert(page_id, Object::Dictionary(p));

    finish_and_save_doc(doc, pages_id, vec![page_id], out_path);
}

fn finish_and_save_doc(
    mut doc: Document,
    pages_id: ObjectId,
    page_ids: Vec<ObjectId>,
    out_path: &str,
) {
    let mut pages = Dictionary::new();
    pages.set("Type", Object::Name(b"Pages".to_vec()));
    let count = page_ids.len();
    pages.set(
        "Kids",
        page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
    );
    pages.set("Count", Object::Integer(count as i64));
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let p = Path::new(out_path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = fs::File::create(out_path).unwrap();
    doc.save_to(&mut file).unwrap();
    println!("  Generated: {}", out_path);
}
