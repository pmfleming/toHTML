mod images;
mod tables;

use crate::{Block, Heading, Inline, List, ListItem, Paragraph, RawHtml};

use super::text::{text_lines, TextLine, TextSegment};
use crate::html::escape::push_attr_escaped;
use tables::{parse_table_with_header, tabular_line};

pub fn blocks_from_segments(segments: &[TextSegment]) -> Vec<Block> {
    let lines = text_lines(segments);
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if let Some((table, consumed)) = parse_table_with_header(&lines, index) {
            blocks.push(Block::Table(table));
            index += consumed;
        } else if rotated_line(&lines[index]) {
            blocks.push(rotated_text_block(&lines[index]));
            index += 1;
        } else if let Some((list, consumed)) = parse_list(&lines[index..]) {
            blocks.push(Block::List(list));
            index += consumed;
        } else if heading_line(&lines, index) {
            blocks.push(heading(&lines[index]));
            index += 1;
        } else if semantic_inline_line(&lines[index]) {
            blocks.push(paragraph_from_line(&lines[index]));
            index += 1;
        } else {
            let (text, consumed) = parse_paragraph(&lines[index..]);
            blocks.push(paragraph(&text));
            index += consumed;
        }
    }

    blocks
}

pub(super) fn add_content_images_to_blocks(
    blocks: &mut Vec<Block>,
    segments: &[TextSegment],
    images: &[super::visual::VisualImage],
    page_width: f32,
    page_height: f32,
    page_number: u32,
) {
    images::add_content_images_to_blocks(
        blocks,
        segments,
        images,
        page_width,
        page_height,
        page_number,
    );
}

fn rotated_line(line: &TextLine) -> bool {
    normalized_rotation(line.rotation).abs() >= 30.0
}

fn normalized_rotation(rotation: f32) -> f32 {
    let mut rotation = rotation % 360.0;
    if rotation > 180.0 {
        rotation -= 360.0;
    } else if rotation < -180.0 {
        rotation += 360.0;
    }
    rotation
}

fn rotated_text_block(line: &TextLine) -> Block {
    let mut html = String::from("    <p class=\"pdf-rotated-text\" data-rotation=\"");
    html.push_str(&format!("{:.0}", normalized_rotation(line.rotation)));
    html.push_str("\">");
    push_attr_escaped(&mut html, &line.text);
    html.push_str("</p>");
    Block::RawHtml(RawHtml { html, source: None })
}

fn parse_paragraph(lines: &[TextLine]) -> (String, usize) {
    let mut text = lines[0].text.clone();
    let mut consumed = 1;

    for line in lines.iter().skip(1) {
        let previous = &lines[consumed - 1];
        if tabular_line(line) || is_list_line(line) || !paragraph_continuation(previous, line) {
            break;
        }
        push_paragraph_line(&mut text, &line.text);
        consumed += 1;
    }

    (text, consumed)
}

fn parse_list(lines: &[TextLine]) -> Option<(List, usize)> {
    let first = list_marker(&lines[0])?;
    let mut items = Vec::new();
    let mut consumed = 0;

    for line in lines {
        let Some(marker) = list_marker(line) else {
            break;
        };
        if marker.ordered != first.ordered {
            break;
        }
        items.push(ListItem {
            checked: None,
            blocks: vec![paragraph(marker.text)],
            source: None,
        });
        consumed += 1;
    }

    (items.len() >= 2).then_some((
        List {
            ordered: first.ordered,
            start: first.start,
            items,
            source: None,
        },
        consumed,
    ))
}

#[derive(Debug, Clone, PartialEq)]
struct ListMarker<'a> {
    ordered: bool,
    start: Option<u64>,
    text: &'a str,
}

fn list_marker(line: &TextLine) -> Option<ListMarker<'_>> {
    unordered_marker(&line.text).or_else(|| ordered_marker(&line.text))
}

fn unordered_marker(text: &str) -> Option<ListMarker<'_>> {
    let text = text.trim_start();
    for marker in ["- ", "* ", "+ ", "• ", "o "] {
        if let Some(item) = text.strip_prefix(marker) {
            return Some(ListMarker {
                ordered: false,
                start: None,
                text: item.trim(),
            });
        }
    }
    None
}

