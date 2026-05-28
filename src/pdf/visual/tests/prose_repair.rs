use super::*;

#[test]
fn repairs_downshifted_subset_text_when_rendering_fragments() {
    let page = page(
        1,
        200.0,
        100.0,
        vec![
            segment("2025 eN".to_string(), 10.0, 70.0, 12.0, 60.0),
            segment("NIQRUKMN".to_string(), 10.0, 50.0, 12.0, 60.0),
        ],
    );

    let html = render_pages(&[page]).expect("visual html");

    assert!(html.contains(">2025 H1</span>"));
    assert!(html.contains(">1,458.01</span>"));
}

#[test]
fn renders_broken_hyphenated_prose_title_as_repaired_line() {
    let mut segments = vec![
        segment(
            "Strategic Project Rewarding (Company Level, Top".to_string(),
            20.0,
            280.0,
            18.0,
            350.0,
        ),
        segment("-".to_string(), 420.0, 280.0, 18.0, 8.0),
        segment("down)".to_string(), 500.0, 280.0, 18.0, 50.0),
    ];
    for index in 0..4 {
        segments.push(segment(
            format!("• Supporting bullet {index}"),
            40.0,
            240.0 - index as f32 * 20.0,
            12.0,
            150.0,
        ));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(620.0),
        height: Some(320.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .expect("visual html");

    assert!(html.contains(">Strategic Project Rewarding (Company Level, Top-down)</span>"));
    assert!(!html.contains(">Top</span>"));
    assert!(!html.contains(">-</span>"));
}

#[test]
fn renders_dense_prose_as_reconstructed_lines() {
    let mut segments = Vec::new();
    for index in 0..20 {
        let y = 280.0 - index as f32 * 10.0;
        segments.push(segment(format!("Left{index}"), 20.0, y, 10.0, 24.0));
        segments.push(segment(format!("Right{index}"), 56.0, y, 10.0, 30.0));
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
fn scales_reconstructed_lines_at_page_edge() {
    let mut segments = Vec::new();
    for index in 0..4 {
        segments.push(segment(
            format!(
                "• This bullet item has enough words to exceed the right edge of the rendered page {index}"
            ),
            20.0,
            280.0 - index as f32 * 18.0,
            10.0,
            260.0,
        )
        .with_font_style(Some("Courier New, Courier, monospace".to_string()), None, None));
    }

    let html = render_pages(&[VisualPage {
        page_number: 1,
        width: Some(180.0),
        height: Some(300.0),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }])
    .unwrap();

    assert_eq!(html.matches("pdf-text-fragment").count(), 4);
    assert!(html.contains("width:160.00pt"));
    assert!(html.contains(";transform:scaleX("));
    assert!(html.contains("right edge of the rendered page 0</span>"));
}

#[test]
fn renders_wide_gap_subbullet_prose_as_repaired_line() {
    let page_height = 300.0;
    let y = 220.0;
    let html = render_pages(&[page(
        1,
        520.0,
        page_height,
        vec![
            segment(
                "• Read Digital Dimming".to_string(),
                70.0,
                240.0,
                12.0,
                130.0,
            ),
            segment("Level".to_string(), 260.0, 240.0, 12.0, 30.0),
            segment("o".to_string(), 108.0, y, 12.0, 7.0),
            segment("Read".to_string(), 126.0, y, 12.0, 22.0),
            segment("igital".to_string(), 166.0, y, 12.0, 28.0),
            segment("D".to_string(), 194.0, y, 12.0, 6.0),
            segment("imming brightness level".to_string(), 203.0, y, 12.0, 68.0),
            segment("D".to_string(), 261.0, y, 12.0, 6.0),
            segment("I returns value between".to_string(), 329.0, y, 12.0, 124.0),
            segment("0".to_string(), 457.0, y, 12.0, 6.0),
            segment("-".to_string(), 464.0, y, 12.0, 6.0),
            segment("200".to_string(), 468.0, y, 12.0, 16.0),
            segment(
                "o Value = dim percentage * 200".to_string(),
                108.0,
                204.0,
                12.0,
                170.0,
            ),
        ],
    )])
    .unwrap();

    assert!(html.contains("class=\"pdf-recreated-page pdf-prose-page\""));
    assert!(
        html.contains("o Read Digital Dimming brightness level, returns value between 0-200"),
        "{html}"
    );
    assert!(!html.contains(">igital</span>"), "{html}");
    assert!(!html.contains("D I returns"), "{html}");
}

#[test]
fn renders_short_style_fragmented_legal_prose_as_repaired_lines() {
    let page_height = 300.0;
    let html = render_pages(&[page(
        12,
        596.0,
        page_height,
        vec![
            segment("ENNF".to_string(), 70.85, 244.0, 12.0, 19.99),
            segment("The purpose of".to_string(), 113.40, 244.0, 12.0, 77.66),
            segment("this Regulation".to_string(), 191.07, 244.0, 12.0, 76.00),
            segment("is to ensure".to_string(), 267.06, 244.0, 12.0, 59.68),
            segment(
                "a high level of cybersecurity of products".to_string(),
                326.74,
                244.0,
                12.0,
                196.93,
            ),
            segment(
                "with digital elements".to_string(),
                113.40,
                223.3,
                12.0,
                100.64,
            ),
            segment(
                "and their integrated remote data processing solutions. Such".to_string(),
                214.04,
                223.3,
                12.0,
                297.04,
            ),
            segment(
                "remote data processing solutions".to_string(),
                113.40,
                202.6,
                12.0,
                159.96,
            ),
            segment("should be defined".to_string(), 273.36, 202.6, 12.0, 87.34),
            segment(
                "as data processing at a distance".to_string(),
                360.70,
                202.6,
                12.0,
                155.60,
            ),
        ],
    )])
    .unwrap();

    assert!(html.contains("class=\"pdf-recreated-page pdf-prose-page\""));
    assert!(html.contains(">(11)</span>"), "{html}");
    assert!(html.contains(">\u{00a0}this Regulation</span>"), "{html}");
    assert!(html.contains(">\u{00a0}is to ensure</span>"), "{html}");
    assert!(html.contains(">\u{00a0}and their integrated"), "{html}");
    assert!(html.contains(">\u{00a0}should be defined</span>"), "{html}");
    assert!(
        html.contains(">\u{00a0}as data processing at a distance</span>"),
        "{html}"
    );
    assert!(!html.contains(">ENNF</span>"), "{html}");
    assert!(!html.contains("element sand"), "{html}");
    assert!(!html.contains("elementsand"), "{html}");
}
