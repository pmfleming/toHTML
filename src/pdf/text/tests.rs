use std::collections::HashMap;

use super::*;

#[test]
fn ignores_parenthesized_binary_without_text_operator() {
    let stream = b"q 100 0 0 100 0 0 cm /Im1 Do (binary-ish junk) Q";

    assert_eq!(extract_text(stream), None);
}

#[test]
fn extracts_text_showing_operator_operands() {
    let stream = b"BT /F1 12 Tf 72 720 Td (Hello PDF) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Hello PDF"));
}

#[test]
fn extracts_text_array_operands() {
    let stream = b"BT [(Hello) 120 (PDF)] TJ ET";

    assert_eq!(extract_text(stream).as_deref(), Some("HelloPDF"));
}

#[test]
fn inserts_space_for_large_text_array_gaps() {
    let stream = b"BT [(Hello) -250 (PDF)] TJ ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Hello PDF"));
}

#[test]
fn decodes_hex_text_operands() {
    let stream = b"BT <48656c6c6f> Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Hello"));
}

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
fn decodes_text_with_active_font_cmap() {
    let cmap = super::super::cmap::CMap::parse(
        br#"
        beginbfchar
        <01> <0041>
        <02> <0042>
        endbfchar
        "#,
    );
    let font_cmaps = HashMap::from([("F1".to_string(), cmap)]);
    let stream = b"BT /F1 12 Tf <0102> Tj ET";

    let segments = extract_segments_with_fonts(stream, &font_cmaps, &HashMap::new());

    assert_eq!(segments_to_text(&segments), "AB");
}

#[test]
fn keeps_structural_table_values() {
    let stream = b"BT ([1..1]) Tj 40 0 Td (+) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("[1..1] +"));
}

#[test]
fn quote_operator_moves_to_next_line_before_showing_text() {
    let stream = b"BT 1 0 0 1 10 100 Tm 20 TL (One) Tj (Two) ' ET";

    let segments = extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new());

    assert!(segments[1].y < segments[0].y);
    assert_eq!(segments_to_text(&segments), "One Two");
}

#[test]
fn skips_invisible_text_rendering_modes() {
    let stream = b"BT 3 Tr (Hidden) Tj 0 Tr (Visible) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Visible"));
}
