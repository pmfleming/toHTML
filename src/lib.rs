mod model;

pub use model::*;

pub fn render_html(document: &Document) -> String {
    let mut html = String::from("<article>\n");

    if let Some(title) = &document.metadata.title {
        html.push_str("  <header>\n");
        html.push_str("    <h1>");
        push_escaped(&mut html, title);
        html.push_str("</h1>\n");
        html.push_str("  </header>\n");
    }

    for block in &document.blocks {
        render_block(&mut html, block);
    }

    html.push_str("</article>\n");
    html
}

fn render_block(html: &mut String, block: &Block) {
    match block {
        Block::Heading(heading) => render_heading(html, heading),
        Block::Paragraph(paragraph) => {
            html.push_str("  <p>");
            render_inlines(html, &paragraph.content);
            html.push_str("</p>\n");
        }
        Block::List(list) => render_list(html, list),
        Block::Table(table) => render_table(html, table),
        Block::Image(image) => render_image(html, image),
        Block::BlockQuote(block_quote) => {
            html.push_str("  <blockquote>\n");
            for block in &block_quote.blocks {
                render_block(html, block);
            }
            html.push_str("  </blockquote>\n");
        }
        Block::CodeBlock(code_block) => render_code_block(html, code_block),
        Block::PageBreak(page_break) => render_page_break(html, page_break),
        Block::PagePlaceholder(placeholder) => render_page_placeholder(html, placeholder),
        Block::HorizontalRule => html.push_str("  <hr>\n"),
        Block::RawHtml(raw) => {
            html.push_str(&raw.html);
            if !raw.html.ends_with('\n') {
                html.push('\n');
            }
        }
    }
}

fn render_heading(html: &mut String, heading: &Heading) {
    let level = heading.level.clamp(1, 6);
    html.push_str("  <h");
    html.push_str(&level.to_string());
    html.push('>');
    render_inlines(html, &heading.content);
    html.push_str("</h");
    html.push_str(&level.to_string());
    html.push_str(">\n");
}

fn render_list(html: &mut String, list: &List) {
    let tag = if list.ordered { "ol" } else { "ul" };
    html.push_str("  <");
    html.push_str(tag);
    if let Some(start) = list.start {
        html.push_str(" start=\"");
        html.push_str(&start.to_string());
        html.push('"');
    }
    html.push_str(">\n");
    for item in &list.items {
        html.push_str("    <li>");
        if let Some(checked) = item.checked {
            html.push_str("<input type=\"checkbox\" disabled");
            if checked {
                html.push_str(" checked");
            }
            html.push('>');
        }
        render_list_item_blocks(html, &item.blocks);
        html.push_str("</li>\n");
    }
    html.push_str("  </");
    html.push_str(tag);
    html.push_str(">\n");
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
    if let Some(caption) = &table.caption {
        html.push_str("    <caption>");
        render_inlines(html, caption);
        html.push_str("</caption>\n");
    }
    for row in &table.rows {
        html.push_str("    <tr>");
        for cell in &row.cells {
            render_table_cell(html, cell);
        }
        html.push_str("</tr>\n");
    }
    html.push_str("  </table>\n");
}

fn render_table_cell(html: &mut String, cell: &TableCell) {
    let tag = if cell.header { "th" } else { "td" };
    html.push('<');
    html.push_str(tag);
    if cell.colspan > 1 {
        html.push_str(" colspan=\"");
        html.push_str(&cell.colspan.to_string());
        html.push('"');
    }
    if cell.rowspan > 1 {
        html.push_str(" rowspan=\"");
        html.push_str(&cell.rowspan.to_string());
        html.push('"');
    }
    html.push('>');
    render_inlines(html, &cell.content);
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
}

fn render_image(html: &mut String, image: &Image) {
    html.push_str("  <img src=\"");
    push_attr_escaped(html, &image.src);
    html.push_str("\" alt=\"");
    if let Some(alt) = &image.alt {
        push_attr_escaped(html, alt);
    }
    html.push('"');
    if let Some(title) = &image.title {
        html.push_str(" title=\"");
        push_attr_escaped(html, title);
        html.push('"');
    }
    html.push_str(">\n");
}

fn render_code_block(html: &mut String, code_block: &CodeBlock) {
    html.push_str("  <pre><code");
    if let Some(language) = &code_block.language {
        html.push_str(" class=\"language-");
        push_attr_escaped(html, language);
        html.push('"');
    }
    html.push('>');
    push_escaped(html, &code_block.code);
    html.push_str("</code></pre>\n");
}

fn render_page_break(html: &mut String, page_break: &PageBreak) {
    html.push_str("  <hr data-page-break");
    if let Some(page_number) = page_break.page_number {
        html.push_str(" data-page=\"");
        html.push_str(&page_number.to_string());
        html.push('"');
    }
    html.push_str(">\n");
}

fn render_page_placeholder(html: &mut String, placeholder: &PagePlaceholder) {
    html.push_str("  <div data-page-placeholder");
    if let Some(page_number) = placeholder.page_number {
        html.push_str(" data-page=\"");
        html.push_str(&page_number.to_string());
        html.push('"');
    }
    match placeholder.reason {
        PlaceholderReason::Empty => html.push_str(" data-reason=\"empty\""),
        PlaceholderReason::NonExtractable => html.push_str(" data-reason=\"non-extractable\""),
    }
    html.push_str("></div>\n");
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
        Inline::Code(code) => {
            html.push_str("<code>");
            push_escaped(html, code);
            html.push_str("</code>");
        }
        Inline::Link(link) => render_link(html, link),
        Inline::Image(image) => {
            html.push_str("<img src=\"");
            push_attr_escaped(html, &image.src);
            html.push_str("\" alt=\"");
            if let Some(alt) = &image.alt {
                push_attr_escaped(html, alt);
            }
            html.push_str("\">");
        }
        Inline::LineBreak => html.push_str("<br>"),
    }
}

fn render_wrapped_inlines(html: &mut String, tag: &str, content: &[Inline]) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    render_inlines(html, content);
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
}

fn render_link(html: &mut String, link: &Link) {
    html.push_str("<a href=\"");
    push_attr_escaped(html, &link.href);
    html.push('"');
    if let Some(title) = &link.title {
        html.push_str(" title=\"");
        push_attr_escaped(html, title);
        html.push('"');
    }
    html.push('>');
    render_inlines(html, &link.content);
    html.push_str("</a>");
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
