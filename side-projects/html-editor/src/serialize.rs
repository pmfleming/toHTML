//! HTML serialize / parse for toHTML-oriented documents.

use crate::doc::{
    chars_to_string, group_runs, Block, Doc, Image, PdfElement, PdfPage, StyledChar, Table,
};

pub fn serialize_document(doc: &Doc) -> String {
    if doc
        .blocks
        .iter()
        .any(|block| matches!(block, Block::PdfPage(_)))
    {
        return serialize_pdf_document(doc);
    }
    serialize_article_document(doc)
}

fn serialize_article_document(doc: &Doc) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("  <meta charset=\"utf-8\">\n");
    out.push_str("  <title>toHTML editor document</title>\n");
    out.push_str("</head>\n<body>\n  <article>\n");
    serialize_article_blocks(&mut out, &doc.blocks, "    ");
    out.push_str("  </article>\n</body>\n</html>\n");
    out
}

fn serialize_pdf_document(doc: &Doc) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("  <meta charset=\"utf-8\">\n");
    out.push_str("  <title>toHTML PDF visual document</title>\n");
    out.push_str(pdf_visual_css());
    out.push_str("</head>\n<body>\n  <main class=\"pdf-reconstructed-document\">\n");

    for block in &doc.blocks {
        if let Block::PdfPage(page) = block {
            out.push_str(&pdf_page_to_html(page, "    "));
            out.push('\n');
        }
    }

    let article_blocks = doc
        .blocks
        .iter()
        .filter(|block| !matches!(block, Block::PdfPage(_)))
        .cloned()
        .collect::<Vec<_>>();
    if !article_blocks.is_empty() {
        out.push_str("    <details class=\"pdf-extracted-content\" open>\n");
        out.push_str("      <summary>Extracted text</summary>\n");
        out.push_str("      <article>\n");
        serialize_article_blocks(&mut out, &article_blocks, "        ");
        out.push_str("      </article>\n");
        out.push_str("    </details>\n");
    }

    out.push_str("  </main>\n</body>\n</html>\n");
    out
}

fn serialize_article_blocks(out: &mut String, blocks: &[Block], indent: &str) {
    let mut i = 0;
    while i < blocks.len() {
        match &blocks[i] {
            Block::Bullet(_) => {
                out.push_str(indent);
                out.push_str("<ul>\n");
                while let Some(Block::Bullet(runs)) = blocks.get(i) {
                    out.push_str(indent);
                    out.push_str("  <li>");
                    out.push_str(&runs_to_html(runs));
                    out.push_str("</li>\n");
                    i += 1;
                }
                out.push_str(indent);
                out.push_str("</ul>\n");
            }
            Block::Numbered(_) => {
                out.push_str(indent);
                out.push_str("<ol>\n");
                while let Some(Block::Numbered(runs)) = blocks.get(i) {
                    out.push_str(indent);
                    out.push_str("  <li>");
                    out.push_str(&runs_to_html(runs));
                    out.push_str("</li>\n");
                    i += 1;
                }
                out.push_str(indent);
                out.push_str("</ol>\n");
            }
            block => {
                out.push_str(&block_to_html(block, indent));
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                i += 1;
            }
        }
    }
}

