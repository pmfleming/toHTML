use super::*;

#[test]
fn extracts_ink_annotations_from_page_annots() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 300] /Annots [4 0 R] >> endobj
4 0 obj << /Type /Annot /Subtype /Ink /C [0 0.301961 0.901961] /BS << /W 2 >> /InkList [[10 20 30 40 50 20]] /Rect [9 19 51 41] >> endobj
%%EOF"#;

    let extraction = document_pages(pdf).unwrap();
    let ink = &extraction.pages[0].ink_annotations[0];

    assert_eq!(ink.color.as_deref(), Some("#004de6"));
    assert_eq!(ink.width, 2.0);
    assert_eq!(
        ink.paths,
        vec![vec![(10.0, 20.0), (30.0, 40.0), (50.0, 20.0)]]
    );
}
