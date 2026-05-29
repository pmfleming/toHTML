use std::collections::HashMap;

use super::super::object::{PdfObjects, PdfReference};
use super::*;

#[test]
fn places_form_graphics_using_page_do_matrix() {
    let source = br#"
1 0 obj
<< /Type /XObject /Subtype /Form /Matrix [1 0 0 1 10 20] /Length 14 >>
stream
0 0 10 5 re f
endstream
endobj
"#;
    let objects = PdfObjects::parse(source);
    let resources = form_resources();

    let (shapes, paths) = form_xobject_graphics(
        &objects,
        &resources,
        &[b"q 2 0 0 3 50 60 cm /Fm0 Do Q".to_vec()],
    );

    assert!(paths.is_empty());
    assert_eq!(shapes.len(), 1);
    assert!((shapes[0].x - 70.0).abs() < 0.1);
    assert!((shapes[0].y - 120.0).abs() < 0.1);
    assert!((shapes[0].width - 20.0).abs() < 0.1);
    assert!((shapes[0].height - 15.0).abs() < 0.1);
}

#[test]
fn places_form_text_using_page_do_matrix() {
    let source = br#"
1 0 obj
<< /Type /XObject /Subtype /Form /Matrix [1 0 0 1 10 20] /Length 37 >>
stream
BT /F1 10 Tf 0 0 Td (Cell) Tj ET
endstream
endobj
"#;
    let objects = PdfObjects::parse(source);
    let resources = form_resources();

    let segments = form_xobject_text_segments(
        &objects,
        &resources,
        &[b"q 2 0 0 3 50 60 cm /Fm0 Do Q".to_vec()],
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        None,
    );

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "Cell");
    assert!((segments[0].x - 70.0).abs() < 0.1);
    assert!((segments[0].y - 120.0).abs() < 0.1);
    assert!((segments[0].font_size - 30.0).abs() < 0.1);
}

fn form_resources() -> HashMap<String, PdfReference> {
    HashMap::from([(
        "Fm0".to_string(),
        PdfReference {
            object: 1,
            generation: 0,
        },
    )])
}
