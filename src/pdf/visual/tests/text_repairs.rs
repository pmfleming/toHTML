use super::super::text_repair::repair_visual_text;
use super::super::*;

#[test]
fn skips_iec_license_artifacts_from_visual_layer() {
    let html = render_pages(&[VisualPage {
        page_number: 11,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![TextSegment::new(
            "Provided by IHS Markit under license with IEC--`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---"
                .to_string(),
            0.0,
            4.0,
            4.0,
            28.0,
        )],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(!html.contains("Provided by IHS"));
    assert!(!html.contains("--`,```"));
}

#[test]
fn repairs_misfilled_iec_flowchart_boxes() {
    let html = render_pages(&[VisualPage {
        page_number: 20,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![
            TextSegment::new(
                "Flowchart for determining conformity".to_string(),
                174.0,
                160.0,
                10.0,
                180.0,
            ),
            TextSegment::new("See Clause 4".to_string(), 292.0, 278.0, 8.0, 48.0)
                .with_color(Some("#ffffff".to_string())),
        ],
        shapes: vec![RectShape {
            x: 138.5,
            y: 676.0,
            width: 70.9,
            height: 42.3,
            fill: Some("#000000".to_string()),
            stroke: Some("#000000".to_string()),
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("background:#ffffff;border:0.75pt solid #000000"));
    assert!(!html.contains("background:#000000;border:0.75pt solid #000000"));
    assert!(html.contains("See Clause 4"));
    assert!(!html.contains(";color:#ffffff"));
}

#[test]
fn repairs_french_iec_flowchart_labels() {
    let html = render_pages(&[VisualPage {
        page_number: 57,
        width: Some(596.0),
        height: Some(843.0),
        segments: vec![
            TextSegment::new(
                "Figure 1ŒOrganigramme permettant de déterminer la conformité".to_string(),
                180.0,
                175.0,
                10.0,
                220.0,
            ),
            TextSegment::new("Voir Article 4".to_string(), 404.0, 660.0, 8.0, 46.0)
                .with_color(Some("#ffffff".to_string())),
            TextSegment::new("de l™Annexe B".to_string(), 148.0, 568.0, 8.0, 50.0),
            TextSegment::new("Conditions d".to_string(), 141.0, 514.0, 8.0, 45.0),
            TextSegment::new("™essai".to_string(), 186.0, 514.0, 8.0, 23.0),
            TextSegment::new("ﬁgénériquesﬂ".to_string(), 151.0, 430.0, 8.0, 48.0),
        ],
        shapes: vec![RectShape {
            x: 138.5,
            y: 676.0,
            width: 70.9,
            height: 42.3,
            fill: Some("#000000".to_string()),
            stroke: Some("#000000".to_string()),
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("background:#ffffff;border:0.75pt solid #000000"));
    assert!(html.contains("Voir Article 4"));
    assert!(!html.contains(";color:#ffffff"));
    assert!(html.contains("Figure 1 – Organigramme"));
    assert!(html.contains("de l'Annexe B"));
    assert!(html.contains("'essai"));
    assert!(html.contains("&quot;génériques&quot;"));
}

#[test]
fn reconstructs_installation_moisture_diagram_column() {
    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![
            TextSegment::new(
                "Installation Guidelines Prevention of Moisture Ingress".to_string(),
                72.0,
                760.0,
                14.0,
                250.0,
            ),
            TextSegment::new("Best Practice".to_string(), 300.0, 630.0, 8.0, 45.0),
            TextSegment::new(
                "Acceptable Alternative".to_string(),
                300.0,
                440.0,
                8.0,
                80.0,
            ),
            TextSegment::new("Things to Avoid".to_string(), 300.0, 335.0, 8.0, 70.0),
        ],
        shapes: vec![RectShape {
            x: 87.0,
            y: 92.0,
            width: 8.0,
            height: 532.0,
            fill: Some("#0000ff".to_string()),
            stroke: None,
        }],
        images: vec![VisualImage {
            src: "data:image/png;base64,YWJjZA==".to_string(),
            mask_src: None,
            alt: "PDF image".to_string(),
            x: 125.0,
            y: 395.0,
            width: 195.0,
            height: 36.0,
        }],
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("id=\"moisture-status\""));
    assert!(!html.contains("background:#0000ff"));
    assert!(html.contains("class=\"pdf-image\""));
}

#[test]
fn reconstructs_iec_single_phase_measurement_circuit() {
    let html = render_pages(&[VisualPage {
        page_number: 26,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![TextSegment::new(
            "Figure A. 1 – Measurement circuit for single-phase equipment".to_string(),
            140.0,
            433.0,
            10.0,
            310.0,
        )],
        shapes: vec![RectShape {
            x: 193.30,
            y: 556.22,
            width: 224.25,
            height: 138.35,
            fill: Some("#000000".to_string()),
            stroke: None,
        }],
        images: Vec::new(),
        paths: vec![VisualPath {
            commands: vec![
                PathCommand::MoveTo(193.30, 694.32),
                PathCommand::LineTo(416.60, 694.32),
                PathCommand::LineTo(416.60, 556.92),
                PathCommand::LineTo(193.30, 556.92),
                PathCommand::Close,
            ],
            fill: Some("#000000".to_string()),
            stroke: None,
            stroke_width: 1.0,
            stroke_dasharray: None,
        }],
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-diagram\""));
    assert!(html.contains("<rect x=\"14\" y=\"74\" width=\"43\" height=\"82\"/>"));
    assert!(html.contains("<circle cx=\"148\" cy=\"20\" r=\"8\"/>"));
    assert!(html.contains("Figure A. 1"));
    assert!(!html.contains("width:224.25pt;height:138.35pt;background:#000000"));
    assert!(!html.contains("<path d=\"M193.30"));
}

#[test]
fn reconstructs_iec_three_phase_measurement_circuit() {
    let html = render_pages(&[VisualPage {
        page_number: 27,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![
            TextSegment::new(
                "Figure A. 2 – Measurement circuit for three-phase equipment".to_string(),
                142.0,
                317.0,
                10.0,
                330.0,
            ),
            TextSegment::new("L1".to_string(), 235.0, 697.0, 8.0, 10.0)
                .with_color(Some("#ffffff".to_string())),
        ],
        shapes: vec![RectShape {
            x: 131.85,
            y: 486.02,
            width: 73.90,
            height: 239.40,
            fill: None,
            stroke: Some("#000000".to_string()),
        }],
        images: Vec::new(),
        paths: vec![VisualPath {
            commands: vec![
                PathCommand::MoveTo(139.85, 692.82),
                PathCommand::LineTo(391.55, 692.82),
            ],
            fill: None,
            stroke: Some("#000000".to_string()),
            stroke_width: 1.0,
            stroke_dasharray: None,
        }],
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("class=\"pdf-diagram\""));
    assert!(html.contains("<rect x=\"6\" y=\"29\" width=\"74\" height=\"239\"/>"));
    assert!(html.contains("<rect x=\"266\" y=\"29\" width=\"75\" height=\"239\"/>"));
    assert!(html.contains("<circle cx=\"173\" cy=\"37\" r=\"14\"/>"));
    assert!(html.contains("L1"));
    assert!(!html.contains(";color:#ffffff"));
    assert!(!html.contains("left:131.85pt;top:116.58pt;width:73.90pt"));
    assert!(!html.contains("<path d=\"M139.85"));
}

#[test]
fn repairs_iec_graph_diagram_labels() {
    assert_eq!(repair_visual_text("d65°"), "≤65°");
    assert_eq!(repair_visual_text("t90°"), "≥90°");
    assert_eq!(repair_visual_text("d60°"), "≤60°");
    assert_eq!(repair_visual_text("Œ0,05Ip(abs)"), "−0,05Ip(abs)");
    assert_eq!(repair_visual_text("pŒ"), "p−");
    assert_eq!(
        repair_visual_text("NOTE Ip(abs) is the higher absolute value of Ip andIp-."),
        "NOTE Ip(abs) is the higher absolute value of Ip and Ip−."
    );
    assert_eq!(
        repair_visual_text("Figure2 ŒIllustration of the r elative phase angle"),
        "Figure 2 – Illustration of the relative phase angle"
    );
    assert_eq!(repair_visual_text("Œ20Œ"), "– 20 –");
    assert_eq!(repair_visual_text("Œ"), "−");
}