fn block_to_html(block: &Block, indent: &str) -> String {
    match block {
        Block::Heading(level, runs) => {
            format!("{indent}<h{level}>{}</h{level}>", runs_to_html(runs))
        }
        Block::Paragraph(runs) => format!("{indent}<p>{}</p>", runs_to_html(runs)),
        Block::Blockquote(runs) => {
            format!("{indent}<blockquote>{}</blockquote>", runs_to_html(runs))
        }
        Block::Pre(runs) => format!(
            "{indent}<pre><code>{}</code></pre>",
            escape_html(&chars_to_string(runs))
        ),
        Block::Table(table) => table_to_html(table),
        Block::Image(image) => image_to_html(image),
        Block::PageBreak(page) => match page {
            Some(page) => format!("{indent}<hr data-page-break data-page=\"{page}\">"),
            None => format!("{indent}<hr data-page-break>"),
        },
        Block::PagePlaceholder { page, reason } => {
            let page_attr = page
                .map(|n| format!(" data-page=\"{n}\""))
                .unwrap_or_default();
            format!(
                "{indent}<div data-page-placeholder{} data-reason=\"{}\"></div>",
                page_attr,
                escape_attr(reason)
            )
        }
        Block::PdfPage(page) => pdf_page_to_html(page, indent),
        Block::RawHtml(html) => html.clone(),
        Block::Hr => format!("{indent}<hr>"),
        Block::Bullet(runs) | Block::Numbered(runs) => {
            format!("{indent}<li>{}</li>", runs_to_html(runs))
        }
    }
}

fn table_to_html(table: &Table) -> String {
    let mut out = String::from("    <table>\n");
    if let Some(caption) = &table.caption {
        out.push_str("      <caption>");
        out.push_str(&runs_to_html(caption));
        out.push_str("</caption>\n");
    }
    for row in &table.rows {
        out.push_str("      <tr>");
        for cell in &row.cells {
            let tag = if cell.header { "th" } else { "td" };
            out.push('<');
            out.push_str(tag);
            if cell.colspan > 1 {
                out.push_str(&format!(" colspan=\"{}\"", cell.colspan));
            }
            if cell.rowspan > 1 {
                out.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
            }
            if let Some(align) = &cell.align {
                out.push_str(" style=\"text-align: ");
                out.push_str(&escape_attr(align));
                out.push('"');
            }
            out.push('>');
            out.push_str(&runs_to_html(&cell.content));
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        out.push_str("</tr>\n");
    }
    out.push_str("    </table>");
    out
}

fn pdf_page_to_html(page: &PdfPage, indent: &str) -> String {
    let mut out = String::new();
    out.push_str(indent);
    out.push_str("<section class=\"");
    out.push_str(&escape_attr(default_if_empty(
        &page.class_name,
        "pdf-recreated-page",
    )));
    out.push('"');
    if let Some(page_number) = page.page {
        out.push_str(&format!(" data-page=\"{page_number}\""));
    }
    if !page.style.is_empty() {
        out.push_str(" style=\"");
        out.push_str(&escape_attr(&page.style));
        out.push('"');
    }
    out.push_str(">\n");

    for element in &page.elements {
        out.push_str(&pdf_element_to_html(element, indent));
    }

    out.push_str(indent);
    out.push_str("</section>");
    out
}

fn pdf_element_to_html(element: &PdfElement, indent: &str) -> String {
    let child_indent = format!("{indent}  ");
    match element {
        PdfElement::Text(text) => format!(
            "{child_indent}<span class=\"{}\" style=\"{}\">{}</span>\n",
            escape_attr(default_if_empty(&text.class_name, "pdf-text-fragment")),
            escape_attr(&text.style),
            escape_html(&text.text)
        ),
        PdfElement::Image(image) => format!(
            "{child_indent}<img class=\"{}\" src=\"{}\" alt=\"{}\" style=\"{}\">\n",
            escape_attr(default_if_empty(&image.class_name, "pdf-image")),
            escape_attr(&image.src),
            escape_attr(&image.alt),
            escape_attr(&image.style)
        ),
        PdfElement::Shape(shape) => format!(
            "{child_indent}<div class=\"{}\" style=\"{}\"></div>\n",
            escape_attr(default_if_empty(&shape.class_name, "pdf-shape")),
            escape_attr(&shape.style)
        ),
        PdfElement::Ink(ink) => {
            let mut out = format!(
                "{child_indent}<svg class=\"{}\" style=\"{}\"",
                escape_attr(default_if_empty(&ink.class_name, "pdf-ink")),
                escape_attr(&ink.style)
            );
            if let Some(view_box) = &ink.view_box {
                out.push_str(" viewBox=\"");
                out.push_str(&escape_attr(view_box));
                out.push('"');
            }
            out.push_str(" aria-hidden=\"true\">");
            for path in &ink.paths {
                out.push_str("<path d=\"");
                out.push_str(&escape_attr(&path.d));
                out.push('"');
                if let Some(fill) = &path.fill {
                    out.push_str(" fill=\"");
                    out.push_str(&escape_attr(fill));
                    out.push('"');
                }
                if let Some(stroke) = &path.stroke {
                    out.push_str(" stroke=\"");
                    out.push_str(&escape_attr(stroke));
                    out.push('"');
                }
                if let Some(width) = &path.stroke_width {
                    out.push_str(" stroke-width=\"");
                    out.push_str(&escape_attr(width));
                    out.push('"');
                }
                out.push_str("/>");
            }
            out.push_str("</svg>\n");
            out
        }
        PdfElement::Link(link) => {
            let mut out = format!(
                "{child_indent}<a class=\"{}\" href=\"{}\" style=\"{}\"",
                escape_attr(default_if_empty(&link.class_name, "pdf-link-overlay")),
                escape_attr(&link.href),
                escape_attr(&link.style)
            );
            if let Some(label) = &link.label {
                out.push_str(" aria-label=\"");
                out.push_str(&escape_attr(label));
                out.push('"');
            }
            out.push_str("></a>\n");
            out
        }
    }
}

fn default_if_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

fn pdf_visual_css() -> &'static str {
    r#"  <style>
    .pdf-reconstructed-document {
      display: flex;
      flex-direction: column;
      gap: 24px;
      align-items: center;
      background: #f3f4f6;
      padding: 24px;
    }
    .pdf-recreated-page {
      position: relative;
      overflow: hidden;
      background: white;
      box-shadow: 0 1px 8px rgba(0,0,0,.18);
    }
    .pdf-text-fragment,
    .pdf-shape,
    .pdf-image,
    .pdf-ink,
    .pdf-link-overlay {
      position: absolute;
      box-sizing: border-box;
    }
    .pdf-text-fragment {
      white-space: pre;
      line-height: 1;
      transform-origin: left top;
    }
    .pdf-image,
    .pdf-ink {
      display: block;
    }
    .pdf-link-overlay {
      outline: 1px solid transparent;
    }
    .pdf-extracted-content {
      width: min(960px, 100%);
      background: white;
      padding: 16px 24px;
    }
  </style>
"#
}

