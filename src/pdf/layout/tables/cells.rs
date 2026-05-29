use super::super::super::text::{TextLine, TextSegment};
use super::table_cell_gap;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct TableTextCell {
    pub(super) text: String,
    pub(super) x: f32,
    pub(super) width: f32,
}

pub(super) fn snap_to_columns(line: &TextLine, columns: &[f32]) -> Vec<TableTextCell> {
    let mut cells = empty_cells(columns);
    for segment in &line.cells {
        let column = nearest_column_index(columns, segment.x);
        append_segment_to_cell(&mut cells[column], segment);
    }
    cells
}

fn empty_cells(columns: &[f32]) -> Vec<TableTextCell> {
    columns
        .iter()
        .map(|x| TableTextCell {
            text: String::new(),
            x: *x,
            width: 0.0,
        })
        .collect()
}

fn append_segment_to_cell(cell: &mut TableTextCell, segment: &TextSegment) {
    if cell.text.is_empty() {
        cell.text = segment.text.clone();
        cell.x = segment.x;
        cell.width = segment.width;
    } else {
        append_text(&mut cell.text, &segment.text);
        cell.width = (segment.x + segment.width - cell.x).max(cell.width);
    }
}

fn nearest_column_index(columns: &[f32], x: f32) -> usize {
    columns
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (x - **left).abs().total_cmp(&(x - **right).abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

pub(super) fn table_cells(line: &TextLine) -> Vec<TableTextCell> {
    let mut cells = Vec::new();
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

pub(super) fn tabular_cells(cells: &[TableTextCell]) -> bool {
    cells.len() >= 2
        && cells
            .windows(2)
            .all(|cells| cells[1].x - cells[0].x >= 24.0)
}

pub(super) fn append_text(existing: &mut String, next: &str) {
    if !existing.ends_with(' ') && !next.starts_with(' ') {
        existing.push(' ');
    }
    existing.push_str(next);
}
