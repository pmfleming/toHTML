use super::*;

#[test]
fn renders_positioned_text_fragments_without_embedding_pdf() {
    let html = render_pages(&[VisualPage {
        page_number: 2,
        width: Some(200.0),
        height: Some(300.0),
        segments: vec![segment("Hello <PDF>".to_string(), 20.0, 240.0, 12.0, 60.0)],
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
    let html = render_pages(&[page(
        1,
        200.0,
        300.0,
        vec![segment("Blue".to_string(), 20.0, 240.0, 12.0, 30.0)
            .with_color(Some("#6185c2".to_string()))],
    )])
    .unwrap();

    assert!(html.contains(";color:#6185c2"));
}

#[test]
fn aligns_long_url_text_with_lower_link_annotation_line() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(500.0),
        height: Some(200.0),
        segments: vec![
            segment(
                "www.iso20022.org/documents/messages/pain/schemas/pain.001.001.03.zip".to_string(),
                55.0,
                120.0,
                11.0,
                320.0,
            )
            .with_color(Some("#0000ff".to_string())),
            segment("E".to_string(), 54.0, 100.0, 11.0, 8.0),
        ],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: vec![VisualLink {
            href: "http://www.iso20022.org/documents/messages/pain/schemas/pain.001.001.03.zip"
                .to_string(),
            x: 55.0,
            y: 95.0,
            width: 348.0,
            height: 16.0,
        }],
    }])
    .unwrap();

    assert!(html.contains("top:89.00pt"), "{html}");
    assert!(html.contains("pain.001.001.03.zip"), "{html}");
    assert!(!html.contains(">E</span>"), "{html}");
}

#[test]
fn expands_same_line_url_text_to_link_annotation_width() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(260.0),
        height: Some(120.0),
        segments: vec![
            segment("www.iso 20022.org".to_string(), 40.0, 80.0, 11.0, 46.0)
                .with_color(Some("#0000ff".to_string())),
        ],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: vec![VisualLink {
            href: "http://www.iso20022.org/".to_string(),
            x: 40.0,
            y: 75.0,
            width: 88.0,
            height: 16.0,
        }],
    }])
    .unwrap();

    assert!(html.contains(">www.iso20022.org/</span>"), "{html}");
    assert!(html.contains("width:88.00pt"), "{html}");
    assert!(!html.contains("scaleX"), "{html}");
}

#[test]
fn repairs_iso20022_catalogue_reference_fragments_on_link_line() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(560.0),
        height: Some(160.0),
        segments: vec![
            segment("0,62 20022".to_string(), 150.0, 90.0, 11.0, 50.0),
            segment("messages”, with “pai".to_string(), 205.0, 90.0, 11.0, 98.0),
            segment(")".to_string(), 304.0, 90.0, 11.0, 6.0),
            segment(
                "n.001.001.03” as reference".to_string(),
                310.0,
                90.0,
                11.0,
                135.0,
            ),
        ],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: vec![VisualLink {
            href: "http://www.iso20022.org/".to_string(),
            x: 40.0,
            y: 53.0,
            width: 88.0,
            height: 16.0,
        }],
    }])
    .unwrap();

    assert!(html.contains(">ISO 20022</span>"), "{html}");
    assert!(
        html.contains(">messages”, with “pain.001.001.03” as reference</span>"),
        "{html}"
    );
    assert!(!html.contains(">0,62 20022</span>"), "{html}");
    assert!(!html.contains(">).</span>"), "{html}");
}

#[test]
fn renders_standalone_checkbox_symbol_as_box_marker() {
    let html = render_pages(&[page(
        1,
        200.0,
        300.0,
        vec![segment("□".to_string(), 20.0, 240.0, 24.0, 12.0)],
    )])
    .unwrap();

    assert!(
        html.contains("class=\"pdf-text-fragment pdf-checkbox-marker\""),
        "{html}"
    );
    assert!(html.contains("border:2.40pt solid #000000"), "{html}");
    assert!(!html.contains(">□</span>"), "{html}");
}

