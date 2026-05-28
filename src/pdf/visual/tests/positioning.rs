use super::*;

#[test]
fn keeps_dense_ruled_table_cells_positioned() {
    let mut segments = Vec::new();
    for row in 0..20 {
        let y = 280.0 - row as f32 * 10.0;
        segments.push(segment(format!("R{row}C0"), 20.0, y, 9.0, 22.0));
        segments.push(segment(format!("R{row}C1"), 58.0, y, 9.0, 22.0));
        segments.push(segment(format!("R{row}C2"), 96.0, y, 9.0, 22.0));
    }

    let mut shapes = Vec::new();
    for row in 0..=20 {
        shapes.push(RectShape {
            x: 16.0,
            y: 286.0 - row as f32 * 10.0,
            width: 120.0,
            height: 0.8,
            fill: Some("#000000".to_string()),
            stroke: None,
        });
    }
    for col in 0..=3 {
        shapes.push(RectShape {
            x: 16.0 + col as f32 * 38.0,
            y: 86.0,
            width: 0.8,
            height: 200.0,
            fill: Some("#000000".to_string()),
            stroke: None,
        });
    }
    for col in 4..=6 {
        shapes.push(RectShape {
            x: 16.0 + col as f32 * 19.0,
            y: 86.0,
            width: 0.8,
            height: 200.0,
            fill: Some("#000000".to_string()),
            stroke: None,
        });
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(150.0),
        height: Some(300.0),
        segments,
        shapes,
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(!html.contains("pdf-prose-page"));
    assert!(html.contains(">R0C0</span>"));
    assert!(html.contains(">R0C1</span>"));
    assert!(!html.contains("R0C0 R0C1"));
}

#[test]
fn keeps_wide_table_cells_positioned_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(segment("FDIS".to_string(), 20.0, 280.0, 10.0, 24.0));
    segments.push(segment(
        "Report on voting".to_string(),
        104.0,
        280.0,
        10.0,
        80.0,
    ));
    for index in 1..4 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(format!("Left{index}"), 20.0, y, 10.0, 24.0));
        segments.push(segment(format!("Right{index}"), 144.0, y, 10.0, 30.0));
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

