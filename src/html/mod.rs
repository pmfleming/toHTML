mod attrs;
mod blocks;
mod escape;
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
    render_default_styles(html);
    html.push_str("</head>\n");
}

fn render_default_styles(html: &mut String) {
    html.push_str(
        r#"  <style>
    :root {
      color-scheme: light;
      font-family: Arial, Helvetica, sans-serif;
      font-size: 12px;
      line-height: 1.45;
      color: #202124;
      background: #f3f4f6;
    }
    body {
      margin: 0;
      background: #f3f4f6;
    }
    article, [data-conversion-warnings] {
      box-sizing: border-box;
      width: min(100%, 8.5in);
      margin: 24px auto;
      padding: 0.62in 0.7in;
      background: #fff;
      box-shadow: 0 1px 5px rgb(60 64 67 / 18%);
    }
    .pdf-visual-document {
      width: 100%;
      margin: 0;
      padding: 0;
    }
    .pdf-visual-source {
      display: block;
      width: 100vw;
      height: 100vh;
      min-height: 100vh;
      border: 0;
      background: #fff;
      box-shadow: none;
    }
    .pdf-extracted-content {
      width: min(100%, 8.5in);
      margin: 24px auto;
      color: #374151;
    }
    .pdf-extracted-content > summary {
      cursor: pointer;
      font-weight: 700;
      margin-bottom: 0.8rem;
    }
    .pdf-extracted-content article {
      margin: 0;
    }
    header {
      margin-bottom: 1.2rem;
      border-bottom: 1px solid #d8dce2;
      padding-bottom: 0.45rem;
    }
    h1, h2, h3, h4, h5, h6 {
      margin: 1.05em 0 0.45em;
      line-height: 1.18;
      color: #111827;
      page-break-after: avoid;
    }
    h1 { font-size: 1.55rem; }
    h2 { font-size: 1.22rem; }
    h3 { font-size: 1.08rem; }
    p {
      margin: 0 0 0.72em;
    }
    table {
      width: 100%;
      margin: 1rem 0;
      border-collapse: collapse;
      font-size: 0.9em;
    }
    th, td {
      border: 1px solid #cfd6df;
      padding: 0.36rem 0.45rem;
      vertical-align: top;
    }
    th {
      background: #eef2f7;
      font-weight: 700;
    }
    pre {
      overflow-x: auto;
      padding: 0.85rem;
      border: 1px solid #d8dce2;
      background: #f8fafc;
      font-size: 0.88em;
      line-height: 1.35;
    }
    code {
      font-family: Consolas, "Liberation Mono", monospace;
    }
    img {
      max-width: 100%;
      height: auto;
    }
    hr[data-page-break] {
      height: 0;
      margin: 1.2rem -0.7in;
      border: 0;
      border-top: 16px solid #f3f4f6;
      break-after: page;
    }
    [data-page-placeholder] {
      min-height: 6rem;
      border: 1px dashed #b7c0cc;
      background: #fafbfc;
    }
    .pdf-rotated-text {
      display: inline-block;
      writing-mode: vertical-rl;
      max-height: 12rem;
      margin: 0.35rem 0.7rem 0.35rem 0;
      padding: 0.25rem;
      background: #fff;
      border: 1px solid #d8dce2;
      font-size: 0.9em;
      line-height: 1.2;
      vertical-align: top;
    }
    [data-conversion-warnings] {
      font-size: 0.9rem;
      color: #4b5563;
    }
    @media print {
      body { background: #fff; }
      article, [data-conversion-warnings] {
        width: auto;
        margin: 0;
        padding: 0;
        box-shadow: none;
      }
      hr[data-page-break] {
        margin: 0;
        border: 0;
        break-after: page;
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
    if document.metadata.visual_source.is_some() {
        render_visual_body(html, document);
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

fn render_visual_body(html: &mut String, document: &Document) {
    html.push_str("<body>\n");
    html.push_str("  <main class=\"pdf-visual-document\">\n");
    render_visual_source(
        html,
        document
            .metadata
            .visual_source
            .as_deref()
            .unwrap_or_default(),
    );
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

fn render_visual_source(html: &mut String, source: &str) {
    html.push_str("    <object class=\"pdf-visual-source\" type=\"application/pdf\" data=\"");
    escape::push_attr_escaped(html, source);
    html.push_str("#toolbar=0&amp;navpanes=0\">\n");
    html.push_str("      <p><a href=\"");
    escape::push_attr_escaped(html, source);
    html.push_str("\">Open source PDF</a></p>\n");
    html.push_str("    </object>\n");
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
    fn renders_complete_html_document_with_document_styles_and_without_javascript() {
        let mut document = Document::with_title("Example");
        document.metadata.language = Some("en".to_string());
        document.blocks.push(Block::paragraph("Hello <world>"));

        let html = render_html(&document);

        assert!(html.starts_with("<!doctype html>\n<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<title>Example</title>"));
        assert!(html.contains("<style>"));
        assert!(html.contains("article, [data-conversion-warnings]"));
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
    fn renders_pdf_visual_source_as_primary_surface() {
        let document = Document {
            metadata: crate::DocumentMetadata {
                visual_source: Some("../input/example.pdf".to_string()),
                ..crate::DocumentMetadata::default()
            },
            blocks: vec![Block::paragraph("Extracted text")],
            ..Document::default()
        };

        let html = render_html(&document);

        assert!(html.contains("class=\"pdf-visual-source\""));
        assert!(html.contains("data=\"../input/example.pdf#toolbar=0&amp;navpanes=0\""));
        assert!(html.contains("<details class=\"pdf-extracted-content\">"));
        assert!(html.contains("Extracted text"));
    }
}
