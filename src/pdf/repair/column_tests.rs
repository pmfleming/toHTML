use super::super::text;
use super::*;

fn segment(text: &str, x: f32, y: f32, font_size: f32, width: f32) -> text::TextSegment {
    text::TextSegment::new(text.to_string(), x, y, font_size, width)
}

fn anchor_rows(left_x: f32, right_x: f32, rows: &[(f32, &str)]) -> Vec<text::TextSegment> {
    rows.iter()
        .flat_map(|(y, suffix)| {
            [
                segment(&format!("Left {suffix}"), left_x, *y, 8.0, 80.0),
                segment(&format!("Right {suffix}"), right_x, *y, 8.0, 90.0),
            ]
        })
        .collect()
}

fn standard_band(middle: text::TextSegment) -> Vec<text::TextSegment> {
    let mut segments = anchor_rows(
        70.0,
        304.0,
        &[(620.0, "nearby"), (584.0, "below"), (560.0, "final")],
    );
    segments.insert(2, middle);
    segments
}

fn leader_page_anchors() -> Vec<text::TextSegment> {
    vec![
        segment("First anchor", 70.0, 650.0, 10.0, 90.0),
        segment("28", 512.0, 650.0, 10.0, 12.0),
        segment("Second anchor", 70.0, 590.0, 10.0, 90.0),
        segment("29", 512.0, 590.0, 10.0, 12.0),
        segment("Third anchor", 70.0, 560.0, 10.0, 90.0),
        segment("30", 512.0, 560.0, 10.0, 12.0),
        segment("Fourth anchor", 70.0, 530.0, 10.0, 90.0),
        segment("31", 512.0, 530.0, 10.0, 12.0),
    ]
}

fn text_at<'a>(segments: &'a [text::TextSegment], value: &str) -> &'a text::TextSegment {
    segments
        .iter()
        .find(|segment| segment.text == value)
        .unwrap_or_else(|| panic!("expected segment text: {value}"))
}

fn assert_text_x(segments: &[text::TextSegment], value: &str, expected: f32) {
    assert!((text_at(segments, value).x - expected).abs() <= 1.0);
}

fn assert_text_y(segments: &[text::TextSegment], value: &str, expected: f32) {
    assert!((text_at(segments, value).y - expected).abs() <= 1.0);
}

#[test]
fn splits_wide_segment_across_repeated_column_anchors() {
    let mut segments = anchor_rows(
        70.0,
        304.0,
        &[
            (620.0, "nearby"),
            (584.0, "below"),
            (560.0, "farther"),
            (540.0, "final"),
        ],
    );
    segments.insert(
        2,
        segment(
            "IEC publications search-webstore.iec.ch/advsearchform IECGlossary-std.iec.ch/glossary",
            70.0,
            600.0,
            8.0,
            365.0,
        ),
    );

    split_segments_at_column_gaps(520.0, &mut segments);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "IEC publications search-webstore.iec.ch/advsearchform"));
    assert_text_x(&segments, "IEC Glossary-std.iec.ch/glossary", 304.0);
}

#[test]
fn snaps_embedded_column_start_to_repeated_anchor() {
    let mut segments = standard_band(segment(
        "IEC publications search-webstore.iec.ch/advsearchform",
        70.0,
        600.0,
        8.0,
        190.0,
    ));
    segments.insert(
        3,
        segment("IECGlossary-std.iec.ch/glossary", 268.0, 600.0, 8.0, 120.0),
    );

    split_segments_at_column_gaps(520.0, &mut segments);

    assert_text_x(&segments, "IECGlossary-std.iec.ch/glossary", 304.0);
}

#[test]
fn keeps_table_label_when_target_anchor_already_has_cell_text() {
    let mut segments = Vec::new();
    for (row, label, tag, cardinality, kind) in [
        ("2.2", "Payment Method", "<PmtMtd>", "[1..1]", "Code"),
        ("2.3", "Batch Booking", "<BtchBookg>", "[0..1]", "Indicator"),
        (
            "2.4",
            "Number Of Transactions",
            "<NbOfTxs>",
            "[0..1]",
            "Text",
        ),
        ("2.5", "Control Sum", "<CtrlSum>", "[0..1]", "Quantity"),
    ] {
        let y = 620.0 - segments.len() as f32 * 3.0;
        segments.push(segment(row, 54.0, y, 11.04, 20.0));
        segments.push(segment(label, 111.62, y, 11.04, 110.0));
        segments.push(segment(tag, 292.51, y, 11.04, 65.0));
        segments.push(segment(cardinality, 390.70, y, 11.04, 24.0));
        segments.push(segment(kind, 447.82, y, 11.04, 44.0));
    }

    split_segments_at_column_gaps(596.0, &mut segments);

    assert_text_x(&segments, "Batch Booking", 111.62);
    assert_text_x(&segments, "<BtchBookg>", 292.51);
    assert_text_x(&segments, "Number Of Transactions", 111.62);
    assert_text_x(&segments, "<NbOfTxs>", 292.51);
}

