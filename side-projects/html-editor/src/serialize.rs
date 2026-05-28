//! HTML serialize / parse for toHTML-oriented documents.

use crate::doc::{chars_to_string, group_runs, Block, Doc, Image, StyledChar, Table};

pub fn serialize_document(doc: &Doc) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("  <meta charset=\"utf-8\">\n");
    out.push_str("  <title>toHTML editor document</title>\n");
    out.push_str("</head>\n<body>\n  <article>\n");

    let mut i = 0;
    while i < doc.blocks.len() {
        match &doc.blocks[i] {
            Block::Bullet(_) => {
                out.push_str("    <ul>\n");
                while let Some(Block::Bullet(runs)) = doc.blocks.get(i) {
                    out.push_str("      <li>");
                    out.push_str(&runs_to_html(runs));
                    out.push_str("</li>\n");
                    i += 1;
                }
                out.push_str("    </ul>\n");
            }
            Block::Numbered(_) => {
                out.push_str("    <ol>\n");
                while let Some(Block::Numbered(runs)) = doc.blocks.get(i) {
                    out.push_str("      <li>");
                    out.push_str(&runs_to_html(runs));
                    out.push_str("</li>\n");
                    i += 1;
                }
                out.push_str("    </ol>\n");
            }
            block => {
                out.push_str(&block_to_html(block));
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                i += 1;
            }
        }
    }

    out.push_str("  </article>\n</body>\n</html>\n");
    out
}

fn block_to_html(block: &Block) -> String {
    match block {
        Block::Heading(level, runs) => format!("    <h{level}>{}</h{level}>", runs_to_html(runs)),
        Block::Paragraph(runs) => format!("    <p>{}</p>", runs_to_html(runs)),
        Block::Blockquote(runs) => format!("    <blockquote>{}</blockquote>", runs_to_html(runs)),
        Block::Pre(runs) => format!(
            "    <pre><code>{}</code></pre>",
            escape_html(&chars_to_string(runs))
        ),
        Block::Table(table) => table_to_html(table),
        Block::Image(image) => image_to_html(image),
        Block::PageBreak(page) => match page {
            Some(page) => format!("    <hr data-page-break data-page=\"{page}\">"),
            None => "    <hr data-page-break>".into(),
        },
        Block::PagePlaceholder { page, reason } => {
            let page_attr = page
                .map(|n| format!(" data-page=\"{n}\""))
                .unwrap_or_default();
            format!(
                "    <div data-page-placeholder{} data-reason=\"{}\"></div>",
                page_attr,
                escape_attr(reason)
            )
        }
        Block::RawHtml(html) => html.clone(),
        Block::Hr => "    <hr>".into(),
        Block::Bullet(runs) | Block::Numbered(runs) => {
            format!("      <li>{}</li>", runs_to_html(runs))
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
