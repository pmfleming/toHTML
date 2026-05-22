use crate::{Block, List};

use super::render_block;
use crate::html::attrs::{push_end_tag, push_number_attr};
use crate::html::inlines::render_inlines;

pub(super) fn render_list(html: &mut String, list: &List) {
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
