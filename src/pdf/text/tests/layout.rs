use std::collections::HashMap;

use super::super::*;

#[test]
fn joins_nearby_segments_without_forcing_word_breaks() {
    let stream = b"BT 1 0 0 1 10 100 Tm (def) Tj 1 0 Td (ault) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("default"));
}

#[test]
fn inserts_spaces_for_larger_position_gaps() {
    let stream = b"BT 1 0 0 1 10 100 Tm (Default) Tj 70 0 Td (Current) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Default Current"));
}

#[test]
fn relative_text_moves_start_from_line_matrix_not_advanced_cursor() {
    let stream = b"BT 1 0 0 1 10 100 Tm (First) Tj 70 0 Td (Second) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert!((segments[1].x - 80.0).abs() < 0.1);
}

#[test]
fn relative_text_moves_use_active_text_matrix_scale() {
    let stream = b"BT 10 0 0 10 10 100 Tm (First) Tj 7 0 Td (Second) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert!((segments[1].x - 80.0).abs() < 0.1);
}

#[test]
fn horizontal_scaling_applies_to_segment_width() {
    let stream = b"BT /F1 10 Tf 50 Tz 1 0 0 1 10 100 Tm (Wide) Tj 28 0 Td (Gap) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert!(segments[0].width < 10.0);
    assert_eq!(segments_to_text(&segments), "Wide Gap");
}

#[test]
fn records_nonstroking_text_color() {
    let stream = b"BT 0.38 0.52 0.76 rg /F1 12 Tf 10 100 Td (Blue heading) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert_eq!(segments[0].text, "Blue heading");
    assert_eq!(segments[0].color.as_deref(), Some("#6185c2"));
}

#[test]
fn records_nonstroking_gray_text_color_after_colored_fill() {
    let stream = b"0.255 0.765 0.388 rg BT 1 g /F1 40 Tf 36 325 Td (Lighting) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert_eq!(segments[0].text, "Lighting");
    assert_eq!(segments[0].color.as_deref(), Some("#ffffff"));
}

#[test]
fn records_generic_color_space_text_color() {
    let stream = b"BT /CS1 cs 1 scn /F1 12 Tf 10 100 Td (White heading) Tj ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    assert_eq!(segments[0].color.as_deref(), Some("#ffffff"));
}

#[test]
fn keeps_text_array_after_stream_boundary_state_operator() {
    let stream = b"BT /F1 1 Tf 10.98 0 0 10.98 72 630.12 Tm \
        [(Example text continues in the design and )]TJ \
        0.002 Tc \n -0.002 Tw [(man)4.4 (u)0.9 (f)3 (ac)2.9 (t)-0.8 (u)0.9 (r)3.7 (e)]TJ ET";

    let segments = extract_segments_with_fonts(
        stream,
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );

    let manufacture = segments
        .iter()
        .find(|segment| segment.text == "manufacture")
        .unwrap();
    assert!(manufacture.x > 250.0);
    assert!(manufacture.x < 350.0);
}
