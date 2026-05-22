use super::*;

use std::env;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use tohtml::{docx_to_document, render_html};
use zip::write::SimpleFileOptions;

#[test]
fn extracts_docx_assets_next_to_output() {
    let root = temp_root();
    let output_dir = root.join("out");
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("output.html");
    let input = docx_fixture();
    let mut document = docx_to_document(&input).unwrap();

    write(
        Format::Docx,
        &input,
        &mut document,
        Path::new("assets"),
        Some(&output),
    )
    .unwrap();

    let html = render_html(&document);
    assert!(html.contains("<img src=\"assets/image1.png\" alt=\"\">"));
    assert_eq!(
        fs::read(output_dir.join("assets").join("image1.png")).unwrap(),
        b"fake image bytes"
    );

    fs::remove_dir_all(root).unwrap();
}

fn temp_root() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("tohtml-cli-assets-test-{suffix}"))
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

fn document_xml() -> &'static str {
    r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
        xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
        xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
      <w:body>
        <w:p><w:r><w:drawing><a:blip r:embed="rId1"/></w:drawing></w:r></w:p>
      </w:body>
    </w:document>"#
}

fn relationships_xml() -> &'static str {
    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
      <Relationship Id="rId1" Type="image" Target="media/image1.png"/>
    </Relationships>"#
}