fn image_to_html(image: &Image) -> String {
    let mut out = String::from("    <img");
    out.push_str(" src=\"");
    out.push_str(&escape_attr(&image.src));
    out.push_str("\" alt=\"");
    out.push_str(&escape_attr(&image.alt));
    out.push('"');
    if let Some(width) = image.width {
        out.push_str(&format!(" width=\"{width}\""));
    }
    if let Some(height) = image.height {
        out.push_str(&format!(" height=\"{height}\""));
    }
    if let Some(title) = &image.title {
        out.push_str(" title=\"");
        out.push_str(&escape_attr(title));
        out.push('"');
    }
    out.push('>');
    out
}

fn runs_to_html(runs: &[StyledChar]) -> String {
    let mut out = String::new();
    for (text, style) in group_runs(runs) {
        if text.is_empty() {
            continue;
        }
        let mut wrapped = escape_html(&text).replace('\n', "<br>");
        if style.code {
            wrapped = format!("<code>{wrapped}</code>");
        }
        if style.bold {
            wrapped = format!("<strong>{wrapped}</strong>");
        }
        if style.italic {
            wrapped = format!("<em>{wrapped}</em>");
        }
        if style.underline {
            wrapped = format!("<u>{wrapped}</u>");
        }
        if style.strike {
            wrapped = format!("<del>{wrapped}</del>");
        }
        if let Some(href) = &style.link {
            wrapped = format!("<a href=\"{}\">{wrapped}</a>", escape_attr(href));
        }
        out.push_str(&wrapped);
    }
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

mod parser;
pub use parser::parse_html;
