use super::super::*;

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
fn decodes_pdf_doc_encoding_fallback_bytes() {
    let stream = b"BT (A\x93B\x94) Tj ET";

    assert_eq!(extract_text(stream).as_deref(), Some("AﬁBﬂ"));
}

#[test]
fn rejects_utf16_text_with_binary_control_characters() {
    let stream = b"BT (\xfe\xff\x00A\x00\x00\x00B) Tj ET";

    assert_eq!(extract_text(stream), None);
}

#[test]
fn decodes_shifted_subset_text_when_it_is_more_readable() {
    let stream = b"BT (7KLV $JUHHPHQW VKDOO EH FRQILGHQWLDO) Tj ET";

    assert_eq!(
        extract_text(stream).as_deref(),
        Some("This Agreement shall be confidential")
    );
}

#[test]
fn decodes_shifted_subset_text_with_punctuation_markers() {
    let stream = b"BT (0878$/&21\\),'\\(17,$/,7<) Tj ET";

    assert_eq!(
        super::super::strings::decode_pdf_text_string(b"0878$/&21),'(17,$/,7<"),
        "MUTUALCONFIDENTIALITY"
    );
    assert_eq!(
        extract_text(stream).as_deref(),
        Some("MUTUALCONFIDENTIALITY")
    );
}
