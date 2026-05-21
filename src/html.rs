use crate::{
    Block, BlockQuote, CodeBlock, Document, Heading, Image, Inline, Link, List, PageBreak,
    PagePlaceholder, Paragraph, PlaceholderReason, RawHtml, Table, TableCell,
};

pub fn render_html(document: &Document) -> String {
    let mut html = String::from("<article>\n");
    render_header(&mut html, document);
    render_blocks(&mut html, &document.blocks);
    html.push_str("</article>\n");
    html
}

fn render_header(html: &mut String, document: &Document) {
    if let Some(title) = &document.metadata.title {
        html.push_str("  <header>\n");
        html.push_str("    <h1>");
        push_escaped(html, title);
        html.push_str("</h1>\n");
        html.push_str("  </header>\n");
    }
}

fn render_blocks(html: &mut String, blocks: &[Block]) {
    for block in blocks {
        render_block(html, block);
    }
}

fn render_block(html: &mut String, block: &Block) {
    match block {
        Block::Heading(heading) => render_heading(html, heading),
        Block::Paragraph(paragraph) => render_paragraph(html, paragraph),
        Block::List(list) => render_list(html, list),
        Block::Table(table) => render_table(html, table),
        Block::Image(image) => render_image_block(html, image),
        Block::BlockQuote(block_quote) => render_block_quote(html, block_quote),
        Block::CodeBlock(code_block) => render_code_block(html, code_block),
        Block::PageBreak(page_break) => render_page_break(html, page_break),
        Block::PagePlaceholder(placeholder) => render_page_placeholder(html, placeholder),
        Block::HorizontalRule => html.push_str("  <hr>\n"),
        Block::RawHtml(raw) => render_raw_html(html, raw),
    }
}

fn render_heading(html: &mut String, heading: &Heading) {
    let level = heading.level.clamp(1, 6);
    let tag = format!("h{level}");
    render_wrapped_inlines(html, &tag, &heading.content);
    html.push('\n');
}

fn render_paragraph(html: &mut String, paragraph: &Paragraph) {
    html.push_str("  <p>");
    render_inlines(html, &paragraph.content);
    html.push_str("</p>\n");
}

fn render_list(html: &mut String, list: &List) {
    let tag = if list.ordered { "ol" } else { "ul" };
    html.push_str("  <");
    html.push_str(tag);
    push_number_attr(html, "start", list.start);
    html.push_str(">\n");

    for item in &list.items {
        html.push_str("    <li>");
        render_checkbox(html, item.checked);
        render_list_item_blocks(html, &item.blocks);
        html.push_str("</li>\n");
    }

    html.push_str("  </");
    html.push_str(tag);
    html.push_str(">\n");
}

fn render_checkbox(html: &mut String, checked: Option<bool>) {
    if let Some(checked) = checked {
        html.push_str("<input type=\"checkbox\" disabled");
        if checked {
            html.push_str(" checked");
        }
        html.push('>');
    }
}

fn render_list_item_blocks(html: &mut String, blocks: &[Block]) {
    for block in blocks {
        match block {
            Block::Paragraph(paragraph) => render_inlines(html, &paragraph.content),
            other => render_block(html, other),
        }
    }
}

fn render_table(html: &mut String, table: &Table) {
    html.push_str("  <table>\n");
    render_caption(html, table);
    for row in &table.rows {
        html.push_str("    <tr>");
        for cell in &row.cells {
            render_table_cell(html, cell);
        }
        html.push_str("</tr>\n");
    }
    html.push_str("  </table>\n");
}

fn render_caption(html: &mut String, table: &Table) {
    if let Some(caption) = &table.caption {
        html.push_str("    <caption>");
        render_inlines(html, caption);
        html.push_str("</caption>\n");
    }
}

fn render_table_cell(html: &mut String, cell: &TableCell) {
    let tag = if cell.header { "th" } else { "td" };
    html.push('<');
    html.push_str(tag);
    push_number_attr(html, "colspan", span_attr(cell.colspan));
    push_number_attr(html, "rowspan", span_attr(cell.rowspan));
    html.push('>');
    render_inlines(html, &cell.content);
    push_end_tag(html, tag);
}