#[test]
fn keeps_long_two_column_labels_positioned_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(segment(
        "IEC publications search-webstore.iec.ch/advsearchform".to_string(),
        70.0,
        280.0,
        8.0,
        180.0,
    ));
    segments.push(segment(
        "IECGlossary-std.iec.ch/glossary".to_string(),
        304.0,
        280.0,
        8.0,
        160.0,
    ));
    for index in 1..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(
            format!("Left column body text {index}"),
            70.0,
            y,
            8.0,
            120.0,
        ));
        segments.push(segment(
            format!("Right column body text {index}"),
            304.0,
            y,
            8.0,
            126.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(520.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    if !html.contains(">IEC publications search-webstore.iec.ch/advsearchform</span>") {
        panic!("{html}");
    }
    assert!(html.contains(">IECGlossary-std.iec.ch/glossary</span>"));
    assert!(!html.contains(
        "IEC publications search-webstore.iec.ch/advsearchform IECGlossary-std.iec.ch/glossary"
    ));
}

#[test]
fn renders_definition_prose_with_inline_symbols_as_reconstructed_line() {
    let mut segments = Vec::new();
    segments.push(segment("total".to_string(), 20.0, 280.0, 10.0, 22.0));
    segments.push(segment("RMS".to_string(), 46.0, 280.0, 10.0, 18.0));
    segments.push(segment(
        "value of the odd harmonic current components of orders".to_string(),
        70.0,
        280.0,
        10.0,
        180.0,
    ));
    segments.push(
        segment("21 to 39".to_string(), 255.0, 280.0, 10.0, 40.0).with_font_style(
            None,
            None,
            Some("italic".to_string()),
        ),
    );
    segments.push(segment(
        ", expressed as:".to_string(),
        340.0,
        280.0,
        10.0,
        70.0,
    ));
    for index in 1..4 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(
            format!("left definition body text {index}"),
            20.0,
            y,
            10.0,
            110.0,
        ));
        segments.push(segment(
            format!("right definition body text {index}"),
            180.0,
            y,
            10.0,
            120.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(460.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-recreated-page pdf-prose-page\""));
    assert!(
        html.contains(
            "total RMS value of the odd harmonic current components of orders 21 to 39, expressed as:"
        ),
        "{html}"
    );
    assert!(!html.contains(">RMS</span>"));
    assert!(!html.contains("39 , expressed"));
    assert!(!html.contains("font-style:italic"));
}

#[test]
fn renders_uppercase_started_prose_with_wide_inline_fragments_as_line() {
    let segments = vec![
        segment(
            "La Norme internationale".to_string(),
            70.80,
            205.44,
            9.96,
            110.70,
        ),
        segment("6".to_string(), 219.84, 205.44, 9.96, 8.77),
        segment("1000".to_string(), 228.95, 205.44, 9.96, 22.15),
        segment("-3-".to_string(), 252.84, 205.44, 9.96, 12.91),
        segment("IEC".to_string(), 253.20, 205.44, 9.96, 13.08),
        segment(
            "2 a été établie par le sous".to_string(),
            266.27,
            205.44,
            9.96,
            115.56,
        ),
        segment("-".to_string(), 415.67, 205.44, 9.96, 4.98),
        segment("comité 77A:".to_string(), 419.39, 205.44, 9.96, 56.02),
        segment("CEM".to_string(), 487.89, 205.44, 9.96, 25.09),
    ];

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(596.0),
        height: Some(843.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-recreated-page pdf-prose-page\""));
    assert!(
        html.contains(
            "La Norme internationale IEC 61000-3-2 a été établie par le sous-comité 77A: CEM"
        ),
        "{html}"
    );
    assert!(!html.contains(">CEM</span>"));
    assert!(!html.contains("scaleX(1.75)"));
}

#[test]
fn keeps_contents_leader_page_numbers_positioned_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(segment(
        "B.4 Test conditions for video-cassette recorders".to_string(),
        84.0,
        280.0,
        10.0,
        190.0,
    ));
    segments.push(segment(
        "................................".to_string(),
        300.0,
        280.0,
        10.0,
        180.0,
    ));
    segments.push(segment("28".to_string(), 512.0, 280.0, 10.0, 12.0));
    for index in 1..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(format!("Body line {index}"), 84.0, y, 10.0, 80.0));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(596.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains(">B.4 Test conditions for video-cassette recorders</span>"));
    assert!(html.contains("left:512.00pt;top:10.00pt;font-size:10.00pt"));
    assert!(!html.contains(
        "B.4 Test conditions for video-cassette recorders ................................ 28"
    ));
}

#[test]
fn splits_compact_domain_labels_into_inferred_columns() {
    let mut segments = Vec::new();
    for index in 0..20 {
        segments.push(segment(
            format!("Left column body text {index}"),
            70.0,
            280.0 - index as f32 * 10.0,
            8.0,
            120.0,
        ));
        segments.push(segment(
            format!("Right column body text {index}"),
            304.0,
            280.0 - index as f32 * 10.0,
            8.0,
            126.0,
        ));
    }
    segments.push(segment("Extra".to_string(), 480.0, 280.0, 8.0, 24.0));
    segments.push(segment(
        "IEC publications search-webstore.iec.ch/advsearchform IECGlossary-std.iec.ch/glossary"
            .to_string(),
        70.0,
        70.0,
        8.0,
        364.0,
    ));

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(596.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains(">IEC publications search-webstore.iec.ch/advsearchform</span>"));
    assert!(html.contains(">IECGlossary-std.iec.ch/glossary</span>"));
    assert!(!html.contains(
        "IEC publications search-webstore.iec.ch/advsearchform IECGlossary-std.iec.ch/glossary"
    ));
}

#[test]
fn keeps_inline_expression_fragments_joined_despite_large_source_gap() {
    let mut segments = Vec::new();
    segments.push(segment(
        "Total = First + Second + Third".to_string(),
        20.0,
        280.0,
        12.0,
        120.0,
    ));
    segments.push(segment("Value".to_string(), 190.0, 280.0, 12.0, 30.0));
    for index in 1..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(format!("Left{index}"), 20.0, y, 10.0, 24.0));
        segments.push(segment(format!("Right{index}"), 144.0, y, 10.0, 30.0));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(240.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("Total = First + Second + Third Value"));
    assert!(!html.contains(">Value</span>"));
}
