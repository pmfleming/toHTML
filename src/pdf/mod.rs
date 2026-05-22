mod cmap;
mod fonts;
mod hex;
mod layout;
mod object;
mod postprocess;
mod streams;
mod text;

use crate::ConvertError;
use crate::{Block, ConversionWarning, Document, Inline, Link, Paragraph};
use crate::{PagePlaceholder, PlaceholderReason, SourceFormat};

pub fn pdf_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    let extraction = streams::document_pages(bytes)?;
    let font_cmaps = cmap::font_cmaps(bytes)?;
    let font_metrics = fonts::font_metrics(bytes);
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Pdf);
    document.metadata.language = document_language(bytes);
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
    apply_detected_links(&mut document, bytes);

    Ok(document)
}

fn apply_detected_links(document: &mut Document, bytes: &[u8]) {
    for uri in link_annotation_uris(bytes) {
        if !link_uri_in_blocks(&mut document.blocks, &uri) {
            document.warnings.push(ConversionWarning {
                message: format!(
                    "PDF link annotation target {uri} could not be associated with extracted text"
                ),
                source: None,
            });
        }
    }
}

fn document_language(bytes: &[u8]) -> Option<String> {
    let marker = find_bytes(bytes, b"/Lang", 0)? + b"/Lang".len();
    literal_after(bytes, marker)
}

fn link_uri_in_blocks(blocks: &mut [Block], uri: &str) -> bool {
    blocks.iter_mut().any(|block| match block {
        Block::Paragraph(paragraph) => link_uri_in_paragraph(paragraph, uri),
        _ => false,
    })
}

fn link_uri_in_paragraph(paragraph: &mut Paragraph, uri: &str) -> bool {
    let mut linked = false;
    paragraph.content = std::mem::take(&mut paragraph.content)
        .into_iter()
        .flat_map(|inline| link_uri_inline(inline, uri, &mut linked))
        .collect();
    linked
}

fn link_uri_inline(inline: Inline, uri: &str, linked: &mut bool) -> Vec<Inline> {
    let Inline::Text(text) = inline else {
        return vec![inline];
    };
    let Some(index) = text.find(uri) else {
        return vec![Inline::Text(text)];
    };
    *linked = true;
    let mut output = Vec::new();
    if index > 0 {
        output.push(Inline::Text(text[..index].to_string()));
    }
    output.push(Inline::Link(Link {
        href: uri.to_string(),
        title: None,
        content: vec![Inline::Text(uri.to_string())],
        source: None,
    }));
    let end = index + uri.len();
    if end < text.len() {
        output.push(Inline::Text(text[end..].to_string()));
    }
    output
}

fn link_annotation_uris(bytes: &[u8]) -> Vec<String> {
    let mut uris = Vec::new();
    let mut cursor = 0;
    while let Some(uri_marker) = find_bytes(bytes, b"/URI", cursor) {
        cursor = uri_marker + b"/URI".len();
        if let Some(uri) = literal_after(bytes, cursor) {
            uris.push(uri);
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn literal_after(bytes: &[u8], from: usize) -> Option<String> {
    let start = bytes[from..].iter().position(|byte| *byte == b'(')? + from + 1;
    let end = bytes[start..].iter().position(|byte| *byte == b')')? + start;
    Some(String::from_utf8_lossy(&bytes[start..end]).to_string())
}

fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
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
}
