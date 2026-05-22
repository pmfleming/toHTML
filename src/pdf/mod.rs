mod cmap;
mod fonts;
mod hex;
mod layout;
#[cfg(test)]
mod layout_tests;
mod links;
mod object;
mod postprocess;
mod streams;
mod struct_tree;
mod text;

use crate::ConvertError;
use crate::{Block, ConversionWarning, Document, PageBreak};
use crate::{PagePlaceholder, PlaceholderReason, SourceFormat};

use object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};

pub fn pdf_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    let extraction = streams::document_pages(bytes)?;
    let font_cmaps = cmap::font_cmaps(bytes)?;
    let font_metrics = fonts::font_metrics(bytes);
    let objects = PdfObjects::parse(bytes);
    let struct_roles = struct_tree::role_map(&objects);
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Pdf);
    document.metadata.title = document_title(&objects);
    document.metadata.language = document_language(&objects);
    document.warnings.extend(
        extraction
            .warnings
            .into_iter()
            .map(|message| ConversionWarning {
                message,
                source: None,
            }),
    );

    let total_pages = extraction.pages.len();
    for (page_index, page) in extraction.pages.iter().enumerate() {
        let mut page_blocks = Vec::new();
        for stream in &page.streams {
            let segments = text::extract_segments_with_fonts(
                stream,
                &font_cmaps,
                &font_metrics,
                &struct_roles,
            );
            page_blocks.extend(layout::blocks_from_segments(&segments));
        }
        if page_blocks.is_empty() {
            page_blocks.push(Block::PagePlaceholder(PagePlaceholder {
                page_number: Some(page.page_number),
                reason: PlaceholderReason::NonExtractable,
                source: None,
            }));
        }
        document.blocks.extend(page_blocks);
        if total_pages > 1 && page_index + 1 < total_pages {
            document.blocks.push(Block::PageBreak(PageBreak {
                page_number: Some(page.page_number),
                source: None,
            }));
        }
    }
    document.blocks = postprocess::blocks(document.blocks, extraction.page_count);

    if document.blocks.is_empty() {
        add_empty_pdf_placeholder(&mut document, extraction.page_count);
    }
    if !document
        .blocks
        .iter()
        .any(|block| !matches!(block, Block::PagePlaceholder(_)))
        && !document
            .warnings
            .iter()
            .any(|warning| warning.message.contains("PDF contained no selectable text"))
    {
        document.warnings.push(ConversionWarning {
            message: "PDF contained no selectable text in supported content streams".to_string(),
            source: None,
        });
    }
    add_image_text_warning(&mut document, bytes);
    links::apply_detected_links(&mut document.blocks, &mut document.warnings, bytes);

    Ok(document)
}

fn document_title(objects: &PdfObjects) -> Option<String> {
    let title = info_dictionary(objects)?.string_bytes("Title")?;
    let decoded = text::decode_string(title);
    let trimmed = decoded.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn document_language(objects: &PdfObjects) -> Option<String> {
    let bytes = catalog_dictionary(objects)?.string_bytes("Lang")?;
    let decoded = text::decode_string(bytes);
    let trimmed = decoded.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn catalog_dictionary(objects: &PdfObjects) -> Option<&PdfDictionary> {
    objects
        .values()
        .find(|object| object.type_name() == Some("Catalog"))
        .and_then(|object| object.dictionary())
}

fn info_dictionary(objects: &PdfObjects) -> Option<&PdfDictionary> {
    objects
        .values()
        .filter_map(|object| object.dictionary())
        .find(|dictionary| {
            dictionary.type_name_is_none()
                && (dictionary.contains_key("Title")
                    || dictionary.contains_key("Author")
                    || dictionary.contains_key("Producer")
                    || dictionary.contains_key("Creator"))
        })
}

trait PdfDictionaryTypeCheck {
    fn type_name_is_none(&self) -> bool;
}

impl PdfDictionaryTypeCheck for PdfDictionary {
    fn type_name_is_none(&self) -> bool {
        !matches!(self.get("Type"), Some(PdfValue::Name(_)))
    }
}

fn add_empty_pdf_placeholder(document: &mut Document, pages: usize) {
    for page in 1..=pages.max(1) {
        document
            .blocks
            .push(Block::PagePlaceholder(PagePlaceholder {
                page_number: Some(page as u32),
                reason: PlaceholderReason::NonExtractable,
                source: None,
            }));
    }
    document.warnings.push(ConversionWarning {
        message: "PDF contained no selectable text in supported content streams".to_string(),
        source: None,
    });
}

fn add_image_text_warning(document: &mut Document, bytes: &[u8]) {
    if !has_image_xobject(bytes) {
        return;
    }

    document.warnings.push(ConversionWarning {
        message:
            "PDF contains image content that may include non-selectable text; OCR is not performed"
                .to_string(),
        source: None,
    });
}

fn has_image_xobject(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    text.contains("/Subtype /Image") || text.contains("/Subtype/Image")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn converts_matching_uri_annotation_text_to_link() {
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Page /Contents 2 0 R /Annots [3 0 R] >> endobj
2 0 obj << /Length 60 >>
stream
BT /F1 12 Tf 72 720 Td (https://example.test) Tj ET
endstream
endobj
3 0 obj << /Subtype /Link /A << /S /URI /URI (https://example.test) >> >> endobj
%%EOF"#;

        let document = pdf_to_document(pdf).unwrap();
        let html = crate::render_html(&document);

        assert!(html.contains("<a href=\"https://example.test\">https://example.test</a>"));
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
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /Contents 4 0 R >> endobj
4 0 obj << /Length 42 >>
stream
BT /F1 12 Tf 72 720 Td (Body) Tj ET
endstream
endobj
5 0 obj << /Title (Sample Report) /Producer (toHTML tests) >> endobj
%%EOF"#;

        let document = pdf_to_document(pdf).unwrap();

        assert_eq!(document.metadata.title.as_deref(), Some("Sample Report"));
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
}
