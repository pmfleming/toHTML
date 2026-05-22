mod cmap;
mod fonts;
mod hex;
mod layout;
mod object;
mod postprocess;
mod streams;
mod text;

use crate::ConvertError;
use crate::{Block, ConversionWarning, Document};
use crate::{PagePlaceholder, PlaceholderReason, SourceFormat};

pub fn pdf_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    let extraction = streams::document_pages(bytes)?;
    let font_cmaps = cmap::font_cmaps(bytes)?;
    let font_metrics = fonts::font_metrics(bytes);
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Pdf);
    document.warnings.extend(
        extraction
            .warnings
            .into_iter()
            .map(|message| ConversionWarning {
                message,
                source: None,
            }),
    );

    for page in &extraction.pages {
        let mut page_blocks = Vec::new();
        for stream in &page.streams {
            let segments = text::extract_segments_with_fonts(stream, &font_cmaps, &font_metrics);
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

    Ok(document)
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
}
