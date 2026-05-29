mod cells;

use crate::{Inline, Table, TableAlignment, TableCell, TableRow};

use super::super::text::{repair_shifted_subset_text, TextLine};
use cells::{append_text, snap_to_columns, table_cells, tabular_cells, TableTextCell};

pub(super) fn parse_table_with_header(lines: &[TextLine], index: usize) -> Option<(Table, usize)> {
    let reference = discover_columns(lines, index)?;
    let first_line = lines.get(index)?;
    if starts_with_bullet_prose(first_line) {
        return None;
    }
    if !line_fits_columns(first_line, &reference) {
        return None;
    }

    let mut rows = Vec::new();
    let mut consumed = 0;
    let mut previous: Option<&TextLine> = None;
    let mut header_emitted = false;

    for line in lines.iter().skip(index) {
        if previous.is_some_and(|prev| table_row_gap(prev, line)) {
            break;
        }
        if row_originates_left_of_table(line, &reference) {
            break;
        }
        let cells = snap_to_columns(line, &reference);
        if row_belongs(&cells, &reference) {
            rows.push(table_row(&cells, !header_emitted));
            header_emitted = true;
            consumed += 1;
            previous = Some(line);
            continue;
        }
        if previous.is_some_and(|prev| extend_wrapped_table_cell(&reference, prev, line, &mut rows))
        {
            consumed += 1;
            previous = Some(line);
            continue;
        }
        break;
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

pub(super) fn tabular_line(line: &TextLine) -> bool {
    !starts_with_bullet_prose(line) && tabular_cells(&table_cells(line))
}

fn discover_columns(lines: &[TextLine], index: usize) -> Option<Vec<f32>> {
    let window: Vec<&TextLine> = lines.iter().skip(index).take(4).collect();
    if window.len() < 2 {
        return None;
    }

    let mut occurrences = Vec::new();
    for (line_index, line) in window.iter().enumerate() {
        occurrences.extend(
            table_cells(line)
                .into_iter()
                .map(|cell| (cell.x, line_index)),
        );
    }

    let columns = consensus_columns(&occurrences, window.len());
    (columns.len() >= 2).then_some(columns)
}

fn consensus_columns(occurrences: &[(f32, usize)], window_size: usize) -> Vec<f32> {
    let mut sorted = occurrences.to_vec();
    sorted.sort_by(|left, right| left.0.total_cmp(&right.0));

    let mut clusters: Vec<(f32, Vec<usize>)> = Vec::new();
    for (x, line) in sorted {
        match clusters.last_mut() {
            Some((column_x, lines_seen)) if (x - *column_x).abs() <= 4.0 => {
                if !lines_seen.contains(&line) {
                    lines_seen.push(line);
                }
                *column_x = column_x.min(x);
            }
            _ => clusters.push((x, vec![line])),
        }
    }

    let threshold = window_size.div_ceil(2).max(2);
    clusters
        .into_iter()
        .filter(|(_, lines_seen)| lines_seen.len() >= threshold)
        .map(|(x, _)| x)
        .collect()
}

fn line_fits_columns(line: &TextLine, reference: &[f32]) -> bool {
    !starts_left_of_table(line, reference)
        && row_belongs(&snap_to_columns(line, reference), reference)
}

fn row_belongs(cells: &[TableTextCell], reference: &[f32]) -> bool {
    cells.len() == reference.len()
        && cells.first().is_some_and(|cell| !cell.text.is_empty())
        && cells.iter().filter(|cell| !cell.text.is_empty()).count() >= 2
}

fn row_originates_left_of_table(line: &TextLine, reference: &[f32]) -> bool {
    starts_left_of_table(line, reference)
}

fn table_row_gap(previous: &TextLine, line: &TextLine) -> bool {
    let gap = previous.y - line.y;
    let line_height = previous.font_size.max(line.font_size).max(8.0);
    gap > line_height * 2.5
}

fn starts_left_of_table(line: &TextLine, reference: &[f32]) -> bool {
    match (line.cells.first(), reference.first()) {
        (Some(segment), Some(first_column)) => segment.x + 6.0 < *first_column,
        _ => false,
    }
}

fn table_cell_gap(font_size: f32) -> f32 {
    (font_size.max(8.0) * 1.75).max(18.0)
}

fn starts_with_bullet_prose(line: &TextLine) -> bool {
    let text = line.text.trim_start();
    if !(text.starts_with("• ") || text.starts_with("o ")) {
        return false;
    }
    text.split_whitespace()
        .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
        .count()
        >= 4
        && text.chars().any(char::is_lowercase)
}

fn extend_wrapped_table_cell(
    reference: &[f32],
    previous: &TextLine,
    line: &TextLine,
    rows: &mut [TableRow],
) -> bool {
    if rows.is_empty() || tabular_line(line) || super::is_list_line(line) {
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

fn table_row(cells: &[TableTextCell], header: bool) -> TableRow {
    TableRow {
        cells: cells
            .iter()
            .map(|cell| {
                let text = repair_table_cell_text(&cell.text);
                TableCell {
                    content: vec![Inline::Text(text.clone())],
                    header,
                    colspan: 1,
                    rowspan: 1,
                    align: table_alignment(&text),
                    source: None,
                }
            })
            .collect(),
        source: None,
    }
}

fn repair_table_cell_text(text: &str) -> String {
    let mut text = repair_shifted_subset_text(text);
    if looks_like_unbalanced_math_expression(&text) {
        let opens = text.chars().filter(|ch| *ch == '(').count();
        let closes = text.chars().filter(|ch| *ch == ')').count();
        for _ in closes..opens {
            text.push(')');
        }
    }
    text
}

fn looks_like_unbalanced_math_expression(text: &str) -> bool {
    let trimmed = text.trim_end();
    trimmed.contains('=')
        && trimmed.contains('(')
        && !trimmed.ends_with(')')
        && trimmed.chars().filter(|ch| *ch == '(').count()
            > trimmed.chars().filter(|ch| *ch == ')').count()
}

fn append_cell_text(cell: &mut TableCell, text: &str) {
    if let Some(Inline::Text(existing)) = cell.content.last_mut() {
        append_text(existing, text);
    } else {
        cell.content.push(Inline::Text(text.to_string()));
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
