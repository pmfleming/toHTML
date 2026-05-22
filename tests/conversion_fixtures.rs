use std::io::{Cursor, Write};

use flate2::{write::ZlibEncoder, Compression};
use tohtml::{docx_to_document, markdown_to_document, pdf_to_document, render_html};
use zip::write::SimpleFileOptions;

#[test]
fn markdown_fixture_renders_gfm_features() {
    let markdown = "# Fixture\n\n- [x] done\n- [ ] next\n\n| Name | Count |\n| --- | ---: |\n| A | 3 |\n\n~~old~~ **new**";

    let html = render_html(&markdown_to_document(markdown));

    assert_contains_all(
        &html,
        &[
            "<title>Fixture</title>",
            "<input type=\"checkbox\" disabled checked>",
            "<input type=\"checkbox\" disabled>",
            "<table>",
            "<th>Name</th>",
            "<td style=\"text-align: right\">3</td>",
            "<del>old</del>",
            "<strong>new</strong>",
        ],
    );
}

#[test]
fn docx_fixture_renders_core_structures() {
    let html = render_html(&docx_to_document(&docx_fixture()).unwrap());

    assert_contains_all(
        &html,
        &[
            "<title>Docx Fixture</title>",
            "<h1>Docx Fixture</h1>",
            "<p>Paragraph text</p>",
            "<ul>",
            "<li>List item</li>",
            "<table>",
            "<td>Cell A</td>",
            "<img src=\"word/media/image1.png\" alt=\"\">",
        ],
    );
}

#[test]
fn pdf_fixture_renders_selectable_text() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Selectable PDF) Tj ET
endstream
endobj
%%EOF"#;

    let html = render_html(&pdf_to_document(pdf).unwrap());

    assert!(html.contains("<p>Selectable PDF</p>"));
}

#[test]
fn pdf_fixture_renders_placeholder_for_non_extractable_page() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page >> endobj
%%EOF"#;

    let html = render_html(&pdf_to_document(pdf).unwrap());

    assert_contains_all(
        &html,
        &[
            "data-page-placeholder",
            "data-reason=\"non-extractable\"",
            "PDF contained no selectable text",
        ],
    );
}

#[test]
fn pdf_fixture_decodes_flate_streams() {
    let content = b"BT /F1 12 Tf 72 720 Td (Compressed PDF) Tj ET";
    let compressed = flate_bytes(content);
    let mut pdf = Vec::new();
    write!(
        pdf,
        "%PDF-1.4\n1 0 obj << /Type /Page /Contents 2 0 R >> endobj\n2 0 obj << /Length {} /Filter /FlateDecode >>\nstream\n",
        compressed.len()
    )
    .unwrap();
    pdf.extend_from_slice(&compressed);
    pdf.extend_from_slice(b"\nendstream\nendobj\n%%EOF");

    let html = render_html(&pdf_to_document(&pdf).unwrap());

    assert!(html.contains("Compressed PDF"));
}

#[test]
fn pdf_fixture_renders_tagged_actual_text_heading() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 92 >>
stream
BT /H1 << /ActualText (Semantic Heading) >> BDC (xxxx) Tj EMC ET
endstream
endobj
%%EOF"#;

    let html = render_html(&pdf_to_document(pdf).unwrap());

    assert!(html.contains("<h1>Semantic Heading</h1>"));
}

#[test]
fn pdf_fixture_uses_struct_tree_role_when_bdc_tag_is_generic() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /StructTreeRoot 5 0 R /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Contents 4 0 R >> endobj
4 0 obj << /Length 80 >>
stream
BT /Span << /MCID 0 >> BDC /F1 16 Tf 72 720 Td (Promoted Heading) Tj EMC ET
endstream
endobj
5 0 obj << /Type /StructTreeRoot /K 6 0 R >> endobj
6 0 obj << /Type /StructElem /S /H1 /K [0] >> endobj
%%EOF"#;

    let html = render_html(&pdf_to_document(pdf).unwrap());

    assert!(
        html.contains("<h1>Promoted Heading</h1>"),
        "html was: {html}"
    );
}

#[test]
fn pdf_fixture_prefers_latest_object_revision_from_incremental_update() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R >> endobj
2 0 obj << /Length 40 >>
stream
BT /F1 12 Tf 72 720 Td (Old text) Tj ET
endstream
endobj
2 0 obj << /Length 40 >>
stream
BT /F1 12 Tf 72 720 Td (New text) Tj ET
endstream
endobj
%%EOF"#;

    let html = render_html(&pdf_to_document(pdf).unwrap());

    assert!(html.contains("New text"), "html was: {html}");
    assert!(!html.contains("Old text"));
}

