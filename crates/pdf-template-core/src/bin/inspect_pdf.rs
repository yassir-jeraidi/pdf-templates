use lopdf::Document;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pdf_path = if args.len() > 1 { &args[1] } else { "/Users/yassirjr/projects/demo-pdf-engine/enterprise-template.pdf" };
    let doc = Document::load(pdf_path).unwrap();
    let pages = doc.get_pages();
    let mut sorted_pages: Vec<(u32, lopdf::ObjectId)> = pages.into_iter().collect();
    sorted_pages.sort_by_key(|p| p.0);

    for (page_num, page_id) in sorted_pages {
        println!("=== PAGE {} ({:?}) ===", page_num, page_id);
        let page_dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
        println!("Page Dict: {:?}", page_dict);
        let content_data = doc.get_page_content(page_id);
        let content = lopdf::content::Content::decode(&content_data).unwrap();
        for op in content.operations {
            if op.operator == "Tj" || op.operator == "TJ" || op.operator == "Tf" || op.operator == "BT" || op.operator == "ET" {
                println!("{:?} {:?}", op.operator, op.operands);
            }
        }
    }
}
