use crate::{
    Block, BlockQuote, CodeBlock, Heading, Image, List, PageBreak, PagePlaceholder,
    PlaceholderReason, RawHtml, Table, TableCell,
};

use super::attrs::{push_attr, push_end_tag, push_number_attr};
use super::escape::push_escaped;
use super::inlines::{render_image_tag, render_inlines};

pub fn render_blocks(html: &mut String, blocks: &[Block]) {
    for block in blocks {
        render_block(html, block);
    }
}

fn render_block(html: &mut String, block: &Block) {
    match block {
        Block::Heading(heading) => render_heading(html, heading),
        Block::Paragraph(paragraph) => render_paragraph(html, &paragraph.content),
        Block::List(list) => render_list(html, list),
        Block::Table(table) => render_table(html, table),
        Block::Image(image) => render_image_block(html, image),
        Block::BlockQuote(block_quote) => render_block_quote(html, block_quote),
        Block::CodeBlock(code_block) => render_code_block(html, code_block),
        Block::PageBreak(page_break) => render_page_break(html, page_break),
        Block::PagePlaceholder(placeholder) => render_page_placeholder(html, placeholder),
        Block::HorizontalRule => html.push_str("    <hr>\n"),
        Block::RawHtml(raw) => render_raw_html(html, raw),
    }
}

fn render_heading(html: &mut String, heading: &Heading) {
    let level = heading.level.clamp(1, 6);
    let tag = format!("h{level}");
    html.push_str("    ");
    render_wrapped_inlines(html, &tag, &heading.content);
    html.push('\n');
}

fn render_paragraph(html: &mut String, content: &[crate::Inline]) {
    html.push_str("    <p>");
    render_inlines(html, content);
    html.push_str("</p>\n");
}

fn render_list(html: &mut String, list: &List) {
    let tag = if list.ordered { "ol" } else { "ul" };
    html.push_str("    <");
    html.push_str(tag);
    push_number_attr(html, "start", list.start);
    html.push_str(">\n");

    for item in &list.items {
        html.push_str("      <li>");
        render_checkbox(html, item.checked);
        render_list_item_blocks(html, &item.blocks);
        html.push_str("</li>\n");
    }

    html.push_str("    ");
    push_end_tag(html, tag);
    html.push('\n');
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
    html.push_str("    <table>\n");
    render_caption(html, table);
    for row in &table.rows {
        html.push_str("      <tr>");
        for cell in &row.cells {
            render_table_cell(html, cell);
        }
        html.push_str("</tr>\n");
    }
    html.push_str("    </table>\n");
}

fn render_caption(html: &mut String, table: &Table) {
    if let Some(caption) = &table.caption {
        html.push_str("      <caption>");
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
    html.push_str("    ");
    render_image_tag(html, image);
    html.push('\n');
}

fn render_block_quote(html: &mut String, block_quote: &BlockQuote) {
    html.push_str("    <blockquote>\n");
    render_blocks(html, &block_quote.blocks);
    html.push_str("    </blockquote>\n");
}

fn render_code_block(html: &mut String, code_block: &CodeBlock) {
    html.push_str("    <pre><code");
    if let Some(language) = &code_block.language {
        push_attr(html, "class", &format!("language-{language}"));
    }
    html.push('>');
    push_escaped(html, &code_block.code);
    html.push_str("</code></pre>\n");
}

fn render_page_break(html: &mut String, page_break: &PageBreak) {
    html.push_str("    <hr data-page-break");
    push_number_attr(html, "data-page", page_break.page_number.map(u64::from));
    html.push_str(">\n");
}

fn render_page_placeholder(html: &mut String, placeholder: &PagePlaceholder) {
    html.push_str("    <div data-page-placeholder");
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

fn render_wrapped_inlines(html: &mut String, tag: &str, content: &[crate::Inline]) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    render_inlines(html, content);
    push_end_tag(html, tag);
}
