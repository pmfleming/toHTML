use super::*;

#[test]
fn renders_repeated_filled_outline_boxes_as_borders() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(220.0),
        segments: vec![segment("M".to_string(), 76.0, 158.0, 8.0, 8.0)],
        shapes: vec![
            RectShape {
                x: 40.0,
                y: 120.0,
                width: 60.0,
                height: 40.0,
                fill: Some("#ffffff".to_string()),
                stroke: None,
            },
            RectShape {
                x: 39.5,
                y: 119.5,
                width: 61.0,
                height: 41.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
            RectShape {
                x: 118.0,
                y: 120.0,
                width: 30.0,
                height: 40.0,
                fill: Some("#ffffff".to_string()),
                stroke: None,
            },
            RectShape {
                x: 117.5,
                y: 119.5,
                width: 31.0,
                height: 41.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
        ],
        images: Vec::new(),
        paths: vec![
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(39.5, 119.5),
                    PathCommand::LineTo(100.5, 119.5),
                    PathCommand::LineTo(100.5, 160.5),
                    PathCommand::LineTo(39.5, 160.5),
                    PathCommand::Close,
                ],
                fill: Some("#000000".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(20.0, 180.0),
                    PathCommand::LineTo(28.0, 166.0),
                    PathCommand::LineTo(36.0, 180.0),
                    PathCommand::Close,
                ],
                fill: Some("#000000".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
        ],
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("background:#ffffff;border:0.75pt solid #000000"));
    assert!(html.contains("fill=\"none\" stroke=\"#000000\""));
    assert!(html.contains("L28.00 54.00L36.00 40.00Z\" fill=\"#000000\""));
}

#[test]
fn renders_large_covered_filled_paths_before_images() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(100.0),
        segments: vec![segment("Title".to_string(), 20.0, 80.0, 12.0, 30.0)],
        shapes: Vec::new(),
        images: vec![VisualImage {
            src: "data:image/png;base64,abc".to_string(),
            mask_src: None,
            alt: "PDF image on page 1".to_string(),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        }],
        paths: vec![
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(90.0, 0.0),
                    PathCommand::LineTo(200.0, 0.0),
                    PathCommand::LineTo(200.0, 100.0),
                    PathCommand::LineTo(90.0, 100.0),
                    PathCommand::Close,
                ],
                fill: Some("#dedede".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(10.0, 90.0),
                    PathCommand::LineTo(20.0, 90.0),
                    PathCommand::LineTo(20.0, 80.0),
                    PathCommand::Close,
                ],
                fill: Some("#ffffff".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
        ],
        links: Vec::new(),
    }])
    .unwrap();

    let background_path = html.find("fill=\"#dedede\"").unwrap();
    let image = html.find("class=\"pdf-image\"").unwrap();
    let title = html.find(">Title</span>").unwrap();
    let logo_path = html.find("fill=\"#ffffff\"").unwrap();

    assert!(background_path < image);
    assert!(image < title);
    assert!(title < logo_path);
}

#[test]
fn renders_large_filled_container_paths_before_text() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(400.0),
        height: Some(240.0),
        segments: vec![segment("Card text".to_string(), 70.0, 128.0, 14.0, 70.0)
            .with_color(Some("#009eb9".to_string()))],
        shapes: Vec::new(),
        images: vec![VisualImage {
            src: "data:image/png;base64,shadow".to_string(),
            mask_src: None,
            alt: "PDF image on page 1".to_string(),
            x: 52.0,
            y: 70.0,
            width: 210.0,
            height: 80.0,
        }],
        paths: vec![
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(60.0, 70.0),
                    PathCommand::CubicTo(60.0, 64.0, 66.0, 58.0, 72.0, 58.0),
                    PathCommand::LineTo(260.0, 58.0),
                    PathCommand::CubicTo(266.0, 58.0, 272.0, 64.0, 272.0, 70.0),
                    PathCommand::LineTo(272.0, 150.0),
                    PathCommand::CubicTo(272.0, 156.0, 266.0, 162.0, 260.0, 162.0),
                    PathCommand::LineTo(72.0, 162.0),
                    PathCommand::CubicTo(66.0, 162.0, 60.0, 156.0, 60.0, 150.0),
                    PathCommand::Close,
                ],
                fill: Some("#ffffff".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
            VisualPath {
                commands: vec![
                    PathCommand::MoveTo(16.0, 224.0),
                    PathCommand::LineTo(26.0, 224.0),
                    PathCommand::LineTo(26.0, 214.0),
                    PathCommand::Close,
                ],
                fill: Some("#ff6600".to_string()),
                stroke: None,
                stroke_width: 1.0,
                stroke_dasharray: None,
            },
        ],
        links: Vec::new(),
    }])
    .unwrap();

    let container_path = html.find("fill=\"#ffffff\"").unwrap();
    let text = html.find(">Card text</span>").unwrap();
    let logo_path = html.find("fill=\"#ff6600\"").unwrap();

    assert!(container_path < text);
    assert!(text < logo_path);
}
