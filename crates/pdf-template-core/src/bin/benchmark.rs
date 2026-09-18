use std::time::Instant;
use lopdf::{Document, Object, Stream, Dictionary};
use lopdf::content::{Content, Operation};
use pdf_template_core::{inspect_template, render_template, RenderOptions};

fn create_benchmark_pdf(num_pages: usize, placeholders_per_page: usize) -> (Vec<u8>, serde_json::Value) {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_id = doc.new_object_id();

    let mut font = Dictionary::new();
    font.set("Type", Object::Name(b"Font".to_vec()));
    font.set("Subtype", Object::Name(b"Type1".to_vec()));
    font.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
    font.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    doc.objects.insert(font_id, Object::Dictionary(font));

    let mut resources = Dictionary::new();
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(font_id));
    resources.set("Font", Object::Dictionary(fonts));

    let mut page_ids = Vec::new();
    let mut data_map = serde_json::Map::new();

    for p in 0..num_pages {
        let page_id = doc.new_object_id();
        let mut ops = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Real(10.0)]),
            Operation::new("Tm", vec![
                Object::Real(1.0), Object::Real(0.0),
                Object::Real(0.0), Object::Real(1.0),
                Object::Real(40.0), Object::Real(800.0),
            ]),
        ];

        for i in 0..placeholders_per_page {
            let key = format!("var_p{}_i{}", p, i);
            let val = format!("Val_{}_{}", p, i);
            data_map.insert(key.clone(), serde_json::Value::String(val));

            ops.push(Operation::new("Td", vec![Object::Real(0.0), Object::Real(-15.0)]));
            ops.push(Operation::new("Tj", vec![Object::string_literal(format!("Label {}: {{{{{}}}}}", i, key))]));
        }

        ops.push(Operation::new("ET", vec![]));

        let content_bytes = Content { operations: ops }.encode().unwrap();
        let stream_id = doc.add_object(Stream::new(Dictionary::new(), content_bytes));

        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set("Resources", Object::Dictionary(resources.clone()));
        page.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]);
        page.set("Contents", Object::Reference(stream_id));
        doc.objects.insert(page_id, Object::Dictionary(page));
        page_ids.push(page_id);
    }

    let mut pages = Dictionary::new();
    pages.set("Type", Object::Name(b"Pages".to_vec()));
    pages.set("Kids", page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>());
    pages.set("Count", Object::Integer(num_pages as i64));
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut buffer = Vec::new();
    doc.save_to(&mut buffer).unwrap();
    (buffer, serde_json::Value::Object(data_map))
}

fn run_benchmark(name: &str, num_pages: usize, placeholders_per_page: usize, iterations: usize) {
    let total_ph = num_pages * placeholders_per_page;
    let (pdf_bytes, data) = create_benchmark_pdf(num_pages, placeholders_per_page);
    let options = RenderOptions::default();

    // Verify inspect
    let inspect_res = inspect_template(&pdf_bytes, None).unwrap();
    assert_eq!(inspect_res.placeholders.len(), total_ph);

    // Warmup
    let _ = render_template(&pdf_bytes, &data, &options, None).unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = render_template(&pdf_bytes, &data, &options, None).unwrap();
    }
    let elapsed = start.elapsed();
    let per_doc = elapsed / (iterations as u32);
    let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "{:<32} | {:>6} pages | {:>6} ph | {:>10.2?} per doc | {:>8.1} docs/sec",
        name, num_pages, total_ph, per_doc, ops_per_sec
    );
}

fn main() {
    println!("=========================================================================================");
    println!("                           RUST CORE BENCHMARK SUITE");
    println!("=========================================================================================");
    println!("{:<32} | {:>8} | {:>8} | {:>14} | {:>12}", "Scenario", "Pages", "Total PH", "Latency / Doc", "Throughput");
    println!("-----------------------------------------------------------------------------------------");

    run_benchmark("1-Page Invoice (Typical)", 1, 5, 200);
    run_benchmark("10-Page Document", 10, 5, 50);
    run_benchmark("100-Page Large Document", 100, 5, 5);
    run_benchmark("1-Page High Density (100 PH)", 1, 100, 50);
    run_benchmark("10-Page Ultra Density (1000 PH)", 10, 100, 10);

    println!("=========================================================================================");
}
