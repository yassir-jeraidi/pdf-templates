use lopdf::{Document, Object};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pdf_path = if args.len() > 1 { &args[1] } else { "/Users/yassirjr/projects/demo-pdf-engine/rendered-10-rows.pdf" };

    let doc = Document::load(pdf_path).expect("Failed to load PDF");
    let pages = doc.get_pages();
    let mut sorted_pages: Vec<(u32, lopdf::ObjectId)> = pages.into_iter().collect();
    sorted_pages.sort_by_key(|p| p.0);

    for (idx, (_page_num, page_id)) in sorted_pages.iter().enumerate() {
        let page_index = idx + 1;
        let mut single_doc = doc.clone();

        let page_dict = single_doc.get_object(*page_id).unwrap().as_dict().unwrap();
        let pages_id = page_dict.get(b"Parent").unwrap().as_reference().unwrap();

        let pages_obj = single_doc.get_object_mut(pages_id).unwrap().as_dict_mut().unwrap();
        pages_obj.set("Count", Object::Integer(1));
        pages_obj.set("Kids", Object::Array(vec![Object::Reference(*page_id)]));

        let out_name = format!("{}-page{}.pdf", pdf_path.trim_end_matches(".pdf"), page_index);
        single_doc.save(&out_name).expect("Failed to save single page");
        println!("Saved {}", out_name);
    }
}
