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

fn render_title(html: &mut String, document: &Document) {
    let title = document
        .metadata
        .title
        .as_deref()
        .unwrap_or("toHTML document");
    push_escaped(html, title);
}

fn render_default_styles(html: &mut String) {
    html.push_str("  <style>\n");
    html.push_str(
        "    body { font-family: Arial, Helvetica, sans-serif; font-size: 12px; line-height: 1.35; color: #111; }\n",
    );
    html.push_str("    article { max-width: 960px; margin: 0 auto; }\n");
    html.push_str("    h1 { font-size: 20px; line-height: 1.2; }\n");
    html.push_str("    h2 { font-size: 16px; line-height: 1.25; margin-top: 1.4em; }\n");
    html.push_str("    h3 { font-size: 14px; line-height: 1.25; }\n");
    html.push_str("    p { margin: 0.45em 0; }\n");
    html.push_str("    table { border-collapse: collapse; margin: 0.75em 0; width: 100%; }\n");
    html.push_str(
        "    th, td { border: 1px solid #777; padding: 3px 5px; vertical-align: top; }\n",
    );
    html.push_str("    th { font-weight: 700; background: #f2f2f2; }\n");
    html.push_str("  </style>\n");
}

fn render_body(html: &mut String, document: &Document) {
    html.push_str("<body>\n");
    html.push_str("  <article>\n");
    render_article_header(html, document);
    render_blocks(html, &document.blocks);
    html.push_str("  </article>\n");
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
    fn renders_complete_html_document_without_css_or_javascript() {
        let mut document = Document::with_title("Example");
        document.metadata.language = Some("en".to_string());
        document.blocks.push(Block::paragraph("Hello <world>"));

        let html = render_html(&document);

        assert!(html.starts_with("<!doctype html>\n<html lang=\"en\">"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<title>Example</title>"));
        assert!(html.contains("font-size: 12px"));
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
}
