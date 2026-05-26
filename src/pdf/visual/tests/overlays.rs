use super::super::*;

#[test]
fn keeps_long_prose_lines_joined_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(TextSegment::new("a)".to_string(), 20.0, 280.0, 10.0, 10.0));
    segments.push(TextSegment::new(
        "anupdateoftheemissionlimitsforlightingequipmentwitharatedpowerﬂ".to_string(),
        38.0,
        280.0,
        10.0,
        282.0,
    ));
    segments.push(TextSegment::new(
        "W to take".to_string(),
        330.0,
        280.0,
        10.0,
        44.0,
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

    assert!(html.contains(
        "a) an update of the emission limits for lighting equipment with a rated power ≤ 25 W to take"
    ));
}

#[test]
fn keeps_email_table_rows_as_positioned_cells_on_dense_pages() {
    let mut segments = vec![
        TextSegment::new("Podgorbunskikh, Anton".to_string(), 20.0, 280.0, 7.0, 64.0),
        TextSegment::new("BDM Russia".to_string(), 95.0, 280.0, 7.0, 38.0),
        TextSegment::new(
            "Anton.Podgorbunskikh@avnet.eu".to_string(),
            145.0,
            280.0,
            7.0,
            110.0,
        ),
        TextSegment::new("+7 (916) 96778 41".to_string(), 270.0, 280.0, 7.0, 64.0),
    ];
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
            170.0,
            y,
            10.0,
            30.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(340.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(html.contains("Anton.Podgorbunskikh@avnet.eu"));
    assert!(html.contains("left:145.00pt;top:13.00pt;font-size:7.00pt"));
    assert!(!html.contains("BDM Russia Anton.Podgorbunskikh@avnet.eu +7"));
}

#[test]
fn keeps_copyright_headers_joined_on_dense_prose_pages() {
    let mut segments = Vec::new();
    segments.push(TextSegment::new(
        "IEC 61000 -3-2:2018 © IEC 2018".to_string(),
        20.0,
        280.0,
        10.0,
        150.0,
    ));
    segments.push(TextSegment::new(
        "Œ 5 Œ".to_string(),
        260.0,
        280.0,
        10.0,
        30.0,
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

    assert!(html.contains("IEC 61000-3-2:2018 © IEC 2018 – 5 –"));
}

#[test]
fn overlays_iec_definition_formulas_with_mathml() {
    let html = render_pages(&[VisualPage {
        page_number: 11,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![
            TextSegment::new(
                "total harmonic current".to_string(),
                70.0,
                590.0,
                10.0,
                106.0,
            ),
            TextSegment::new("total ha".to_string(), 70.0, 458.0, 10.0, 36.0),
            TextSegment::new("rmonic distortion".to_string(), 110.0, 458.0, 10.0, 82.0),
            TextSegment::new("THD".to_string(), 70.0, 446.0, 10.0, 21.0),
            TextSegment::new(
                "partial odd harmonic current".to_string(),
                70.0,
                290.0,
                10.0,
                136.0,
            ),
        ],
        shapes: vec![RectShape {
            x: 302.0,
            y: 531.0,
            width: 27.0,
            height: 16.0,
            fill: Some("#000000".to_string()),
            stroke: None,
        }],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("class=\"pdf-formula\"").count(), 3);
    assert!(html.contains("<mi>THC</mi><mo>=</mo><msqrt>"));
    assert!(html.contains("<mi>THD</mi><mo>=</mo><msqrt>"));
    assert!(html.contains("<mi>POHC</mi><mo>=</mo><msqrt>"));
    assert!(html.contains("total harmonic distortion"));
    assert!(!html.contains("pdf-shape"));
    assert!(!html.contains("pdf-ink"));
}

#[test]
fn overlays_iec_class_a_limit_table_fractions() {
    let html = render_pages(&[VisualPage {
        page_number: 23,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![
            TextSegment::new("Limits for Class".to_string(), 198.0, 678.0, 10.0, 76.0),
            TextSegment::new("A equipment".to_string(), 332.0, 678.0, 10.0, 62.0),
            TextSegment::new("Odd harmonics".to_string(), 266.0, 618.0, 8.0, 58.0),
            TextSegment::new("Even harmonics".to_string(), 264.0, 482.0, 8.0, 62.0),
            TextSegment::new("15".to_string(), 192.0, 505.0, 8.0, 12.0),
            TextSegment::new("d".to_string(), 204.0, 505.0, 8.0, 4.0),
            TextSegment::new("h".to_string(), 211.0, 505.0, 8.0, 4.0),
            TextSegment::new("d".to_string(), 219.0, 505.0, 8.0, 4.0),
            TextSegment::new("39".to_string(), 226.0, 505.0, 8.0, 9.0),
            TextSegment::new("30".to_string(), 368.0, 282.0, 8.0, 11.0),
            TextSegment::new("Ÿ".to_string(), 381.0, 282.0, 8.0, 4.0),
            TextSegment::new("O".to_string(), 386.0, 282.0, 8.0, 4.0),
            TextSegment::new("11".to_string(), 191.0, 220.0, 8.0, 9.0),
            TextSegment::new("d".to_string(), 203.0, 220.0, 8.0, 4.0),
            TextSegment::new("h".to_string(), 210.0, 220.0, 8.0, 4.0),
            TextSegment::new("d".to_string(), 217.0, 220.0, 8.0, 4.0),
            TextSegment::new("39".to_string(), 225.0, 220.0, 8.0, 9.0),
            TextSegment::new("O".to_string(), 146.0, 172.0, 8.0, 4.0),
        ],
        shapes: vec![
            RectShape {
                x: 386.65,
                y: 503.09,
                width: 9.75,
                height: 8.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
            RectShape {
                x: 388.77,
                y: 412.84,
                width: 5.75,
                height: 8.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
            RectShape {
                x: 299.64,
                y: 522.12,
                width: 0.72,
                height: 28.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
        ],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("class=\"pdf-formula\"").count(), 7);
    assert!(html.contains("15 ≤"));
    assert!(html.contains("≤ 39"));
    assert!(html.contains("8 ≤"));
    assert!(html.contains("≤ 40"));
    assert!(html.contains(">0,15<"));
    assert!(html.contains(">0,23<"));
    assert!(html.contains("30 · <i"));
    assert!(html.contains(">λ</i><sup"));
    assert!(html.contains(">11 ≤"));
    assert!(html.contains("border-top:0.65pt solid #000"));
    assert!(!html.contains("width:9.75pt;height:8.00pt;background:#000000"));
    assert!(!html.contains("width:5.75pt;height:8.00pt;background:#000000"));
    assert!(html.contains("left:299.64pt"));
}

#[test]
fn overlays_french_iec_limit_table_fractions() {
    let html = render_pages(&[VisualPage {
        page_number: 60,
        width: Some(596.0),
        height: Some(843.0),
        segments: vec![
            TextSegment::new(
                "Limites pour les appareils de".to_string(),
                230.0,
                647.0,
                10.0,
                140.0,
            ),
            TextSegment::new("lasse A".to_string(), 389.0, 647.0, 10.0, 35.0),
            TextSegment::new("Harmoniques impairs".to_string(), 253.0, 596.0, 8.0, 84.0),
            TextSegment::new("Harmoniques pairs".to_string(), 258.0, 460.0, 8.0, 73.0),
        ],
        shapes: vec![
            RectShape {
                x: 383.25,
                y: 472.62,
                width: 9.75,
                height: 8.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
            RectShape {
                x: 385.42,
                y: 382.62,
                width: 5.75,
                height: 8.0,
                fill: Some("#000000".to_string()),
                stroke: None,
            },
        ],
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("class=\"pdf-formula\"").count(), 7);
    assert!(html.contains(">0,15<"));
    assert!(html.contains(">0,23<"));
    assert!(html.contains("15 ≤"));
    assert!(html.contains("8 ≤"));
    assert!(html.contains("30 · <i"));
    assert!(!html.contains("width:9.75pt;height:8.00pt;background:#000000"));
    assert!(!html.contains("width:5.75pt;height:8.00pt;background:#000000"));
}

#[test]
fn does_not_overlay_iec_formulas_on_unrelated_page_eleven() {
    let html = render_pages(&[VisualPage {
        page_number: 11,
        width: Some(596.0),
        height: Some(842.0),
        segments: vec![TextSegment::new(
            "ordinary page eleven".to_string(),
            70.0,
            590.0,
            10.0,
            106.0,
        )],
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert!(!html.contains("pdf-formula"));
    assert!(!html.contains("<mi>THC</mi>"));
}