#[test]
fn splits_fused_column_text_even_when_reported_width_is_narrow() {
    let mut segments = standard_band(segment(
        "The left column sentence continues with enough words RightColumn starts here",
        70.0,
        600.0,
        8.0,
        150.0,
    ));
    segments.extend(anchor_rows(70.0, 304.0, &[(540.0, "last")]));

    split_segments_at_column_gaps(520.0, &mut segments);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "The left column sentence continues with enough words"));
    assert_text_x(&segments, "RightColumn starts here", 304.0);
}

#[test]
fn keeps_single_column_prose_from_splitting_at_right_margin_anchors() {
    let mut segments = vec![
        segment("Intro above", 70.0, 640.0, 10.0, 90.0),
        segment("right edge", 423.0, 640.0, 10.0, 60.0),
        segment(
            "This part of IEC61000 is applicable to electrical equipment having a rated",
            70.0,
            620.0,
            10.0,
            420.0,
        ),
        segment("Body below", 70.0, 600.0, 10.0, 90.0),
        segment("right below", 423.0, 600.0, 10.0, 60.0),
        segment("Body final", 70.0, 580.0, 10.0, 90.0),
        segment("right final", 423.0, 580.0, 10.0, 60.0),
        segment("Body last", 70.0, 560.0, 10.0, 90.0),
        segment("right last", 423.0, 560.0, 10.0, 60.0),
    ];

    split_segments_at_column_gaps(596.0, &mut segments);

    assert!(segments.iter().any(|segment| {
        segment.text == "This part of IEC61000 is applicable to electrical equipment having a rated"
    }));
    assert_eq!(
        segments
            .iter()
            .filter(|segment| segment.text == "a rated")
            .count(),
        0
    );
}

#[test]
fn strips_leading_license_artifact_and_snaps_to_text_anchor() {
    let mut segments = vec![
        segment("Body above", 70.0, 640.0, 10.0, 90.0),
        segment("Right above", 304.0, 640.0, 10.0, 90.0),
        segment(
            "--`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested under conditions.",
            59.0,
            620.0,
            10.0,
            360.0,
        ),
    ];
    segments.extend(anchor_rows(
        70.0,
        304.0,
        &[(600.0, "below"), (580.0, "final"), (560.0, "last")],
    ));

    split_segments_at_column_gaps(596.0, &mut segments);

    assert_text_x(&segments, "equipment tested under conditions.", 70.0);
}

#[test]
fn strips_embedded_license_artifact_from_segment_text() {
    let mut segments = vec![
        segment("Body above", 70.0, 640.0, 10.0, 90.0),
        segment("Right above", 304.0, 640.0, 10.0, 90.0),
        segment(
            "which can be produced by --`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested",
            70.0,
            620.0,
            10.0,
            420.0,
        ),
    ];
    segments.extend(anchor_rows(
        70.0,
        304.0,
        &[(600.0, "below"), (580.0, "final"), (560.0, "last")],
    ));

    split_segments_at_column_gaps(596.0, &mut segments);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "which can be produced by equipment tested"));
    assert!(!segments
        .iter()
        .any(|segment| segment.text.contains("`,```")));
}

#[test]
fn removes_standalone_license_artifact_segments() {
    let mut segments = vec![
        segment("Body above", 70.0, 640.0, 10.0, 90.0),
        segment("Right above", 304.0, 640.0, 10.0, 90.0),
        segment(
            "--`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---",
            59.0,
            620.0,
            10.0,
            220.0,
        ),
        segment("equipment tested", 70.0, 620.0, 10.0, 90.0),
    ];
    segments.extend(anchor_rows(
        70.0,
        304.0,
        &[(600.0, "below"), (580.0, "final"), (560.0, "last")],
    ));

    split_segments_at_column_gaps(596.0, &mut segments);

    assert!(!segments
        .iter()
        .any(|segment| segment.text.contains("`,```")));
    assert_text_x(&segments, "equipment tested", 70.0);
}

#[test]
fn aligns_embedded_right_column_heading_with_previous_left_heading() {
    let mut segments = standard_band(segment(
        "Left body continues with enough words for width RightHeading-example.org/help",
        70.0,
        610.0,
        8.0,
        150.0,
    ));
    segments.insert(
        2,
        segment("Left Heading-example.org/topic", 70.0, 620.0, 8.0, 140.0),
    );
    segments[0].y = 640.0;
    segments[1].y = 640.0;

    split_segments_at_column_gaps(520.0, &mut segments);

    let right_heading = text_at(&segments, "RightHeading-example.org/help");
    assert!((right_heading.x - 304.0).abs() <= 1.0);
    assert!((right_heading.y - 620.0).abs() <= 1.0);
}