#[test]
fn reconstructs_split_reporting_period_heading_on_image_page() {
    let page_height = 300.0;
    let y = page_height - 36.0 - 24.0;
    let html = render_pages(&[VisualPage {
        page_number: 7,
        width: Some(500.0),
        height: Some(page_height),
        segments: vec![
            segment("Oracle".to_string(), 35.0, y - 8.0, 32.0, 108.0)
                .with_font_style(None, Some(700), None)
                .with_color(Some("#ff6600".to_string())),
            segment("Handling Time E Last".to_string(), 200.0, y, 24.0, 205.0)
                .with_font_style(None, Some(700), None)
                .with_color(Some("#00b0f0".to_string())),
            segment("2".to_string(), 405.0, y, 24.0, 16.0)
                .with_font_style(None, Some(700), None)
                .with_color(Some("#00b0f0".to_string())),
            segment("months )".to_string(), 421.0, y, 24.0, 80.0)
                .with_font_style(None, Some(700), None)
                .with_color(Some("#00b0f0".to_string())),
        ],
        shapes: Vec::new(),
        images: vec![VisualImage {
            src: "chart.png".to_string(),
            mask_src: None,
            alt: String::new(),
            x: 0.0,
            y: 0.0,
            width: 500.0,
            height: 120.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(
        html.contains(">Handling Time (Last 2 months)</span>"),
        "{html}"
    );
    assert!(!html.contains(">2</span>"), "{html}");
    assert!(!html.contains(">months )</span>"), "{html}");
}

#[test]
fn renders_inferred_centered_title_divider_for_standards_title_page() {
    let page_height = 843.0;
    let html = render_pages(&[page(
        40,
        596.0,
        page_height,
        vec![
            segment(
                "COMMISSION ÉLECTROTECHNIQUE INTERNATIONALE".to_string(),
                130.80,
                page_height - 90.12 - 12.0,
                12.0,
                319.21,
            ),
            segment(
                "COMPATIBILITÉ ÉLECTROMAGNÉTIQUE (CEM)".to_string(),
                148.08,
                page_height - 154.20 - 12.0,
                12.0,
                278.93,
            )
            .with_font_style(None, Some(700), None),
        ],
    )])
    .unwrap();

    assert!(html.contains(
        "class=\"pdf-shape\" style=\"left:263.00pt;top:128.16pt;width:70.00pt;height:0.75pt;background:#000000\""
    ));
    assert!(html.find("pdf-shape").unwrap() < html.find("COMPATIBILITÉ").unwrap());
}

#[test]
fn does_not_duplicate_existing_centered_title_divider() {
    let page_height = 843.0;
    let html = render_pages(&[VisualPage {
        page_number: 40,
        width: Some(596.0),
        height: Some(page_height),
        segments: vec![
            segment(
                "COMMISSION ÉLECTROTECHNIQUE INTERNATIONALE".to_string(),
                130.80,
                page_height - 90.12 - 12.0,
                12.0,
                319.21,
            ),
            segment(
                "COMPATIBILITÉ ÉLECTROMAGNÉTIQUE (CEM)".to_string(),
                148.08,
                page_height - 154.20 - 12.0,
                12.0,
                278.93,
            )
            .with_font_style(None, Some(700), None),
        ],
        shapes: vec![RectShape {
            x: 260.0,
            y: page_height - 128.0 - 0.75,
            width: 76.0,
            height: 0.75,
            fill: Some("#000000".to_string()),
            stroke: None,
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("class=\"pdf-shape\"").count(), 1);
}

#[test]
fn renders_inferred_sum_marker_in_harmonic_formula_cluster() {
    let page_height = 842.0;
    let html = render_pages(&[page(
        11,
        596.0,
        page_height,
        vec![
            segment(
                "THD".to_string(),
                252.0,
                page_height - 479.0 - 10.7,
                10.7,
                22.0,
            )
            .with_font_style(
                Some("Times New Roman, Times, serif".to_string()),
                None,
                Some("italic".to_string()),
            ),
            segment(
                "40".to_string(),
                296.0,
                page_height - 469.0 - 8.6,
                8.6,
                10.0,
            ),
            segment(
                "Ih".to_string(),
                316.0,
                page_height - 472.0 - 10.7,
                10.7,
                9.0,
            )
            .with_font_style(
                Some("Times New Roman, Times, serif".to_string()),
                None,
                Some("italic".to_string()),
            ),
            segment(
                "h 2".to_string(),
                293.0,
                page_height - 495.0 - 8.6,
                8.6,
                16.0,
            )
            .with_font_style(
                Some("Times New Roman, Times, serif".to_string()),
                None,
                Some("italic".to_string()),
            ),
        ],
    )])
    .unwrap();

    assert!(html.contains(">∑</span>"), "{html}");
    assert!(html.contains("left:293.38pt;top:474.88pt;font-size:18.73pt"));
    assert!(html.contains(">40</span>"));
    assert!(html.contains(">h 2</span>"));
}

#[test]
fn does_not_duplicate_existing_sum_marker_in_formula_cluster() {
    let page_height = 842.0;
    let html = render_pages(&[page(
        11,
        596.0,
        page_height,
        vec![
            segment(
                "THC".to_string(),
                265.0,
                page_height - 313.0 - 10.0,
                10.0,
                20.0,
            ),
            segment("40".to_string(), 306.0, page_height - 303.0 - 8.0, 8.0, 9.0),
            segment(
                "∑".to_string(),
                304.0,
                page_height - 314.0 - 16.0,
                16.0,
                8.0,
            ),
            segment(
                "I".to_string(),
                318.0,
                page_height - 313.0 - 10.0,
                10.0,
                5.0,
            ),
            segment(
                "h 2".to_string(),
                303.0,
                page_height - 327.0 - 8.0,
                8.0,
                15.0,
            ),
        ],
    )])
    .unwrap();

    assert_eq!(html.matches(">∑</span>").count(), 1);
}
