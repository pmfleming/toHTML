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
            "<td>3</td>",
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