#[test]
fn does_not_treat_checklist_marker_column_as_table_anchor() {
    let mut segments = vec![
        segment(
            "Current Backlog By Workstream (",
            43.78,
            368.23,
            18.0,
            267.64,
        ),
        segment("01", 311.52, 368.23, 18.0, 20.0),
        segment("-Dec-2024)", 331.44, 368.23, 18.0, 84.0),
        segment("✓", 618.36, 368.06, 24.0, 12.0),
        segment("Change Request", 654.38, 368.06, 24.0, 156.0),
    ];
    segments.extend([
        segment("Left A", 43.78, 430.0, 18.0, 70.0),
        segment("Right A", 654.38, 430.0, 18.0, 90.0),
        segment("Left B", 43.78, 410.0, 18.0, 70.0),
        segment("Right B", 654.38, 410.0, 18.0, 90.0),
        segment("Left C", 43.78, 390.0, 18.0, 70.0),
        segment("Right C", 654.38, 390.0, 18.0, 90.0),
    ]);

    split_segments_at_column_gaps(1152.0, &mut segments);

    assert_text_x(&segments, "01", 311.52);
}

#[test]
fn lifts_split_right_domain_heading_to_previous_left_heading_line() {
    let mut segments = vec![
        segment("Left nearby", 70.0, 650.0, 8.0, 80.0),
        segment("Right nearby", 304.0, 650.0, 8.0, 90.0),
        segment("Left Heading-example.org/topic", 70.0, 620.0, 8.0, 150.0),
        segment(
            "Left body continues with enough words",
            70.0,
            610.0,
            8.0,
            160.0,
        ),
        segment("ABC", 304.0, 610.0, 8.0, 16.0),
        segment("Customer Service", 320.0, 610.0, 8.0, 72.0),
        segment("-example.org/help", 410.0, 610.0, 8.0, 74.0),
    ];
    segments.extend(anchor_rows(
        70.0,
        304.0,
        &[(590.0, "below"), (570.0, "final")],
    ));

    split_segments_at_column_gaps(520.0, &mut segments);

    for text in ["ABC", "Customer Service", "-example.org/help"] {
        assert_text_y(&segments, text, 620.0);
    }
}

#[test]
fn splits_embedded_contents_leader_page_number_to_right_anchor() {
    let mut segments = leader_page_anchors();
    segments.insert(
        2,
        segment(
            "B.4 Test conditions for video-cassette recorders ................................. OU",
            84.0,
            620.0,
            10.0,
            440.0,
        ),
    );

    split_segments_at_column_gaps(596.0, &mut segments);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "B.4 Test conditions for video-cassette recorders"));
    assert!(segments
        .iter()
        .any(|segment| segment.text.starts_with("...")));
    let page = segments
        .iter()
        .find(|segment| segment.text == "28" && (segment.y - 620.0).abs() <= 1.0)
        .expect("expected decoded page number");
    assert!((page.x - 512.0).abs() <= 1.0);
}

#[test]
fn splits_joined_contents_leader_line_to_right_anchor() {
    let mut segments = leader_page_anchors();
    for (index, segment) in [
        segment("B.4", 84.0, 620.0, 10.0, 18.0),
        segment(
            "Test conditions for video-cassette recorders",
            120.0,
            620.0,
            10.0,
            190.0,
        ),
        segment(
            "................................. 28",
            300.0,
            620.0,
            10.0,
            220.0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        segments.insert(index + 2, segment);
    }

    split_segments_at_column_gaps(596.0, &mut segments);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "B.4 Test conditions for video-cassette recorders"));
    let page = segments
        .iter()
        .find(|segment| segment.text == "28" && (segment.y - 620.0).abs() <= 1.0)
        .expect("expected right anchored page number");
    assert!((page.x - 512.0).abs() <= 1.0);
    assert!(!segments.iter().any(|segment| segment
        .text
        .contains("................................. 28")));
}

#[test]
fn keeps_wide_segment_without_nearby_column_band_unsplit() {
    let mut segments = vec![
        segment("Left anchor", 70.0, 560.0, 8.0, 80.0),
        segment("Right anchor", 304.0, 560.0, 8.0, 90.0),
        segment("Left anchor two", 70.0, 540.0, 8.0, 80.0),
        segment("Right anchor two", 304.0, 540.0, 8.0, 90.0),
        segment(
            "The technical content is kept under review by the committee and Please read updates",
            70.0,
            650.0,
            8.0,
            365.0,
        ),
        segment("Left anchor three", 70.0, 520.0, 8.0, 80.0),
        segment("Right anchor three", 304.0, 520.0, 8.0, 90.0),
    ];

    split_segments_at_column_gaps(520.0, &mut segments);

    assert_eq!(
        segments
            .iter()
            .filter(|segment| segment.text.contains("technical content"))
            .count(),
        1
    );
    assert_eq!(segments.len(), 7);
}
