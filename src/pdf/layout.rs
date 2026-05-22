use crate::{
    Block, Heading, Inline, List, ListItem, Paragraph, Table, TableAlignment, TableCell, TableRow,
};

use super::text::{text_lines, TextLine, TextSegment};

pub fn blocks_from_segments(segments: &[TextSegment]) -> Vec<Block> {
    let lines = text_lines(segments);
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if let Some((table, consumed)) = parse_table(&lines[index..]) {
            blocks.push(Block::Table(table));
            index += consumed;
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
    for marker in ["- ", "* ", "+ ", "• "] {
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
    line.font_size >= body_size * 1.25
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

fn parse_table(lines: &[TextLine]) -> Option<(Table, usize)> {
    let first = lines.first()?;
    let (reference, first_cells) = table_start(lines)?;
    let mut rows = vec![table_row(&first_cells, true)];
    let mut consumed = 1;
    let mut previous = first;
    for line in lines.iter().skip(1) {
        let cells = table_cells(line);
        if aligned_table_row(&reference, &cells) {
            rows.push(table_row(&cells, false));
            consumed += 1;
            previous = line;
            continue;
        }
        if extend_wrapped_table_cell(&reference, previous, line, &mut rows) {
            consumed += 1;
            previous = line;
            continue;
        }
        if !aligned_table_row(&reference, &cells) {
            break;
        }
    }

    (rows.len() >= 2).then_some((
        Table {
            rows,
            caption: None,
            source: None,
        },
        consumed,
    ))
}

fn table_start(lines: &[TextLine]) -> Option<(Vec<f32>, Vec<TableTextCell>)> {
    let first = lines.first()?;
    let first_cells = table_cells(first);
    if tabular_cells(&first_cells) {
        return Some((column_positions(&first_cells), first_cells));
    }

    let next_cells = table_cells(lines.get(1)?);
    if message_item_header(first) && tabular_cells(&next_cells) {
        return Some((
            column_positions(&next_cells),
            synthetic_message_item_header(&next_cells),
        ));
    }

    None
}

fn tabular_line(line: &TextLine) -> bool {
    tabular_cells(&table_cells(line))
}

#[derive(Debug, Clone, PartialEq)]
struct TableTextCell {
    text: String,
    x: f32,
    width: f32,
}

fn table_cells(line: &TextLine) -> Vec<TableTextCell> {
    let mut cells: Vec<TableTextCell> = Vec::new();
    let gap_threshold = table_cell_gap(line.font_size);
    for segment in &line.cells {
        let Some(current) = cells.last_mut() else {
            cells.push(TableTextCell {
                text: segment.text.clone(),
                x: segment.x,
                width: segment.width,
            });
            continue;
        };

        let current_end = current.x + current.width;
        if segment.x - current_end >= gap_threshold {
            cells.push(TableTextCell {
                text: segment.text.clone(),
                x: segment.x,
                width: segment.width,
            });
        } else {
            append_text(&mut current.text, &segment.text);
            current.width = (segment.x + segment.width - current.x).max(current.width);
        }
    }
    cells
}

fn table_cell_gap(font_size: f32) -> f32 {
    (font_size.max(8.0) * 1.75).max(18.0)
}

fn tabular_cells(cells: &[TableTextCell]) -> bool {
    cells.len() >= 2
        && cells
            .windows(2)
            .all(|cells| cells[1].x - cells[0].x >= 24.0)
}

fn column_positions(cells: &[TableTextCell]) -> Vec<f32> {
    cells.iter().map(|cell| cell.x).collect()
}

fn aligned_table_row(reference: &[f32], candidate: &[TableTextCell]) -> bool {
    tabular_cells(candidate)
        && reference.len() == candidate.len()
        && reference
            .iter()
            .zip(candidate)
            .all(|(left, right)| (*left - right.x).abs() <= 12.0)
}

fn extend_wrapped_table_cell(
    reference: &[f32],
    previous: &TextLine,
    line: &TextLine,
    rows: &mut [TableRow],
) -> bool {
    if rows.is_empty() || tabular_line(line) || is_list_line(line) {
        return false;
    }
    let gap = previous.y - line.y;
    let line_height = previous.font_size.max(line.font_size).max(8.0);
    if gap <= 0.0 || gap > line_height * 2.5 {
        return false;
    }

    let Some(row) = rows.last_mut() else {
        return false;
    };
    let Some(target_index) = continuation_cell_index(reference, line.x)
        .filter(|index| *index > 0)
        .or_else(|| fallback_continuation_index(row, line))
    else {
        return false;
    };

    let Some(cell) = row.cells.get_mut(target_index) else {
        return false;
    };
    append_cell_text(cell, &line.text);
    true
}

fn continuation_cell_index(reference: &[f32], x: f32) -> Option<usize> {
    reference
        .iter()
        .enumerate()
        .filter(|(_, column_x)| x + 12.0 >= **column_x)
        .min_by(|(_, left), (_, right)| (x - **left).abs().total_cmp(&(x - **right).abs()))
        .map(|(index, _)| index)
}

fn fallback_continuation_index(row: &TableRow, line: &TextLine) -> Option<usize> {
    let text = line.text.trim_start();
    if text.is_empty()
        || text.contains(':')
        || text
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit() || ch.is_ascii_uppercase())
    {
        return None;
    }
    row.cells.len().checked_sub(1).filter(|index| *index > 0)
}

fn message_item_header(line: &TextLine) -> bool {
    let text = line.text.as_str();
    text.contains("MessageItem") && text.contains("<XMLTag>") && text.contains("Mult.")
}

fn synthetic_message_item_header(reference: &[TableTextCell]) -> Vec<TableTextCell> {
    if !matches!(reference.len(), 4..=6) {
        return reference.to_vec();
    }

    let labels = match reference.len() {
        4 => vec!["MessageItem", "<XMLTag>", "Mult.", "Represent./Type"],
        5 => vec![
            "Index Or",
            "MessageItem",
            "<XMLTag>",
            "Mult.",
            "Represent./Type",
        ],
        6 => vec![
            "Index",
            "Or",
            "MessageItem",
            "<XMLTag>",
            "Mult.",
            "Represent./Type",
        ],
        _ => unreachable!(),
    };

    reference
        .iter()
        .zip(labels)
        .map(|(cell, label)| TableTextCell {
            text: label.to_string(),
            x: cell.x,
            width: cell.width,
        })
        .collect()
}

fn table_row(cells: &[TableTextCell], header: bool) -> TableRow {
    TableRow {
        cells: cells
            .iter()
            .map(|cell| TableCell {
                content: vec![Inline::Text(cell.text.clone())],
                header,
                colspan: 1,
                rowspan: 1,
                align: table_alignment(&cell.text),
                source: None,
            })
            .collect(),
        source: None,
    }
}

fn append_cell_text(cell: &mut TableCell, text: &str) {
    if let Some(Inline::Text(existing)) = cell.content.last_mut() {
        append_text(existing, text);
    } else {
        cell.content.push(Inline::Text(text.to_string()));
    }
}

fn append_text(existing: &mut String, next: &str) {
    if !existing.ends_with(' ') && !next.starts_with(' ') {
        existing.push(' ');
    }
    existing.push_str(next);
}

fn table_alignment(text: &str) -> Option<TableAlignment> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let numeric = text
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | ',' | '%' | '+' | '-' | ' '));
    numeric.then_some(TableAlignment::Right)
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
