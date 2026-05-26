mod attrs;
mod blocks;
pub(crate) mod escape;
mod inlines;

use crate::{ConversionWarning, Document};

use blocks::render_blocks;
use escape::push_escaped;

pub fn render_html(document: &Document) -> String {
    let mut html = String::from("<!doctype html>\n");
    render_html_open(&mut html, document);
    render_head(&mut html, document);
    render_body(&mut html, document);
    html.push_str("</html>\n");
    html
}

fn render_html_open(html: &mut String, document: &Document) {
    html.push_str("<html");
    if let Some(language) = &document.metadata.language {
        html.push_str(" lang=\"");
        escape::push_attr_escaped(html, language);
        html.push('"');
    }
    html.push_str(">\n");
}

fn render_head(html: &mut String, document: &Document) {
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"utf-8\">\n");
    html.push_str("  <title>");
    render_title(html, document);
    html.push_str("</title>\n");
    if document.metadata.visual_html.is_some() {
        render_pdf_visual_styles(html);
    }
    html.push_str("</head>\n");
}

fn render_pdf_visual_styles(html: &mut String) {
    html.push_str(
        r#"  <style>
    .pdf-reconstructed-document {
      margin: 0;
      overflow-x: auto;
    }
    .pdf-recreated-page {
      position: relative;
      box-sizing: content-box;
      margin: 0 auto;
      overflow: hidden;
      background: #fff;
      font-family: Calibri, Arial, Helvetica, sans-serif;
    }
    .pdf-prose-page {
      font-family: "Times New Roman", Times, serif;
    }
    .pdf-text-fragment {
      position: absolute;
      display: block;
      overflow: visible;
      white-space: pre;
      line-height: 1;
      transform-origin: left top;
    }
    .pdf-shape {
      position: absolute;
      box-sizing: border-box;
    }
    .pdf-image {
      position: absolute;
      display: block;
      object-fit: fill;
    }
    .pdf-ink {
      position: absolute;
      display: block;
      overflow: visible;
      pointer-events: none;
    }
    .pdf-link-overlay {
      position: absolute;
      display: block;
      background: transparent;
    }
    hr[data-page-break] {
      border: 0;
      margin: 0;
    }
    @media screen {
      body {
        background: #f4f6f8;
      }
      .pdf-recreated-page {
        box-shadow: 0 0 0 1px #d8dde3, 0 12px 30px rgba(15, 23, 42, 0.12);
      }
    }
    @media print {
      body {
        margin: 0;
      }
      .pdf-recreated-page {
        margin: 0;
        box-shadow: none;
        break-after: page;
        page-break-after: always;
      }
      .pdf-recreated-page:last-of-type {
        break-after: auto;
        page-break-after: auto;
      }
      hr[data-page-break] {
        break-after: page;
        page-break-after: always;
      }
      .pdf-extracted-content,
      [data-conversion-warnings] {
        break-before: page;
        page-break-before: always;
      }
    }
  </style>
"#,
    );
}

fn render_title(html: &mut String, document: &Document) {
    let title = document
        .metadata
        .title
        .as_deref()
        .unwrap_or("toHTML document");
    push_escaped(html, title);
}

fn render_body(html: &mut String, document: &Document) {
    if document.metadata.visual_html.is_some() {
        render_reconstructed_body(html, document);
        return;
    }

    html.push_str("<body>\n");
    html.push_str("  <article>\n");
    render_article_header(html, document);
    render_blocks(html, &document.blocks);
    html.push_str("  </article>\n");
    render_warnings(html, &document.warnings);
    html.push_str("</body>\n");
}

fn render_reconstructed_body(html: &mut String, document: &Document) {
    html.push_str("<body>\n");
    html.push_str("  <main class=\"pdf-reconstructed-document\">\n");
    if let Some(visual_html) = &document.metadata.visual_html {
        html.push_str(visual_html);
    }
    html.push_str("    <details class=\"pdf-extracted-content\">\n");
    html.push_str("      <summary>Extracted HTML content</summary>\n");
    html.push_str("      <article>\n");
    render_article_header(html, document);
    render_blocks(html, &document.blocks);
    html.push_str("      </article>\n");
    html.push_str("    </details>\n");
    html.push_str("  </main>\n");
    render_warnings(html, &document.warnings);
    html.push_str("</body>\n");
}

