use crate::{Block, Image, Inline, SourceFormat, SourceSpan};

use super::super::text::{text_lines, TextLine, TextSegment};
use super::super::visual::VisualImage;

pub(super) fn add_content_images_to_blocks(
    blocks: &mut Vec<Block>,
    segments: &[TextSegment],
    images: &[VisualImage],
    page_width: f32,
    page_height: f32,
    page_number: u32,
) {
    let mut promoted = images
        .iter()
        .filter(|image| content_image(image, page_width, page_height))
        .collect::<Vec<_>>();
    if promoted.is_empty() {
        return;
    }

    let lines = text_lines(segments);
    promoted.sort_by(|left, right| {
        image_top(left, page_height).total_cmp(&image_top(right, page_height))
    });

    let mut inserted = 0;
    for image in promoted {
        let block = Block::Image(Image {
            src: image.src.clone(),
            alt: Some(image.alt.clone()),
            title: None,
            width: rounded_dimension(image.width),
            height: rounded_dimension(image.height),
            asset_id: None,
            source: Some(SourceSpan {
                format: SourceFormat::Pdf,
                page: Some(page_number),
                path: None,
                byte_range: None,
            }),
        });
        let base_index = insertion_index(blocks, &lines, image, page_height);
        let index = (base_index + inserted).min(blocks.len());
        blocks.insert(index, block);
        inserted += 1;
    }
}

fn content_image(image: &VisualImage, page_width: f32, page_height: f32) -> bool {
    if image.width < 24.0 || image.height < 18.0 {
        return false;
    }
    let page_area = page_width.max(1.0) * page_height.max(1.0);
    let image_area = image.width * image.height;
    if image_area < page_area * 0.01 && image.width < page_width * 0.22 {
        return false;
    }

    let top = image_top(image, page_height);
    top >= 72.0 || image_area >= page_area * 0.04
}

fn insertion_index(
    blocks: &[Block],
    lines: &[TextLine],
    image: &VisualImage,
    page_height: f32,
) -> usize {
    let top = image_top(image, page_height);
    let Some(anchor) = lines
        .iter()
        .filter(|line| line_top(line, page_height) + line.font_size * 0.5 <= top)
        .max_by(|left, right| line_top(left, page_height).total_cmp(&line_top(right, page_height)))
    else {
        return 0;
    };

    blocks
        .iter()
        .position(|block| block_contains_line(block, &anchor.text))
        .map_or(blocks.len(), |index| index + 1)
}

fn block_contains_line(block: &Block, line: &str) -> bool {
    let line = normalize_for_match(line);
    !line.is_empty()
        && block_text(block).is_some_and(|text| normalize_for_match(&text).contains(&line))
}

fn block_text(block: &Block) -> Option<String> {
    match block {
        Block::Heading(heading) => Some(inline_text(&heading.content)),
        Block::Paragraph(paragraph) => Some(inline_text(&paragraph.content)),
        Block::List(list) => Some(
            list.items
                .iter()
                .flat_map(|item| item.blocks.iter().filter_map(block_text))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        Block::Table(table) => Some(
            table
                .rows
                .iter()
                .flat_map(|row| row.cells.iter().map(|cell| inline_text(&cell.content)))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        _ => None,
    }
}

fn inline_text(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(value) | Inline::Code(value) => text.push_str(value),
            Inline::Emphasis(content)
            | Inline::Strong(content)
            | Inline::Strikethrough(content) => {
                text.push_str(&inline_text(content));
            }
            Inline::Link(link) => text.push_str(&inline_text(&link.content)),
            Inline::Image(image) => {
                if let Some(alt) = &image.alt {
                    text.push_str(alt);
                }
            }
            Inline::LineBreak => text.push(' '),
        }
    }
    text
}

fn normalize_for_match(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn rounded_dimension(value: f32) -> Option<u32> {
    (value > 0.0).then_some(value.round().max(1.0) as u32)
}

fn image_top(image: &VisualImage, page_height: f32) -> f32 {
    (page_height - image.y - image.height).max(0.0)
}

fn line_top(line: &TextLine, page_height: f32) -> f32 {
    (page_height - line.y - line.font_size).max(0.0)
}
