use std::collections::HashMap;

use super::super::*;

#[test]
fn decodes_text_with_active_font_cmap() {
    let cmap = super::super::super::cmap::CMap::parse(
        br#"
        beginbfchar
        <01> <0041>
        <02> <0042>
        endbfchar
        "#,
    );
    let font_cmaps = HashMap::from([("F1".to_string(), cmap)]);
    let stream = b"BT /F1 12 Tf <0102> Tj ET";

    let segments =
        extract_segments_with_fonts(stream, &font_cmaps, &HashMap::new(), &HashMap::new());

    assert_eq!(segments_to_text(&segments), "AB");
}

#[test]
fn keeps_symbol_font_task_markers_from_active_cmap() {
    let cmap = super::super::super::cmap::CMap::parse(
        br#"
        beginbfchar
        <0039> <2713>
        <0087> <F070>
        endbfchar
        "#,
    );
    let font_cmaps = HashMap::from([("F11".to_string(), cmap)]);
    let stream = b"BT /F11 24 Tf <0087> Tj 28 0 Td <0039> Tj ET";

    let segments =
        extract_segments_with_fonts(stream, &font_cmaps, &HashMap::new(), &HashMap::new());

    assert_eq!(segments_to_text(&segments), "□ ✓");
}

#[test]
fn keeps_structural_table_values() {
    let stream = b"BT ([1..1]) Tj 40 0 Td (+) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("[1..1] +"));
}

#[test]
fn skips_probable_symbol_font_noise() {
    let stream = b"BT (}uW) Tj 20 0 Td (Customer Name) Tj 0 -20 Td (Dvo du v }v]]}vW) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Customer Name"));
}

#[test]
fn keeps_pdfdoc_bullet_marker() {
    let stream = b"BT (\x80) Tj 20 0 Td (Product Feature) Tj ET";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert_eq!(segments[0].text, "•");
    assert_eq!(segments_to_text(&segments), "• Product Feature");
}

#[test]
fn keeps_split_schema_cardinality_fragments_literal() {
    let stream = b"BT ([0 - 9]{1) Tj 40 0 Td (,15}) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("[0 - 9]{1,15}"));
}

#[test]
fn quote_operator_moves_to_next_line_before_showing_text() {
    let stream = b"BT 1 0 0 1 10 100 Tm 20 TL (One) Tj (Two) ' ET";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert!(segments[1].y < segments[0].y);
    assert_eq!(segments_to_text(&segments), "One Two");
}

#[test]
fn applies_flipped_current_transformation_matrix_to_line_order() {
    let stream = b"q 1 0 0 -1 0 792 cm BT /F1 12 Tf 72 72 Td (Top) Tj 0 40 Td (Lower) Tj ET Q";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert!(segments[0].y > segments[1].y);
    assert_eq!(segments_to_text(&segments), "Top Lower");
}

#[test]
fn applies_current_transformation_matrix_scale_to_font_metrics() {
    let stream = b"q 0.125 0 0 0.125 0 0 cm BT /F1 96 Tf 576 576 Td (Scaled) Tj ET Q";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert!((segments[0].font_size - 12.0).abs() < 0.1);
    assert!((segments[0].width - 32.4).abs() < 0.1);
    assert!((segments[0].x - 72.0).abs() < 0.1);
}

#[test]
fn records_text_matrix_rotation() {
    let stream = b"BT 0 1 -1 0 100 100 Tm (Rotated) Tj ET";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert!((segments[0].rotation - 90.0).abs() < 0.1);
}

#[test]
fn skips_invisible_text_rendering_modes() {
    let stream = b"BT 3 Tr (Hidden) Tj 0 Tr (Visible) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Visible"));
}

#[test]
fn preserves_marked_content_role_on_segments() {
    let stream = b"BT /H1 << /MCID 0 >> BDC (Tagged Heading) Tj EMC ET";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert_eq!(segments[0].role.as_deref(), Some("H1"));
}

#[test]
fn uses_actual_text_for_marked_content() {
    let stream = b"BT /Span << /ActualText (replacement) >> BDC (xxxxx) Tj EMC ET";

    assert_eq!(extract_text(stream).as_deref(), Some("replacement"));
}

#[test]
fn skips_artifact_marked_content() {
    let stream = b"BT /Artifact BMC (Page 1) Tj EMC (Body) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("Body"));
}

#[test]
fn keeps_artifact_text_segments_for_visual_rendering() {
    let stream = b"BT /Artifact BMC (colour inside) Tj EMC ET";

    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());

    assert_eq!(segments[0].text, "colour inside");
    assert_eq!(segments[0].role.as_deref(), Some("Artifact"));
    assert_eq!(extract_text(stream), None);
}
