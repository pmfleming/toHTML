mod code;
mod furniture;
mod links;
mod text;

use crate::{Block, Inline};

pub fn blocks(blocks: Vec<Block>, page_count: usize) -> Vec<Block> {
    let repeated = furniture::repeated_short_paragraphs(&blocks, page_count);
    let blocks = blocks
        .into_iter()
        .filter(|block| !furniture::is_page_furniture(block, &repeated))
        .map(text::repair_block)
        .collect();
    let blocks = links::repair_iso20022_catalogue_link_blocks(blocks);
    code::collapse_code_blocks(blocks)
}

pub(super) fn link_artifacts(blocks: Vec<Block>) -> Vec<Block> {
    links::repair_iso20022_catalogue_link_blocks(blocks)
}

fn paragraph_text(block: &Block) -> Option<String> {
    let Block::Paragraph(paragraph) = block else {
        return None;
    };
    let mut text = String::new();
    for inline in &paragraph.content {
        if let Inline::Text(value) = inline {
            text.push_str(value);
        }
    }
    Some(text)
}

#[cfg(test)]
mod tests;
