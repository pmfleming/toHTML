use super::super::*;

#[test]
fn renders_positioned_text_fragments_without_embedding_pdf() {
    let html = render_pages(&[VisualPage {
        page_number: 2,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "Hello <PDF>".to_string(),
            20.0,
            240.0,
            12.0,
            60.0,
        )],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: vec![VisualPath {
            commands: vec![
                PathCommand::MoveTo(296.0, 520.0),
                PathCommand::LineTo(318.0, 520.0),
            ],
            fill: None,
            stroke: Some("#000000".to_string()),
            stroke_width: 16.0,
            stroke_dasharray: None,
        }],
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-recreated-page\" data-page=\"2\""));
    assert!(html.contains("left:20.00pt;top:48.00pt;font-size:12.00pt"));
    assert!(html.contains("Hello &lt;PDF&gt;"));
    assert!(!html.contains("application/pdf"));
}

#[test]
fn renders_text_color_when_present() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![
            TextSegment::new("Blue".to_string(), 20.0, 240.0, 12.0, 30.0)
                .with_color(Some("#6185c2".to_string())),
        ],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains(";color:#6185c2"));
}

#[test]
fn repairs_shifted_subset_text_at_visual_render_boundary() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(400.0),
        height: Some(300.0),
        segments: vec![TextSegment::new(
            "AVXVHGLQWKLVMXWXDOCRQILGHQWLDOLWyAJUHHPHQWµDisclosin".to_string(),
            20.0,
            240.0,
            12.0,
            240.0,
        )],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("As used in this Mutual Confidentiality Agreement, Disclosin"));
    assert!(!html.contains("AVXVHGLQ"));
}

#[test]
fn renders_dense_prose_as_reconstructed_lines() {
    let mut segments = Vec::new();
    for index in 0..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(TextSegment::new(
            format!("Left{index}"),
            20.0,
            y,
            10.0,
            24.0,
        ));
        segments.push(TextSegment::new(
            format!("Right{index}"),
            56.0,
            y,
            10.0,
            30.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(100.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("pdf-text-fragment").count(), 20);
    assert!(html.contains("class=\"pdf-recreated-page pdf-prose-page\""));
    assert!(html.contains("Left0 Right0"));
    assert!(html.contains("width:66.00pt"));
}

#[test]
fn keeps_wide_table_cells_positioned_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(TextSegment::new(
        "FDIS".to_string(),
        20.0,
        280.0,
        10.0,
        24.0,
    ));
    segments.push(TextSegment::new(
        "Report on voting".to_string(),
        104.0,
        280.0,
        10.0,
        80.0,
    ));
    for index in 1..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(TextSegment::new(
            format!("Left{index}"),
            20.0,
            y,
            10.0,
            24.0,
        ));
        segments.push(TextSegment::new(
            format!("Right{index}"),
            144.0,
            y,
            10.0,
            30.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(220.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("left:20.00pt;top:10.00pt;font-size:10.00pt;width:24.00pt"));
    assert!(html.contains("left:104.00pt;top:10.00pt;font-size:10.00pt;width:80.00pt"));
    assert!(!html.contains("FDIS Report on voting"));
}
