use crate::{Inline, Table, TableAlignment, TableCell, TableRow};

use super::inlines::parse_inlines;
use super::source::markdown_source;

pub fn table_start(lines: &[&str], index: usize) -> bool {
    let Some(header) = lines.get(index) else {
        return false;
    };
    let Some(separator) = lines.get(index + 1) else {
        return false;
    };
    header.contains('|') && parse_alignments(separator).is_some()
}

pub fn parse_table(lines: &[&str]) -> (Table, usize) {
    let headers = split_cells(lines[0]);
    let alignments = parse_alignments(lines[1]).unwrap_or_default();
    let mut rows = vec![table_row(&headers, &alignments, true)];
    let mut consumed = 2;

    while let Some(line) = lines.get(consumed) {
        if !line.contains('|') || line.trim().is_empty() {
            break;
        }
        rows.push(table_row(&split_cells(line), &alignments, false));
        consumed += 1;
    }

    (
        Table {
            rows,
            caption: None,
            source: markdown_source(),
        },
        consumed,
    )
}

fn table_row(cells: &[String], alignments: &[Option<TableAlignment>], header: bool) -> TableRow {
    TableRow {
        cells: cells
            .iter()
            .enumerate()
            .map(|(index, cell)| table_cell(cell, alignments.get(index).copied().flatten(), header))
            .collect(),
        source: markdown_source(),
    }
}

fn table_cell(text: &str, align: Option<TableAlignment>, header: bool) -> TableCell {
    TableCell {
        content: parse_cell_inlines(text),
        header,
        colspan: 1,
        rowspan: 1,
        align,
        source: markdown_source(),
    }
}

fn parse_cell_inlines(text: &str) -> Vec<Inline> {
    parse_inlines(text.trim())
}

fn split_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn parse_alignments(line: &str) -> Option<Vec<Option<TableAlignment>>> {
    let cells = split_cells(line);
    if cells.is_empty() || !cells.iter().all(|cell| separator_cell(cell)) {
        return None;
    }
    Some(cells.iter().map(|cell| alignment(cell)).collect())
}

fn separator_cell(cell: &str) -> bool {
    let trimmed = cell.trim();
    let dashes = trimmed.chars().filter(|ch| *ch == '-').count();
    dashes >= 3 && trimmed.chars().all(|ch| matches!(ch, '-' | ':'))
}

fn alignment(cell: &str) -> Option<TableAlignment> {
    let trimmed = cell.trim();
    match (trimmed.starts_with(':'), trimmed.ends_with(':')) {
        (true, true) => Some(TableAlignment::Center),
        (true, false) => Some(TableAlignment::Left),
        (false, true) => Some(TableAlignment::Right),
        (false, false) => None,
    }
}
