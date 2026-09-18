use lopdf::{Document, Object, ObjectId, Stream, Dictionary};
use lopdf::content::{Content, Operation};

fn main() {
    println!("Generating high-impact 2-Page Enterprise Template...");
    let out_path1 = "/Users/yassirjr/projects/pdf-engine/examples/enterprise-order/template.pdf";
    let out_path2 = "/Users/yassirjr/projects/demo-pdf-engine/enterprise-template.pdf";

    let (mut doc, pages_id, font_h_id, font_b_id, font_ob_id) = create_base_doc();
    let page1_id = doc.new_object_id();
    let page2_id = doc.new_object_id();

    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_h_id));
    fonts.set("F2", Object::Reference(font_b_id));
    fonts.set("F3", Object::Reference(font_ob_id));
    resources.set("Font", Object::Dictionary(fonts));

    // =========================================================================
    // PAGE 1: HEADER, METADATA, DYNAMIC ITEMIZED TABLE (1-10 ROWS), FINANCIALS
    // =========================================================================
    let mut ops1: Vec<Operation> = Vec::new();

    // 1. Top Header Banner (Y: 742 to 842)
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.06), Object::Real(0.09), Object::Real(0.16)]));
    ops1.push(Operation::new("re", vec![Object::Real(0.0), Object::Real(745.0), Object::Real(595.28), Object::Real(97.0)]));
    ops1.push(Operation::new("f", vec![]));

    // Accent line (3pt blue)
    ops1.push(Operation::new("rg", vec![Object::Real(0.15), Object::Real(0.45), Object::Real(0.92)]));
    ops1.push(Operation::new("re", vec![Object::Real(0.0), Object::Real(742.0), Object::Real(595.28), Object::Real(3.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    // Company Title (White Bold 18pt)
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(18.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 802.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("NOVA TECHNOLOGIES")]));
    ops1.push(Operation::new("ET", vec![]));

    // Company Subtitle (Light Gray 8.5pt)
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.65), Object::Real(0.72), Object::Real(0.82)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 786.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("ENTERPRISE CLOUD & COGNITIVE SYSTEMS")]));
    ops1.push(Operation::new("ET", vec![]));

    // Header badge container on right
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.12), Object::Real(0.18), Object::Real(0.28)]));
    ops1.push(Operation::new("re", vec![Object::Real(415.0), Object::Real(802.0), Object::Real(140.28), Object::Real(20.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    // Badge text
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.75), Object::Real(1.0)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 425.into(), 808.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("OFFICIAL STATEMENT")]));
    ops1.push(Operation::new("ET", vec![]));

    // Document Number placeholder (right-aligned, ending at 555.28)
    let doc_num_x = right_align_x("{{document.number}}", 14.0, 555.28);
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(14.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), doc_num_x.into(), 776.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{document.number}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // Document Status placeholder (right-aligned, ending at 555.28)
    let doc_stat_x = right_align_x("{{document.status}}", 8.5, 555.28);
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.35), Object::Real(0.85), Object::Real(0.55)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), doc_stat_x.into(), 760.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{document.status}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // 2. Metadata Cards (Y: 628 to 726)
    // Left Card: CLIENT INFORMATION
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.97), Object::Real(0.98), Object::Real(0.99)]));
    ops1.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(628.0), Object::Real(250.0), Object::Real(98.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("w", vec![Object::Real(0.75)]));
    ops1.push(Operation::new("RG", vec![Object::Real(0.88), Object::Real(0.91), Object::Real(0.94)]));
    ops1.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(628.0), Object::Real(250.0), Object::Real(98.0)]));
    ops1.push(Operation::new("S", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.48), Object::Real(0.58)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 712.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("CLIENT INFORMATION")]));

    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(11.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 694.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{client.name}}")]));

    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.28), Object::Real(0.33), Object::Real(0.40)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 680.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{client.company}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 666.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{client.email}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 652.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{client.location}}")]));

    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.50), Object::Real(0.55), Object::Real(0.62)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 52.into(), 638.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Tax ID: {{client.vat_id}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // Right Card: ORDER SPECIFICATIONS
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.97), Object::Real(0.98), Object::Real(0.99)]));
    ops1.push(Operation::new("re", vec![Object::Real(305.28), Object::Real(628.0), Object::Real(250.0), Object::Real(98.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("w", vec![Object::Real(0.75)]));
    ops1.push(Operation::new("RG", vec![Object::Real(0.88), Object::Real(0.91), Object::Real(0.94)]));
    ops1.push(Operation::new("re", vec![Object::Real(305.28), Object::Real(628.0), Object::Real(250.0), Object::Real(98.0)]));
    ops1.push(Operation::new("S", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.48), Object::Real(0.58)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 712.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("ORDER SPECIFICATIONS")]));

    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.45), Object::Real(0.52)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 694.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Issue Date:")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 678.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Due Date:")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 662.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("PO Reference:")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 646.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Currency:")]));

    let order_specs_right = 543.28;
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{order.issue_date}}", 8.5, order_specs_right).into(), 694.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{order.issue_date}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{order.due_date}}", 8.5, order_specs_right).into(), 678.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{order.due_date}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{order.po_ref}}", 8.5, order_specs_right).into(), 662.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{order.po_ref}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{order.currency}}", 8.5, order_specs_right).into(), 646.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{order.currency}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // 3. Itemized Table Section
    // Section Header
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(9.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 612.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("ITEMIZED DELIVERABLES & SERVICES")]));
    ops1.push(Operation::new("ET", vec![]));

    // Table Header Bar (Dark Navy Background at Y: 584 to 606)
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(584.0), Object::Real(515.28), Object::Real(22.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    // Table Header Column Labels (White)
    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 48.into(), 591.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("#")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 75.into(), 591.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("DESCRIPTION")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 345.into(), 591.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("QTY")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 410.into(), 591.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("UNIT RATE")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 500.into(), 591.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("AMOUNT")]));
    ops1.push(Operation::new("ET", vec![]));

    // Single Dynamic Prototype Table Row:
    // With row_pitch = 28.0, row 0 fits cleanly under 584.0.
    // Name at 574, ID/Qty/Rate/Amount at 570, Desc at 563.
    let proto_y = 570.0;
    ops1.push(Operation::new("BT", vec![]));
    // ID
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.45), Object::Real(0.52)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 48.into(), proto_y.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{items.id}}")]));

    // Name (Bold 9pt)
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(9.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 75.into(), (proto_y + 4.0).into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{items.name}}")]));

    // Description (Regular 7.5pt, gray)
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(7.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.45), Object::Real(0.50), Object::Real(0.58)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 75.into(), (proto_y - 7.0).into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{items.desc}}")]));

    // Qty
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.20), Object::Real(0.25), Object::Real(0.32)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 348.into(), proto_y.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{items.qty}}")]));

    // Unit Price (Right aligned to 455.0)
    let rate_str = "{{items.rate}}";
    let rate_x = right_align_x(rate_str, 8.5, 455.0);
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), rate_x.into(), proto_y.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal(rate_str)]));

    // Total Amount (Right aligned to 545.28)
    let amt_str = "{{items.amount}}";
    let amt_x = right_align_x(amt_str, 8.5, 545.28);
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), amt_x.into(), proto_y.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal(amt_str)]));
    ops1.push(Operation::new("ET", vec![]));

    // 4. Financial Totals Summary Card (Anchored at Y: 145 to 255)
    // Card container
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.97), Object::Real(0.98), Object::Real(0.99)]));
    ops1.push(Operation::new("re", vec![Object::Real(305.28), Object::Real(145.0), Object::Real(250.0), Object::Real(110.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("w", vec![Object::Real(0.75)]));
    ops1.push(Operation::new("RG", vec![Object::Real(0.88), Object::Real(0.91), Object::Real(0.94)]));
    ops1.push(Operation::new("re", vec![Object::Real(305.28), Object::Real(145.0), Object::Real(250.0), Object::Real(110.0)]));
    ops1.push(Operation::new("S", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.45), Object::Real(0.52)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 235.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Subtotal:")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 217.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Discount (10%):")]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 199.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Applicable Tax (5%):")]));

    let fin_right = 543.28;
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{financials.subtotal}}", 8.5, fin_right).into(), 235.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{financials.subtotal}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{financials.discount}}", 8.5, fin_right).into(), 217.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{financials.discount}}")]));

    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{financials.tax}}", 8.5, fin_right).into(), 199.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{financials.tax}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // Grand Total Due Banner (Navy Background)
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.06), Object::Real(0.09), Object::Real(0.16)]));
    ops1.push(Operation::new("re", vec![Object::Real(305.28), Object::Real(145.0), Object::Real(250.0), Object::Real(38.0)]));
    ops1.push(Operation::new("f", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(9.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 317.28.into(), 160.0.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("TOTAL AMOUNT DUE:")]));

    // Right-aligned grand total in cyan `#38bdf8`
    let total_right = 543.28;
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(12.0)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.22), Object::Real(0.74), Object::Real(0.97)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), right_align_x("{{financials.total}}", 12.0, total_right).into(), 160.0.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("{{financials.total}}")]));
    ops1.push(Operation::new("ET", vec![]));

    // 5. Page 1 Footer (Y: 40 to 65)
    ops1.push(Operation::new("q", vec![]));
    ops1.push(Operation::new("w", vec![Object::Real(0.5)]));
    ops1.push(Operation::new("RG", vec![Object::Real(0.85), Object::Real(0.88), Object::Real(0.92)]));
    ops1.push(Operation::new("m", vec![Object::Real(40.0), Object::Real(65.0)]));
    ops1.push(Operation::new("l", vec![Object::Real(555.28), Object::Real(65.0)]));
    ops1.push(Operation::new("S", vec![]));
    ops1.push(Operation::new("Q", vec![]));

    ops1.push(Operation::new("BT", vec![]));
    ops1.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(7.5)]));
    ops1.push(Operation::new("rg", vec![Object::Real(0.50), Object::Real(0.55), Object::Real(0.62)]));
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 48.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Nova Technologies SAS  |  Register #RC-2024-88491  |  Casablanca Finance City, Morocco")]));

    let p1_hash_x = right_align_x("Page 1 of 2  |  Hash: {{document.security_hash}}", 7.5, 555.28);
    ops1.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), p1_hash_x.into(), 48.into()]));
    ops1.push(Operation::new("Tj", vec![Object::string_literal("Page 1 of 2  |  Hash: {{document.security_hash}}")]));
    ops1.push(Operation::new("ET", vec![]));

    let content1 = Content { operations: ops1 };
    let stream1_id = doc.add_object(Stream::new(Dictionary::new(), content1.encode().unwrap()));

    let mut page1 = Dictionary::new();
    page1.set("Type", Object::Name(b"Page".to_vec()));
    page1.set("Parent", Object::Reference(pages_id));
    page1.set("Resources", Object::Dictionary(resources.clone()));
    page1.set("MediaBox", vec![0.into(), 0.into(), 595.28.into(), 841.89.into()]);
    page1.set("Contents", Object::Reference(stream1_id));
    doc.objects.insert(page1_id, Object::Dictionary(page1));

    // =========================================================================
    // PAGE 2: DELIVERABLES, PAYMENT INSTRUCTIONS & CONTRACT TERMS
    // =========================================================================
    let mut ops2: Vec<Operation> = Vec::new();

    // 1. Page 2 Top Header Banner (Y: 755 to 842)
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.06), Object::Real(0.09), Object::Real(0.16)]));
    ops2.push(Operation::new("re", vec![Object::Real(0.0), Object::Real(755.0), Object::Real(595.28), Object::Real(87.0)]));
    ops2.push(Operation::new("f", vec![]));

    // Accent line (3pt blue)
    ops2.push(Operation::new("rg", vec![Object::Real(0.15), Object::Real(0.45), Object::Real(0.92)]));
    ops2.push(Operation::new("re", vec![Object::Real(0.0), Object::Real(752.0), Object::Real(595.28), Object::Real(3.0)]));
    ops2.push(Operation::new("f", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    // Title
    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(16.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 806.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("NOVA TECHNOLOGIES")]));
    ops2.push(Operation::new("ET", vec![]));

    // Subtitle
    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.65), Object::Real(0.72), Object::Real(0.82)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 788.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("SPECIFICATIONS, PAYMENT INSTRUCTIONS & CONTRACT TERMS")]));
    ops2.push(Operation::new("ET", vec![]));

    // Badge container on right
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.12), Object::Real(0.18), Object::Real(0.28)]));
    ops2.push(Operation::new("re", vec![Object::Real(410.0), Object::Real(804.0), Object::Real(145.28), Object::Real(20.0)]));
    ops2.push(Operation::new("f", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.75), Object::Real(1.0)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 420.into(), 810.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("STATEMENT ANNEXURE")]));
    ops2.push(Operation::new("ET", vec![]));

    // Ref text
    let p2_doc_x = right_align_x("Ref: {{document.number}}", 9.0, 555.28);
    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(9.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(1.0), Object::Real(1.0), Object::Real(1.0)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), p2_doc_x.into(), 786.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Ref: {{document.number}}")]));

    let p2_stat_x = right_align_x("Status: {{document.status}}", 8.5, 555.28);
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.35), Object::Real(0.85), Object::Real(0.55)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), p2_stat_x.into(), 770.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Status: {{document.status}}")]));
    ops2.push(Operation::new("ET", vec![]));

    // 2. Section 1: KEY DELIVERABLES & MILESTONES (Y: 575 to 730)
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.98), Object::Real(0.99), Object::Real(1.0)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(575.0), Object::Real(515.28), Object::Real(155.0)]));
    ops2.push(Operation::new("f", vec![]));
    ops2.push(Operation::new("w", vec![Object::Real(0.75)]));
    ops2.push(Operation::new("RG", vec![Object::Real(0.88), Object::Real(0.91), Object::Real(0.94)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(575.0), Object::Real(515.28), Object::Real(155.0)]));
    ops2.push(Operation::new("S", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(9.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 710.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("KEY DELIVERABLES & TECHNICAL MILESTONES")]));

    // Single Dynamic Prototype Deliverable Item at Y = 686.0
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(7.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.20), Object::Real(0.25), Object::Real(0.32)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 686.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{deliverables}}")]));
    ops2.push(Operation::new("ET", vec![]));

    // 3. Section 2: WIRE TRANSFER & PAYMENT INSTRUCTIONS (Y: 415 to 555)
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.95), Object::Real(0.96), Object::Real(0.98)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(415.0), Object::Real(515.28), Object::Real(140.0)]));
    ops2.push(Operation::new("f", vec![]));
    // Blue accent bar on left
    ops2.push(Operation::new("rg", vec![Object::Real(0.15), Object::Real(0.45), Object::Real(0.92)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(415.0), Object::Real(4.0), Object::Real(140.0)]));
    ops2.push(Operation::new("f", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.08), Object::Real(0.12), Object::Real(0.19)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 536.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("WIRE TRANSFER & PAYMENT INSTRUCTIONS")]));

    // Column 1 (X: 55 to 280)
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.45), Object::Real(0.52)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 514.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Account Name:")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 494.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Bank & Branch:")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 474.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("IBAN Number:")]));

    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.10), Object::Real(0.15), Object::Real(0.22)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 125.into(), 514.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.account}}")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 125.into(), 494.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.bank}}")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 125.into(), 474.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.iban}}")]));

    // Column 2 (X: 320 to 545)
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(8.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.40), Object::Real(0.45), Object::Real(0.52)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 320.into(), 514.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("SWIFT / BIC:")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 320.into(), 494.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Payment Terms:")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 320.into(), 474.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Inquiries:")]));

    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.10), Object::Real(0.15), Object::Real(0.22)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 395.into(), 514.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.swift}}")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 395.into(), 494.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.terms}}")]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 395.into(), 474.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{payment.contact_email}}")]));

    // Compliance note
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F3".to_vec()), Object::Real(7.0)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.45), Object::Real(0.50), Object::Real(0.58)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 436.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Electronic signature and transaction audit logs are verified under ISO-27001 compliance standards.")]));
    ops2.push(Operation::new("ET", vec![]));

    // 4. Section 3: CONTRACTUAL TERMS & CONDITIONS (Y: 95 to 395)
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.99), Object::Real(0.99), Object::Real(1.0)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(95.0), Object::Real(515.28), Object::Real(300.0)]));
    ops2.push(Operation::new("f", vec![]));
    ops2.push(Operation::new("w", vec![Object::Real(0.75)]));
    ops2.push(Operation::new("RG", vec![Object::Real(0.88), Object::Real(0.91), Object::Real(0.94)]));
    ops2.push(Operation::new("re", vec![Object::Real(40.0), Object::Real(95.0), Object::Real(515.28), Object::Real(300.0)]));
    ops2.push(Operation::new("S", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F2".to_vec()), Object::Real(8.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.35), Object::Real(0.40), Object::Real(0.48)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 375.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("CONTRACTUAL TERMS & CONDITIONS")]));

    // Single Dynamic Prototype Term Item at Y = 350.0
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(7.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.30), Object::Real(0.35), Object::Real(0.42)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 55.into(), 350.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("{{terms}}")]));
    ops2.push(Operation::new("ET", vec![]));

    // 5. Page 2 Footer (Y: 40 to 65)
    ops2.push(Operation::new("q", vec![]));
    ops2.push(Operation::new("w", vec![Object::Real(0.5)]));
    ops2.push(Operation::new("RG", vec![Object::Real(0.85), Object::Real(0.88), Object::Real(0.92)]));
    ops2.push(Operation::new("m", vec![Object::Real(40.0), Object::Real(65.0)]));
    ops2.push(Operation::new("l", vec![Object::Real(555.28), Object::Real(65.0)]));
    ops2.push(Operation::new("S", vec![]));
    ops2.push(Operation::new("Q", vec![]));

    ops2.push(Operation::new("BT", vec![]));
    ops2.push(Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(7.5)]));
    ops2.push(Operation::new("rg", vec![Object::Real(0.50), Object::Real(0.55), Object::Real(0.62)]));
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), 48.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Nova Technologies SAS  |  Confidential Enterprise Document  |  Page 2 of 2")]));

    let p2_hash_x = right_align_x("Hash: {{document.security_hash}}", 7.5, 555.28);
    ops2.push(Operation::new("Tm", vec![1.into(), 0.into(), 0.into(), 1.into(), p2_hash_x.into(), 48.into()]));
    ops2.push(Operation::new("Tj", vec![Object::string_literal("Hash: {{document.security_hash}}")]));
    ops2.push(Operation::new("ET", vec![]));

    let content2 = Content { operations: ops2 };
    let stream2_id = doc.add_object(Stream::new(Dictionary::new(), content2.encode().unwrap()));

    let mut page2 = Dictionary::new();
    page2.set("Type", Object::Name(b"Page".to_vec()));
    page2.set("Parent", Object::Reference(pages_id));
    page2.set("Resources", Object::Dictionary(resources));
    page2.set("MediaBox", vec![0.into(), 0.into(), 595.28.into(), 841.89.into()]);
    page2.set("Contents", Object::Reference(stream2_id));
    doc.objects.insert(page2_id, Object::Dictionary(page2));

    // =========================================================================
    // PAGES DICTIONARY & CATALOG
    // =========================================================================
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Count", Object::Integer(2));
    pages_dict.set("Kids", Object::Array(vec![Object::Reference(page1_id), Object::Reference(page2_id)]));
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(Object::Dictionary(catalog));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    doc.save(out_path1).expect("save template 1");
    doc.save(out_path2).expect("save template 2");
    println!("2-Page Enterprise template generated successfully at {} and {}!", out_path1, out_path2);
}

fn create_base_doc() -> (Document, ObjectId, ObjectId, ObjectId, ObjectId) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_helvetica_id = doc.new_object_id();
    let font_bold_id = doc.new_object_id();
    let font_oblique_id = doc.new_object_id();

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

    let mut font_ob = Dictionary::new();
    font_ob.set("Type", Object::Name(b"Font".to_vec()));
    font_ob.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_ob.set("BaseFont", Object::Name(b"Helvetica-Oblique".to_vec()));
    font_ob.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    doc.objects.insert(font_oblique_id, Object::Dictionary(font_ob));

    (doc, pages_id, font_helvetica_id, font_bold_id, font_oblique_id)
}

fn right_align_x(text: &str, font_size: f64, target_right_x: f64) -> f64 {
    let font = pdf_template_core::parser::font::FontInfo::default();
    let width = font.measure_text_width(text, font_size);
    target_right_x - width
}