fn ordered_marker(text: &str) -> Option<ListMarker<'_>> {
    let text = text.trim_start();
    let marker_end = text.find(|ch: char| !(ch.is_ascii_digit()))?;
    let marker = &text[..marker_end];
    let rest = text[marker_end..].trim_start();
    if marker.is_empty() || !matches!(rest.chars().next(), Some('.' | ')')) {
        return None;
    }
    Some(ListMarker {
        ordered: true,
        start: marker.parse().ok(),
        text: rest[1..].trim_start(),
    })
}

fn is_list_line(line: &TextLine) -> bool {
    list_marker(line).is_some()
}

fn heading_line(lines: &[TextLine], index: usize) -> bool {
    let line = &lines[index];
    if tagged_heading_level(line).is_some() {
        return true;
    }
    if line.text.len() > 120 || line.text.ends_with('.') || line.text.contains("  ") {
        return false;
    }
    let body_size = median_font_size(lines);
    if line.font_size < body_size * 1.25 {
        return false;
    }
    isolated_line(lines, index) || ends_like_heading(&line.text)
}

fn isolated_line(lines: &[TextLine], index: usize) -> bool {
    let line = &lines[index];
    let line_height = line.font_size.max(8.0);
    let isolated_above = index
        .checked_sub(1)
        .and_then(|previous| lines.get(previous))
        .is_none_or(|previous| previous.y - line.y >= line_height * 1.5);
    let isolated_below = lines
        .get(index + 1)
        .is_none_or(|next| line.y - next.y >= line_height * 1.5);
    isolated_above && isolated_below
}

fn ends_like_heading(text: &str) -> bool {
    let trimmed = text.trim_end();
    !trimmed.ends_with(',')
        && !trimmed.ends_with(';')
        && !trimmed.ends_with(':')
        && trimmed.split_whitespace().count() <= 12
}

fn median_font_size(lines: &[TextLine]) -> f32 {
    let mut sizes: Vec<f32> = lines.iter().map(|line| line.font_size).collect();
    sizes.sort_by(f32::total_cmp);
    let index = sizes.len().saturating_sub(1) / 2;
    sizes.get(index).copied().unwrap_or(12.0).max(1.0)
}

fn paragraph_continuation(previous: &TextLine, candidate: &TextLine) -> bool {
    let gap = previous.y - candidate.y;
    let line_height = previous.font_size.max(candidate.font_size).max(8.0);
    let indentation_delta = (candidate.x - previous.x).abs();
    gap > 0.0 && gap <= line_height * 1.8 && indentation_delta <= line_height * 2.0
}

fn push_paragraph_line(text: &mut String, line: &str) {
    if !text.ends_with(' ') && !line.starts_with(' ') {
        text.push(' ');
    }
    text.push_str(line);
}

fn paragraph(text: &str) -> Block {
    Block::Paragraph(Paragraph {
        content: vec![Inline::Text(text.to_string())],
        source: None,
    })
}

fn paragraph_from_line(line: &TextLine) -> Block {
    Block::Paragraph(Paragraph {
        content: vec![semantic_inline(&line.text, line.role.as_deref())],
        source: None,
    })
}

fn semantic_inline(text: &str, role: Option<&str>) -> Inline {
    match role {
        Some("Strong") => Inline::Strong(vec![Inline::Text(text.to_string())]),
        Some("Em") => Inline::Emphasis(vec![Inline::Text(text.to_string())]),
        _ => Inline::Text(text.to_string()),
    }
}

fn semantic_inline_line(line: &TextLine) -> bool {
    matches!(line.role.as_deref(), Some("Strong" | "Em"))
}

fn heading(line: &TextLine) -> Block {
    Block::Heading(Heading {
        level: tagged_heading_level(line).unwrap_or(2),
        content: vec![Inline::Text(line.text.clone())],
        source: None,
    })
}

fn tagged_heading_level(line: &TextLine) -> Option<u8> {
    let role = line.role.as_deref()?;
    match role {
        "H" | "H1" => Some(1),
        "H2" => Some(2),
        "H3" => Some(3),
        "H4" => Some(4),
        "H5" => Some(5),
        "H6" => Some(6),
        _ => None,
    }
}