#[test]
fn pdf_fixture_decodes_multi_byte_to_unicode_cmap() {
    let content = b"BT /F1 12 Tf 72 720 Td <000100020003> Tj ET";
    let cmap = b"beginbfchar\n<0001> <0048>\n<0002> <0069>\n<0003> <0021>\nendbfchar\n";
    let mut pdf = Vec::new();
    write!(
        pdf,
        "%PDF-1.4\n\
1 0 obj << /Type /Page /Contents 2 0 R /Resources << /Font << /F1 3 0 R >> >> >> endobj\n\
2 0 obj << /Length {content_len} >>\nstream\n",
        content_len = content.len()
    )
    .unwrap();
    pdf.extend_from_slice(content);
    write!(
        pdf,
        "\nendstream\nendobj\n\
3 0 obj << /Type /Font /Subtype /Type0 /ToUnicode 4 0 R >> endobj\n\
4 0 obj << /Length {cmap_len} >>\nstream\n",
        cmap_len = cmap.len()
    )
    .unwrap();
    pdf.extend_from_slice(cmap);
    pdf.extend_from_slice(b"\nendstream\nendobj\n%%EOF");

    let html = render_html(&pdf_to_document(&pdf).unwrap());

    assert!(html.contains("Hi!"), "html was: {html}");
}

#[test]
fn pdf_fixture_warns_for_image_only_page() {
    let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R /Resources << /XObject << /Im0 3 0 R >> >> >> endobj
2 0 obj << /Length 20 >>
stream
q 100 0 0 100 0 0 cm /Im0 Do Q
endstream
endobj
3 0 obj << /Type /XObject /Subtype /Image /Width 100 /Height 100 >> endobj
%%EOF"#;

    let document = pdf_to_document(pdf).unwrap();
    let html = render_html(&document);

    assert!(html.contains("data-page-placeholder"), "html was: {html}");
    assert!(document
        .warnings
        .iter()
        .any(|warning| warning.message.contains("image content")));
}

#[test]
fn pdf_fixture_removes_repeated_page_header() {
    let mut pdf = String::from("%PDF-1.4\n");
    pdf.push_str("1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n");
    pdf.push_str("2 0 obj << /Type /Pages /Kids [3 0 R 5 0 R 7 0 R 9 0 R] /Count 4 >> endobj\n");
    // Four pages, each with the same running header at top and unique body below.
    let mut object_id = 3;
    for body in ["Body One", "Body Two", "Body Three", "Body Four"] {
        pdf.push_str(&format!(
            "{object_id} 0 obj << /Type /Page /Parent 2 0 R /Contents {} 0 R >> endobj\n",
            object_id + 1
        ));
        let stream = format!("BT /F1 12 Tf 72 760 Td (Repeated Header) Tj 0 -40 Td ({body}) Tj ET");
        pdf.push_str(&format!(
            "{} 0 obj << /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            object_id + 1,
            stream.len(),
            stream
        ));
        object_id += 2;
    }
    pdf.push_str("%%EOF");

    let document = pdf_to_document(pdf.as_bytes()).unwrap();
    let html = render_html(&document);
    let header_count = html.matches("Repeated Header").count();

    assert!(
        header_count <= 1,
        "header should be removed; html was: {html}"
    );
    assert!(html.contains("Body One"));
    assert!(html.contains("Body Four"));
}

fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected rendered HTML to contain {needle:?}\n{haystack}"
        );
    }
}

fn docx_fixture() -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut bytes);
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml().as_bytes()).unwrap();
        zip.start_file("word/_rels/document.xml.rels", options)
            .unwrap();
        zip.write_all(relationships_xml().as_bytes()).unwrap();
        zip.start_file("word/media/image1.png", options).unwrap();
        zip.write_all(b"fake image bytes").unwrap();
        zip.finish().unwrap();
    }
    bytes.into_inner()
}

fn flate_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn document_xml() -> &'static str {
    r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
        xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
        xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
      <w:body>
        <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Docx Fixture</w:t></w:r></w:p>
        <w:p><w:r><w:t>Paragraph text</w:t></w:r></w:p>
        <w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>List item</w:t></w:r></w:p>
        <w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell A</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        <w:p><w:r><w:drawing><a:blip r:embed="rId1"/></w:drawing></w:r></w:p>
      </w:body>
    </w:document>"#
}

fn relationships_xml() -> &'static str {
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
      <Relationship Id="rId1" Type="image" Target="media/image1.png"/>
    </Relationships>"#
}
