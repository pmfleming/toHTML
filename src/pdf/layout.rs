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
    if !tabular_line(first) {
        return None;
    }

    let mut rows = vec![table_row(first, true)];
    let mut consumed = 1;
    for line in lines.iter().skip(1) {
        if !aligned_table_row(first, line) {
            break;
        }
        rows.push(table_row(line, false));
        consumed += 1;
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

fn tabular_line(line: &TextLine) -> bool {
    line.cells.len() >= 2
        && line
            .cells
            .windows(2)
            .all(|cells| cells[1].x - cells[0].x >= 36.0)
}

fn aligned_table_row(reference: &TextLine, candidate: &TextLine) -> bool {
    tabular_line(candidate)
        && reference.cells.len() == candidate.cells.len()
        && reference
            .cells
            .iter()
            .zip(&candidate.cells)
            .all(|(left, right)| (left.x - right.x).abs() <= 8.0)
}

fn table_row(line: &TextLine, header: bool) -> TableRow {
    TableRow {
        cells: line
            .cells
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_aligned_lines_to_table() {
        let segments = vec![
            segment("Model", 10.0, 100.0),
            segment("Current", 120.0, 100.0),
            segment("EUD-150", 10.0, 84.0),
            segment("700mA", 120.0, 84.0),
        ];

        let blocks = blocks_from_segments(&segments);

        let Block::Table(table) = &blocks[0] else {
            panic!("expected table");
        };
        assert_eq!(table.rows.len(), 2);
        assert!(table.rows[0].cells[0].header);
        assert_eq!(
            table.rows[1].cells[1].content,
            vec![Inline::Text("700mA".to_string())]
        );
    }

    #[test]
    fn marks_numeric_table_cells_as_right_aligned() {
        let segments = vec![
            segment("Name", 10.0, 100.0),
            segment("Count", 120.0, 100.0),
            segment("A", 10.0, 84.0),
            segment("300", 120.0, 84.0),
        ];

        let blocks = blocks_from_segments(&segments);

        let Block::Table(table) = &blocks[0] else {
            panic!("expected table");
        };
        assert_eq!(table.rows[1].cells[1].align, Some(TableAlignment::Right));
    }

    #[test]
    fn keeps_single_line_as_paragraph() {
        let segments = vec![
            segment("Default", 10.0, 100.0),
            segment("Current", 70.0, 100.0),
        ];

        assert!(matches!(
            blocks_from_segments(&segments)[0],
            Block::Paragraph(_)
        ));
    }

    #[test]
    fn merges_nearby_lines_into_paragraph() {
        let segments = vec![
            segment("This is the first line", 10.0, 100.0),
            segment("and this continues it.", 10.0, 86.0),
        ];

        let blocks = blocks_from_segments(&segments);

        let Block::Paragraph(paragraph) = &blocks[0] else {
            panic!("expected paragraph");
        };
        assert_eq!(
            paragraph.content,
            vec![Inline::Text(
                "This is the first line and this continues it.".to_string()
            )]
        );
    }

    #[test]
    fn infers_unordered_lists_from_markers() {
        let segments = vec![
            segment("- Alpha", 10.0, 100.0),
            segment("- Beta", 10.0, 86.0),
        ];

        let blocks = blocks_from_segments(&segments);

        let Block::List(list) = &blocks[0] else {
            panic!("expected list");
        };
        assert!(!list.ordered);
        assert_eq!(list.items.len(), 2);
    }

    #[test]
    fn infers_large_text_heading() {
        let segments = vec![
            TextSegment::new("Overview".to_string(), 10.0, 120.0, 18.0, 70.0),
            segment("Body text continues here.", 10.0, 96.0),
        ];

        let blocks = blocks_from_segments(&segments);

        assert!(matches!(blocks[0], Block::Heading(_)));
        assert!(matches!(blocks[1], Block::Paragraph(_)));
    }

    #[test]
    fn uses_tagged_heading_roles() {
        let segments = vec![
            TextSegment::new("Tagged".to_string(), 10.0, 120.0, 12.0, 42.0)
                .with_role(Some("H1".to_string())),
        ];

        let blocks = blocks_from_segments(&segments);

        let Block::Heading(heading) = &blocks[0] else {
            panic!("expected heading");
        };
        assert_eq!(heading.level, 1);
    }

    fn segment(text: &str, x: f32, y: f32) -> TextSegment {
        TextSegment {
            text: text.to_string(),
            x,
            y,
            font_size: 12.0,
            width: text.len() as f32 * 6.0,
            role: None,
        }
    }
}
