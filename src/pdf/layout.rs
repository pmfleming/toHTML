use crate::{Block, Inline, Paragraph, Table, TableCell, TableRow};

use super::text::{text_lines, TextLine, TextSegment};

pub fn blocks_from_segments(segments: &[TextSegment]) -> Vec<Block> {
    let lines = text_lines(segments);
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if let Some((table, consumed)) = parse_table(&lines[index..]) {
            blocks.push(Block::Table(table));
            index += consumed;
        } else {
            blocks.push(paragraph(&lines[index].text));
            index += 1;
        }
    }

    blocks
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
                align: None,
                source: None,
            })
            .collect(),
        source: None,
    }
}

fn paragraph(text: &str) -> Block {
    Block::Paragraph(Paragraph {
        content: vec![Inline::Text(text.to_string())],
        source: None,
    })
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

    fn segment(text: &str, x: f32, y: f32) -> TextSegment {
        TextSegment {
            text: text.to_string(),
            x,
            y,
            font_size: 12.0,
            width: text.len() as f32 * 6.0,
        }
    }
}
