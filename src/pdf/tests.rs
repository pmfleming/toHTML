use super::*;

fn pdf_with_info_object(info_object: &str) -> String {
    format!(
        r#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Contents 4 0 R >> endobj
4 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Body) Tj ET
endstream
endobj
5 0 obj {info_object} endobj
%%EOF"#
    )
}

#[test]
fn extracts_simple_uncompressed_text_stream() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Hello PDF) Tj ET
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert!(matches!(&document.blocks[0], Block::Paragraph(_)));
    assert!(crate::render_html(&document).contains("Hello PDF"));
    assert!(document
        .metadata
        .visual_html
        .as_deref()
        .unwrap_or_default()
        .contains("pdf-text-fragment"));
}

#[test]
fn uses_page_local_cmaps_for_reused_font_resource_names() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Resources << /Font << /F1 10 0 R >> >> /Contents 4 0 R >> endobj
4 0 obj << /Length 37 >>
stream
BT /F1 12 Tf 72 720 Td <41> Tj ET
endstream
endobj
5 0 obj << /Type /Page /Parent 2 0 R /Resources << /Font << /F1 20 0 R >> >> /Contents 6 0 R >> endobj
6 0 obj << /Length 37 >>
stream
BT /F1 12 Tf 72 720 Td <41> Tj ET
endstream
endobj
10 0 obj << /Type /Font /Subtype /Type0 /ToUnicode 11 0 R >> endobj
11 0 obj << /Length 220 >>
stream
/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def /CMapName /Adobe-Identity-UCS def /CMapType 2 def 1 begincodespacerange <00> <FF> endcodespacerange 1 beginbfchar <41> <0050006100670065004F006E0065> endbfchar endcmap CMapName currentdict /CMap defineresource pop end end
endstream
endobj
20 0 obj << /Type /Font /Subtype /Type0 /ToUnicode 21 0 R >> endobj
21 0 obj << /Length 220 >>
stream
/CIDInit /ProcSet findresource begin 12 dict begin begincmap /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def /CMapName /Adobe-Identity-UCS def /CMapType 2 def 1 begincodespacerange <00> <FF> endcodespacerange 1 beginbfchar <41> <005000610067006500540077006F> endbfchar endcmap CMapName currentdict /CMap defineresource pop end end
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();
    let html = crate::render_html(&document);

    assert!(html.contains("PageOne"));
    assert!(html.contains("PageTwo"));
}

#[test]
fn decodes_simple_font_encoding_differences_without_to_unicode() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >> endobj
4 0 obj << /Length 37 >>
stream
BT /F1 12 Tf 72 720 Td <41424320> Tj ET
endstream
endobj
5 0 obj << /Type /Font /Subtype /Type1 /Encoding << /BaseEncoding /WinAnsiEncoding /Differences [65 /T 66 /h 67 /e] >> >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();
    let html = crate::render_html(&document);

    assert!(html.contains("The"));
}

#[test]
fn creates_placeholder_for_non_extractable_pdf() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert!(matches!(document.blocks[0], Block::PagePlaceholder(_)));
    assert!(document
        .warnings
        .iter()
        .any(|warning| warning.message.contains("no selectable text")));
}

#[test]
fn warns_when_pdf_contains_image_content() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Hello PDF) Tj ET
endstream
endobj
3 0 obj << /Subtype/Image /Width 1 /Height 1 >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert_eq!(document.warnings.len(), 1);
    assert!(document.warnings[0]
        .message
        .contains("OCR is not performed"));
}

#[test]
fn includes_pdf_images_by_default() {
    let document = pdf_to_document(pdf_with_jpeg_xobject()).unwrap();
    let visual_html = document.metadata.visual_html.unwrap_or_default();

    assert!(visual_html.contains("class=\"pdf-image\""));
    assert!(visual_html.contains("src=\"data:image/jpeg;base64,YWJjZA==\""));
}

