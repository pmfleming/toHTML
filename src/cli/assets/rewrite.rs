use std::collections::HashMap;

use tohtml::{Block, Document, Image, Inline};

pub(super) fn image_sources(
    document: &mut Document,
    src_by_id: &HashMap<String, String>,
    src_by_original_path: &HashMap<String, String>,
) {
    for block in &mut document.blocks {
        block_image_sources(block, src_by_id, src_by_original_path);
    }
}

fn block_image_sources(
    block: &mut Block,
    src_by_id: &HashMap<String, String>,
    src_by_original_path: &HashMap<String, String>,
) {
    match block {
        Block::Heading(heading) => {
            inline_image_sources(&mut heading.content, src_by_id, src_by_original_path)
        }
        Block::Paragraph(paragraph) => {
            inline_image_sources(&mut paragraph.content, src_by_id, src_by_original_path)
        }
        Block::List(list) => list_image_sources(list, src_by_id, src_by_original_path),
        Block::Table(table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    inline_image_sources(&mut cell.content, src_by_id, src_by_original_path);
                }
            }
        }
        Block::Image(image) => image_source(image, src_by_id, src_by_original_path),
        Block::BlockQuote(block_quote) => {
            for block in &mut block_quote.blocks {
                block_image_sources(block, src_by_id, src_by_original_path);
            }
        }
        Block::CodeBlock(_)
        | Block::PageBreak(_)
        | Block::PagePlaceholder(_)
        | Block::HorizontalRule
        | Block::RawHtml(_) => {}
    }
}

fn list_image_sources(
    list: &mut tohtml::List,
    src_by_id: &HashMap<String, String>,
    src_by_original_path: &HashMap<String, String>,
) {
    for item in &mut list.items {
        for block in &mut item.blocks {
            block_image_sources(block, src_by_id, src_by_original_path);
        }
    }
}

fn inline_image_sources(
    inlines: &mut [Inline],
    src_by_id: &HashMap<String, String>,
    src_by_original_path: &HashMap<String, String>,
) {
    for inline in inlines {
        match inline {
            Inline::Emphasis(children)
            | Inline::Strong(children)
            | Inline::Strikethrough(children) => {
                inline_image_sources(children, src_by_id, src_by_original_path)
            }
            Inline::Link(link) => {
                inline_image_sources(&mut link.content, src_by_id, src_by_original_path)
            }
            Inline::Image(image) => image_source(image, src_by_id, src_by_original_path),
            Inline::Text(_) | Inline::Code(_) | Inline::LineBreak => {}
        }
    }
}

fn image_source(
    image: &mut Image,
    src_by_id: &HashMap<String, String>,
    src_by_original_path: &HashMap<String, String>,
) {
    if let Some(src) = image
        .asset_id
        .as_ref()
        .and_then(|asset_id| src_by_id.get(asset_id))
        .or_else(|| src_by_original_path.get(&image.src))
    {
        image.src = src.clone();
    }
}