fn render_article_header(html: &mut String, document: &Document) {
    if let Some(title) = &document.metadata.title {
        html.push_str("    <header>\n");
        html.push_str("      <h1>");
        push_escaped(html, title);
        html.push_str("</h1>\n");
        html.push_str("    </header>\n");
    }
}

fn render_warnings(html: &mut String, warnings: &[ConversionWarning]) {
    if warnings.is_empty() {
        return;
    }

    html.push_str("  <section data-conversion-warnings>\n");
    html.push_str("    <h2>Conversion Warnings</h2>\n");
    html.push_str("    <ul>\n");
    for warning in warnings {
        html.push_str("      <li>");
        push_escaped(html, &warning.message);
        html.push_str("</li>\n");
    }
    html.push_str("    </ul>\n");
    html.push_str("  </section>\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Inline, Link, PagePlaceholder, Paragraph, PlaceholderReason};

    #[test]
    fn renders_complete_html_document_without_broad_css_or_javascript() {
        let mut document = Document::with_title("Example");
        document.metadata.language = Some("en".to_string());
        document.blocks.push(Block::paragraph("Hello <world>"));

        let html = render_html(&document);

        assert!(html.starts_with("<!doctype html>\n<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<title>Example</title>"));
        assert!(!html.contains("<style"));
        assert!(html.contains("<article>"));
        assert!(html.contains("<h1>Example</h1>"));
        assert!(html.contains("<p>Hello &lt;world&gt;</p>"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn renders_rich_inline_content() {
        let document = Document {
            blocks: vec![Block::Paragraph(Paragraph {
                content: vec![
                    Inline::text("Use "),
                    Inline::Strong(vec![Inline::text("structured")]),
                    Inline::text(" "),
                    Inline::Link(Link {
                        href: "https://example.test?a=1&b=2".to_string(),
                        title: Some("Example".to_string()),
                        content: vec![Inline::text("HTML")],
                        source: None,
                    }),
                ],
                source: None,
            })],
            ..Document::default()
        };

        let html = render_html(&document);

        assert!(html.contains("<strong>structured</strong>"));
        assert!(html.contains("href=\"https://example.test?a=1&amp;b=2\""));
    }

    #[test]
    fn renders_pdf_page_placeholder() {
        let document = Document {
            blocks: vec![Block::PagePlaceholder(PagePlaceholder {
                page_number: Some(7),
                reason: PlaceholderReason::NonExtractable,
                source: None,
            })],
            ..Document::default()
        };

        let html = render_html(&document);

        assert!(html.contains("data-page-placeholder"));
        assert!(html.contains("data-page=\"7\""));
        assert!(html.contains("data-reason=\"non-extractable\""));
    }

    #[test]
    fn renders_pdf_reconstructed_html_with_minimal_visual_styles() {
        let document = Document {
            metadata: crate::DocumentMetadata {
                visual_html: Some(
                    "    <section class=\"pdf-recreated-page\"><span>Placed text</span></section>\n"
                        .to_string(),
                ),
                ..crate::DocumentMetadata::default()
            },
            blocks: vec![Block::paragraph("Extracted text")],
            ..Document::default()
        };

        let html = render_html(&document);

        assert!(html.contains("<style>"));
        assert!(html.contains(".pdf-recreated-page"));
        assert!(html.contains("break-after: page"));
        assert!(html.contains("hr[data-page-break]"));
        assert!(html.contains("@media print"));
        assert!(html.contains("class=\"pdf-reconstructed-document\""));
        assert!(html.contains("Placed text"));
        assert!(html.contains("<details class=\"pdf-extracted-content\">"));
        assert!(html.contains("Extracted text"));
        assert!(!html.contains("application/pdf"));
    }
}
