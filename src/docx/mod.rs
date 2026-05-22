mod package;
mod relationships;
mod xml;

use std::collections::HashMap;

use crate::{Block, Document, SourceFormat};
use crate::{ConversionWarning, ConvertError};

use package::DocxPackage;
use relationships::Relationships;

pub fn docx_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    let mut package = DocxPackage::open(bytes)?;
    let document_xml = package.read_required_string("word/document.xml")?;
    let relationships = package
        .read_optional_string("word/_rels/document.xml.rels")?
        .map(|xml| Relationships::parse(&xml))
        .transpose()?
        .unwrap_or_default();

    let parsed = xml::parse_document(&document_xml, &relationships)?;
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Docx);
    document.metadata.title = first_heading_title(&parsed.blocks);
    document.blocks = parsed.blocks;
    document.assets = parsed.assets;
    document.warnings = parsed.warnings;
    add_missing_asset_warnings(&mut document, package.media_paths()?);
    Ok(document)
}

fn first_heading_title(blocks: &[Block]) -> Option<String> {
    blocks.iter().find_map(|block| match block {
        Block::Heading(heading) if heading.level == 1 => Some(xml::plain_text(&heading.content)),
        _ => None,
    })
}

fn add_missing_asset_warnings(document: &mut Document, media_paths: Vec<String>) {
    let referenced = document
        .assets
        .iter()
        .map(|asset| (&asset.path, true))
        .collect::<HashMap<_, _>>();

    for path in media_paths {
        if !referenced.contains_key(&path) {
            document.warnings.push(ConversionWarning {
                message: format!("DOCX media part was present but not referenced: {path}"),
                source: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_document_xml() {
        let bytes = empty_docx();
        let error = docx_to_document(&bytes).unwrap_err();

        assert!(matches!(
            error,
            ConvertError::MissingPart("word/document.xml")
        ));
    }

    fn empty_docx() -> Vec<u8> {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;

        let mut bytes = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut bytes);
            zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(br#"<?xml version="1.0"?>"#).unwrap();
            zip.finish().unwrap();
        }
        bytes.into_inner()
    }
}
