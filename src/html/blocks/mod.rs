mod lists;
mod media;
mod pages;
mod tables;
mod text;

use crate::Block;

use lists::render_list;
use media::render_image_block;
use pages::{render_page_break, render_page_placeholder};
use tables::render_table;
use text::{
    render_block_quote, render_code_block, render_heading, render_paragraph, render_raw_html,
};

pub fn render_blocks(html: &mut String, blocks: &[Block]) {
    for block in blocks {
        render_block(html, block);
    }
}

pub(super) fn render_block(html: &mut String, block: &Block) {
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