#[test]
fn excludes_pdf_images_when_requested() {
    let document = pdf_to_document_with_options(
        pdf_with_jpeg_xobject(),
        PdfConversionOptions {
            include_images: false,
        },
    )
    .unwrap();
    let visual_html = document.metadata.visual_html.unwrap_or_default();

    assert!(!visual_html.contains("class=\"pdf-image\""));
}

#[test]
fn positions_embedded_pdf_images() {
    let document = pdf_to_document_with_options(
        pdf_with_jpeg_xobject(),
        PdfConversionOptions {
            include_images: true,
        },
    )
    .unwrap();
    let visual_html = document.metadata.visual_html.unwrap_or_default();

    assert!(visual_html.contains("class=\"pdf-image\""));
    assert!(visual_html.contains("src=\"data:image/jpeg;base64,YWJjZA==\""));
    assert!(visual_html.contains("left:10.00pt;top:240.00pt;width:50.00pt;height:40.00pt"));
}

#[test]
fn converts_matching_uri_annotation_text_to_link() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R /Annots [3 0 R] >> endobj
2 0 obj << /Length 60 >>
stream
BT /F1 12 Tf 72 720 Td (https://example.test) Tj ET
endstream
endobj
3 0 obj << /Subtype /Link /Rect [72 710 200 730] /A << /S /URI /URI (https://example.test) >> >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();
    let html = crate::render_html(&document);

    assert!(html.contains("<a href=\"https://example.test\">https://example.test</a>"));
    assert!(html.contains("class=\"pdf-link-overlay\""));
    assert!(html.contains("href=\"https://example.test\" aria-label=\"https://example.test\""));
}

#[test]
fn links_visible_uri_without_scheme_inside_table_cell() {
    let mut blocks = vec![Block::Table(crate::Table {
        rows: vec![crate::TableRow {
            cells: vec![crate::TableCell::text(
                "see www.example.test for details",
                false,
            )],
            source: None,
        }],
        caption: None,
        source: None,
    })];

    assert!(links::link_uri_in_blocks(
        &mut blocks,
        "http://www.example.test/"
    ));

    let html = crate::render_html(&Document {
        blocks,
        ..Document::new()
    });
    assert!(html.contains("<a href=\"http://www.example.test/\">www.example.test</a>"));
}

#[test]
fn links_visible_email_address_for_mailto_annotation() {
    let mut blocks = vec![Block::paragraph("contact Person.Example@example.test")];

    assert!(links::link_uri_in_blocks(
        &mut blocks,
        "mailto:Person.Example@example.test"
    ));

    let html = crate::render_html(&Document {
        blocks,
        ..Document::new()
    });
    assert!(html.contains(
        "<a href=\"mailto:Person.Example@example.test\">Person.Example@example.test</a>"
    ));
}

#[test]
fn ignores_binary_false_positive_uri_markers() {
    let bytes = b"/URI (https://example.test) stream \0\0 /URI (\0\xffnot-a-link) endstream";

    assert_eq!(
        links::link_annotation_uris(bytes),
        vec!["https://example.test"]
    );
}

#[test]
fn preserves_catalog_language() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Lang (nl-NL) >> endobj
2 0 obj << /Type /Page /Contents 3 0 R >> endobj
3 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Hallo) Tj ET
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert_eq!(document.metadata.language.as_deref(), Some("nl-NL"));
}

#[test]
fn decodes_utf16be_catalog_language() {
    // /Lang (þÿ\0e\0n\0-\0U\0S) — UTF-16BE BOM + ASCII as 16-bit code units.
    let pdf: &[u8] = b"%PDF-1.4\n\
1 0 obj << /Type /Catalog /Lang (\xfe\xff\x00e\x00n\x00-\x00U\x00S) >> endobj\n\
2 0 obj << /Type /Page /Contents 3 0 R >> endobj\n\
3 0 obj << /Length 42 >>\nstream\nBT /F1 12 Tf 72 720 Td (Hi) Tj ET\nendstream\nendobj\n%%EOF";

    let document = pdf_to_document(pdf).unwrap();

    assert_eq!(document.metadata.language.as_deref(), Some("en-US"));
}

