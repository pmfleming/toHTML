mod streams;
mod text;

use crate::ConvertError;
use crate::{Block, ConversionWarning, Document, Inline};
use crate::{PagePlaceholder, Paragraph, PlaceholderReason, SourceFormat};

pub fn pdf_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    let streams = streams::content_streams(bytes)?;
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Pdf);

    for text in streams
        .iter()
        .filter_map(|stream| text::extract_text(stream))
    {
        document.blocks.push(Block::Paragraph(Paragraph {
            content: vec![Inline::Text(text)],
            source: None,
        }));
    }

    if document.blocks.is_empty() {
        add_empty_pdf_placeholder(&mut document, page_count(bytes));
    }

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

fn page_count(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    text.matches("/Type /Page")
        .count()
        .saturating_sub(text.matches("/Type /Pages").count())
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
        assert_eq!(document.warnings.len(), 1);
    }
}
