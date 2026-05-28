use super::*;

#[test]
fn preserves_fragment_rotation() {
    let html = render_pages(&[page(
        1,
        200.0,
        300.0,
        vec![segment("Sideways".to_string(), 20.0, 240.0, 12.0, 60.0).with_rotation(90.0)],
    )])
    .unwrap();

    assert!(html.contains("transform:rotate(90.00deg)"));
}

#[test]
fn avoids_expanding_small_diagram_labels() {
    let html = render_pages(&[page(
        1,
        200.0,
        300.0,
        vec![segment("Technical FAE".to_string(), 20.0, 240.0, 8.0, 90.0)],
    )])
    .unwrap();

    assert!(!html.contains("scaleX("));
}

#[test]
fn renders_shapes_before_text() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![segment("Cell".to_string(), 20.0, 240.0, 12.0, 24.0)],
        shapes: vec![RectShape {
            x: 10.0,
            y: 220.0,
            width: 100.0,
            height: 30.0,
            fill: Some("#eeeeee".to_string()),
            stroke: Some("#000000".to_string()),
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-shape\""));
    assert!(html.find("pdf-shape") < html.find("pdf-text-fragment"));
    assert!(html.contains("background:#eeeeee"));
    assert!(html.contains("border:0.75pt solid #000000"));
}

#[test]
fn renders_images_before_shapes_and_text() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![segment("Caption".to_string(), 20.0, 240.0, 12.0, 42.0)],
        shapes: Vec::new(),
        images: vec![VisualImage {
            src: "data:image/jpeg;base64,YWJjZA==".to_string(),
            mask_src: None,
            alt: "PDF image".to_string(),
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 40.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-image\""));
    assert!(html.contains("src=\"data:image/jpeg;base64,YWJjZA==\""));
    assert!(html.contains("left:10.00pt;top:240.00pt;width:50.00pt;height:40.00pt"));
    assert!(html.find("pdf-image") < html.find("pdf-text-fragment"));
}

#[test]
fn renders_page_background_before_embedded_diagram_images() {
    let html = render_pages(&[VisualPage {
        page_number: 2,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![segment(
            "Diagram label".to_string(),
            20.0,
            220.0,
            12.0,
            72.0,
        )],
        shapes: vec![
            RectShape {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 300.0,
                fill: Some("#ffffff".to_string()),
                stroke: None,
            },
            RectShape {
                x: 10.0,
                y: 220.0,
                width: 100.0,
                height: 2.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
        ],
        images: vec![VisualImage {
            src: "data:image/png;base64,Ym94".to_string(),
            mask_src: None,
            alt: "PDF image".to_string(),
            x: 15.0,
            y: 195.0,
            width: 80.0,
            height: 44.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.find("pdf-shape").unwrap() < html.find("pdf-image").unwrap());
    assert!(html.find("pdf-image").unwrap() < html.find("Diagram label").unwrap());
}