#[test]
fn extracts_document_title_from_info_dictionary() {
    let pdf = pdf_with_info_object("<< /Title (Sample Report) /Producer (toHTML tests) >>");

    let document = pdf_to_document(pdf.as_bytes()).unwrap();

    assert_eq!(document.metadata.title.as_deref(), Some("Sample Report"));
}

#[test]
fn removes_office_prefix_from_document_title() {
    let pdf = pdf_with_info_object("<< /Title (Microsoft Word - Project Proposal) >>");

    let document = pdf_to_document(pdf.as_bytes()).unwrap();

    assert_eq!(document.metadata.title.as_deref(), Some("Project Proposal"));
}

#[test]
fn rejects_generated_filename_document_title() {
    let pdf = pdf_with_info_object("<< /Title (ExampleWorkbook.xlsx) >>");

    let document = pdf_to_document(pdf.as_bytes()).unwrap();

    assert_eq!(document.metadata.title, None);
}

#[test]
fn rejects_generic_watermark_document_title() {
    let pdf = pdf_with_info_object("<< /Title (English) >>");

    let document = pdf_to_document(pdf.as_bytes()).unwrap();

    assert_eq!(document.metadata.title, None);
}

#[test]
fn decodes_utf16be_document_title() {
    // /Title (þÿ\0H\0i) — "Hi" as UTF-16BE.
    let pdf: &[u8] = b"%PDF-1.4\n\
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n\
3 0 obj << /Type /Page /Parent 2 0 R /Contents 4 0 R >> endobj\n\
4 0 obj << /Length 42 >>\nstream\nBT /F1 12 Tf 72 720 Td (Body) Tj ET\nendstream\nendobj\n\
5 0 obj << /Title (\xfe\xff\x00H\x00i) >> endobj\n%%EOF";

    let document = pdf_to_document(pdf).unwrap();

    assert_eq!(document.metadata.title.as_deref(), Some("Hi"));
}

#[test]
fn emits_page_break_between_pdf_pages() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Contents 4 0 R >> endobj
4 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (First) Tj ET
endstream
endobj
5 0 obj << /Type /Page /Parent 2 0 R /Contents 6 0 R >> endobj
6 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Second) Tj ET
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert!(document
        .blocks
        .iter()
        .any(|block| matches!(block, Block::PageBreak(_))));
}

#[test]
fn carries_text_state_across_page_content_streams() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Contents [4 0 R 5 0 R] >> endobj
4 0 obj << /Length 16 >>
stream
BT /F1 1 Tf ET
endstream
endobj
5 0 obj << /Length 46 >>
stream
BT 10 0 0 10 72 720 Tm (Body) Tj ET
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();
    let visual_html = document.metadata.visual_html.unwrap_or_default();

    assert!(visual_html.contains("font-size:10.00pt"));
    assert!(!visual_html.contains("font-size:48.00pt"));
}

#[test]
fn reports_encryption_as_extraction_risk() {
    let pdf = br#"%PDF-1.4
trailer << /Encrypt 4 0 R >>
1 0 obj << /Type /Page >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert!(document
        .warnings
        .iter()
        .any(|warning| warning.message.contains("encryption")));
}

#[test]
fn reports_unsupported_content_stream_filter_as_page_warning() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 4 /Filter /LZWDecode >>
stream
abcd
endstream
endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();

    assert!(document.warnings.iter().any(|warning| {
        warning
            .message
            .contains("Page 1: unsupported PDF stream filter LZWDecode")
    }));
    assert!(document
        .blocks
        .iter()
        .any(|block| matches!(block, Block::PagePlaceholder(_))));
}

fn pdf_with_jpeg_xobject() -> &'static [u8] {
    br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 300] /Resources << /XObject << /Im1 5 0 R >> >> /Contents 4 0 R >> endobj
4 0 obj << >>
stream
q 50 0 0 40 10 20 cm /Im1 Do Q
endstream
endobj
5 0 obj << /Type /XObject /Subtype /Image /Width 2 /Height 2 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode >>
stream
abcd
endstream
endobj
%%EOF"#
}