fn span_attr(span: u16) -> Option<u64> {
    (span > 1).then_some(u64::from(span))
}

fn render_image_block(html: &mut String, image: &Image) {
    html.push_str("  ");
    render_image_tag(html, image);
    html.push('\n');
}

fn render_block_quote(html: &mut String, block_quote: &BlockQuote) {
    html.push_str("  <blockquote>\n");
    render_blocks(html, &block_quote.blocks);
    html.push_str("  </blockquote>\n");
}

fn render_code_block(html: &mut String, code_block: &CodeBlock) {
    html.push_str("  <pre><code");
    if let Some(language) = &code_block.language {
        push_attr(html, "class", &format!("language-{language}"));
    }
    html.push('>');
    push_escaped(html, &code_block.code);
    html.push_str("</code></pre>\n");
}

fn render_page_break(html: &mut String, page_break: &PageBreak) {
    html.push_str("  <hr data-page-break");
    push_number_attr(html, "data-page", page_break.page_number.map(u64::from));
    html.push_str(">\n");
}

fn render_page_placeholder(html: &mut String, placeholder: &PagePlaceholder) {
    html.push_str("  <div data-page-placeholder");
    push_number_attr(html, "data-page", placeholder.page_number.map(u64::from));
    push_attr(html, "data-reason", placeholder_reason(placeholder.reason));
    html.push_str("></div>\n");
}

fn placeholder_reason(reason: PlaceholderReason) -> &'static str {
    match reason {
        PlaceholderReason::Empty => "empty",
        PlaceholderReason::NonExtractable => "non-extractable",
    }
}

fn render_raw_html(html: &mut String, raw: &RawHtml) {
    html.push_str(&raw.html);
    if !raw.html.ends_with('\n') {
        html.push('\n');
    }
}

fn render_inlines(html: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        render_inline(html, inline);
    }
}

fn render_inline(html: &mut String, inline: &Inline) {
    match inline {
        Inline::Text(text) => push_escaped(html, text),
        Inline::Emphasis(content) => render_wrapped_inlines(html, "em", content),
        Inline::Strong(content) => render_wrapped_inlines(html, "strong", content),
        Inline::Strikethrough(content) => render_wrapped_inlines(html, "del", content),
        Inline::Code(code) => render_inline_code(html, code),
        Inline::Link(link) => render_link(html, link),
        Inline::Image(image) => render_image_tag(html, image),
        Inline::LineBreak => html.push_str("<br>"),
    }
}

fn render_inline_code(html: &mut String, code: &str) {
    html.push_str("<code>");
    push_escaped(html, code);
    html.push_str("</code>");
}

fn render_wrapped_inlines(html: &mut String, tag: &str, content: &[Inline]) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    render_inlines(html, content);
    push_end_tag(html, tag);
}

fn render_link(html: &mut String, link: &Link) {
    html.push_str("<a");
    push_attr(html, "href", &link.href);
    if let Some(title) = &link.title {
        push_attr(html, "title", title);
    }
    html.push('>');
    render_inlines(html, &link.content);
    html.push_str("</a>");
}

fn render_image_tag(html: &mut String, image: &Image) {
    html.push_str("<img");
    push_attr(html, "src", &image.src);
    push_attr(html, "alt", image.alt.as_deref().unwrap_or(""));
    if let Some(title) = &image.title {
        push_attr(html, "title", title);
    }
    html.push('>');
}

fn push_end_tag(html: &mut String, tag: &str) {
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
}

fn push_number_attr(html: &mut String, name: &str, value: Option<u64>) {
    if let Some(value) = value {
        push_attr(html, name, &value.to_string());
    }
}

fn push_attr(html: &mut String, name: &str, value: &str) {
    html.push(' ');
    html.push_str(name);
    html.push_str("=\"");
    push_attr_escaped(html, value);
    html.push('"');
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn push_attr_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Inline, Link, PagePlaceholder, Paragraph};

    #[test]
    fn renders_basic_document() {
        let mut document = Document::with_title("Example");
        document.blocks.push(Block::paragraph("Hello <world>"));

        let html = render_html(&document);

        assert!(html.contains("<h1>Example</h1>"));
        assert!(html.contains("<p>Hello &lt;world&gt;</p>"));
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
