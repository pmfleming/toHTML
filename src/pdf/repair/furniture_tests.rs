use super::super::{graphics::RectShape, text};
use super::*;

#[test]
fn splits_two_word_sublabel_across_two_header_columns() {
    let page_height = 842.0;
    let mut segments = vec![
        positioned_text_segment(page_height, "Header", 37.68, 174.58, 24.39),
        positioned_text_segment(page_height, "Command", 83.18, 174.58, 28.44),
        positioned_text_segment(page_height, "Offset", 146.54, 174.58, 24.32),
        positioned_text_segment(page_height, "Data", 203.69, 174.58, 16.24),
        positioned_text_segment(page_height, "Data", 268.97, 174.58, 16.20),
        positioned_text_segment(page_height, "Checksum", 325.87, 174.58, 32.41),
        positioned_text_segment(page_height, "End", 389.62, 174.58, 12.15),
        positioned_text_segment(page_height, "End", 423.46, 174.58, 12.15),
        positioned_text_segment(page_height, "Definition", 486.82, 174.58, 40.59),
        positioned_text_segment(page_height, "Address Length", 141.50, 190.18, 81.41),
    ];

    split_multicolumn_sublabels(page_height, &mut segments);

    let address = segments
        .iter()
        .find(|segment| segment.text == "Address")
        .expect("expected first sublabel");
    let length = segments
        .iter()
        .find(|segment| segment.text == "Length")
        .expect("expected second sublabel");
    assert!((136.0..=150.0).contains(&address.x));
    assert!((194.0..=208.0).contains(&length.x));
    assert_eq!(
        segments
            .iter()
            .filter(|segment| segment.text == "Address Length")
            .count(),
        0
    );
}

#[test]
fn keeps_two_word_text_without_column_anchors_unsplit() {
    let page_height = 842.0;
    let mut segments = vec![positioned_text_segment(
        page_height,
        "Address Length",
        141.50,
        190.18,
        81.41,
    )];

    split_multicolumn_sublabels(page_height, &mut segments);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].text, "Address Length");
}

#[test]
fn tightens_visual_text_widths_before_adjacent_fragments() {
    let mut segments = vec![
        text::TextSegment::new("manufacture".to_string(), 314.0, 600.0, 11.0, 54.0),
        text::TextSegment::new("of LED".to_string(), 348.0, 600.0, 11.0, 40.0),
    ];

    tighten_overlapping_text_widths(&mut segments);

    assert!(segments[0].width < 40.0);
    assert!(segments[0].width >= 29.0);
}

#[test]
fn restores_centered_header_page_number_marker() {
    let page_width = 596.0;
    let page_height = 842.0;
    let mut segments = vec![
        positioned_text_segment(page_height, "Document section header", 70.0, 56.0, 140.0),
        positioned_text_segment(page_height, "23", 291.5, 56.0, 11.0),
    ];

    restore_centered_page_number_markers(page_width, page_height, &mut segments);

    assert_eq!(segments[1].text, "– 23 –");
    let center = segments[1].x + segments[1].width / 2.0;
    assert!((center - 297.0).abs() < 0.1);
}

#[test]
fn keeps_centered_body_numbers_unwrapped() {
    let page_width = 596.0;
    let page_height = 842.0;
    let mut segments = vec![
        positioned_text_segment(page_height, "Section title", 250.0, 260.0, 90.0),
        positioned_text_segment(page_height, "23", 291.5, 280.0, 11.0),
    ];

    restore_centered_page_number_markers(page_width, page_height, &mut segments);

    assert_eq!(segments[1].text, "23");
}

#[test]
fn removes_short_header_hairline_below_right_end_of_long_rule() {
    let page_height = 842.0;
    let mut shapes = vec![
        horizontal_line(page_height, 52.0, 72.0, 490.0),
        horizontal_line(page_height, 476.0, 87.0, 120.0),
    ];

    remove_redundant_header_hairlines(page_height, &mut shapes);

    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].x, 52.0);
}

#[test]
fn keeps_short_header_hairline_without_long_rule_above() {
    let page_height = 842.0;
    let mut shapes = vec![horizontal_line(page_height, 476.0, 87.0, 120.0)];

    remove_redundant_header_hairlines(page_height, &mut shapes);

    assert_eq!(shapes.len(), 1);
}

#[test]
fn keeps_table_hairlines_outside_header_band() {
    let page_height = 842.0;
    let mut shapes = vec![
        horizontal_line(page_height, 52.0, 360.0, 490.0),
        horizontal_line(page_height, 476.0, 374.0, 120.0),
    ];

    remove_redundant_header_hairlines(page_height, &mut shapes);

    assert_eq!(shapes.len(), 2);
}

fn positioned_text_segment(
    page_height: f32,
    text: &str,
    x: f32,
    top: f32,
    width: f32,
) -> text::TextSegment {
    let font_size = 9.0;
    text::TextSegment::new(
        text.to_string(),
        x,
        page_height - top - font_size,
        font_size,
        width,
    )
}

fn horizontal_line(page_height: f32, x: f32, top: f32, width: f32) -> RectShape {
    let height = 0.72;
    RectShape {
        x,
        y: page_height - top - height,
        width,
        height,
        fill: Some("#000000".to_string()),
        stroke: None,
    }
}
