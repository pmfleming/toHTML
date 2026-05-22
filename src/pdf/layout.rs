use crate::{
    Block, Heading, Inline, List, ListItem, Paragraph, RawHtml, Table, TableAlignment, TableCell,
    TableRow,
};

use super::text::{text_lines, TextLine, TextSegment};

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
    push_escaped_html(&mut html, &line.text);
    html.push_str("</p>");
    Block::RawHtml(RawHtml { html, source: None })
}

fn push_escaped_html(output: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
}

fn parse_table_with_header(lines: &[TextLine], index: usize) -> Option<(Table, usize)> {
    let reference = discover_columns(lines, index)?;
    // The starting line must itself look like a row in this column structure.
    // This prevents the detector from forming a table that begins on a paragraph
    // sitting above the real table.
    let first_line = lines.get(index)?;
    if !line_fits_columns(first_line, &reference) {
        return None;
    }
    let mut rows: Vec<TableRow> = Vec::new();
    let mut consumed = 0;
    let mut previous: Option<&TextLine> = None;
    let mut header_emitted = false;

    for line in lines.iter().skip(index) {
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
        if let Some(prev) = previous {
            if extend_wrapped_table_cell(&reference, prev, line, &mut rows) {
                consumed += 1;
                previous = Some(line);
                continue;
            }
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

fn discover_columns(lines: &[TextLine], index: usize) -> Option<Vec<f32>> {
    let window: Vec<&TextLine> = lines.iter().skip(index).take(4).collect();
    if window.len() < 2 {
        return None;
    }

    // Collect raw segment x positions per line. Column boundaries are positions
    // that recur across multiple lines within tolerance — incidental word
    // starts inside one cell will not align with anything in other rows.
    let mut occurrences: Vec<(f32, usize)> = Vec::new();
    for (line_index, line) in window.iter().enumerate() {
        for segment in &line.cells {
            occurrences.push((segment.x, line_index));
        }
    }

    let columns = consensus_columns(&occurrences, window.len());
    (columns.len() >= 2).then_some(columns)
}

fn consensus_columns(occurrences: &[(f32, usize)], window_size: usize) -> Vec<f32> {
    let mut sorted: Vec<(f32, usize)> = occurrences.to_vec();
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

    // Require a column to appear in more than half the window (rounded up),
    // with an absolute floor of 2. This filters out trailing-digit positions
    // that happen to recur in only a couple of rows.
    let threshold = window_size.div_ceil(2).max(2);
    clusters
        .into_iter()
        .filter(|(_, lines_seen)| lines_seen.len() >= threshold)
        .map(|(x, _)| x)
        .collect()
}

fn snap_to_columns(line: &TextLine, columns: &[f32]) -> Vec<TableTextCell> {
    let mut cells: Vec<TableTextCell> = columns
        .iter()
        .map(|x| TableTextCell {
            text: String::new(),
            x: *x,
            width: 0.0,
        })
        .collect();
    // Iterate raw segments rather than clustered cells so that header words like
    // "Version" and "Date" (which cluster together because their gap is below
    // the cluster threshold) still land in their own columns.
    for segment in &line.cells {
        let column = nearest_column_index(columns, segment.x);
        let cell = &mut cells[column];
        if cell.text.is_empty() {
            cell.text = segment.text.clone();
            cell.x = segment.x;
            cell.width = segment.width;
        } else {
            append_text(&mut cell.text, &segment.text);
            cell.width = (segment.x + segment.width - cell.x).max(cell.width);
        }
    }
    cells
}

fn nearest_column_index(columns: &[f32], x: f32) -> usize {
    columns
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (x - **left).abs().total_cmp(&(x - **right).abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn line_fits_columns(line: &TextLine, reference: &[f32]) -> bool {
    // The line must start at or near the table's first column. A line starting
    // significantly left of the table's left edge is a page-margin paragraph,
    // not a table row.
    if let Some(first_segment) = line.cells.first() {
        if let Some(first_column) = reference.first() {
            if first_segment.x + 6.0 < *first_column {
                return false;
            }
        }
    }
    let cells = snap_to_columns(line, reference);
    cells.first().is_some_and(|cell| !cell.text.is_empty())
        && cells.iter().filter(|cell| !cell.text.is_empty()).count() >= 2
}

fn row_belongs(cells: &[TableTextCell], reference: &[f32]) -> bool {
    // A real row should have content in the leftmost column and at least one more.
    // This stops wrap lines (which only have content in the middle column) from
    // looking like new rows.
    cells.len() == reference.len()
        && cells.first().is_some_and(|cell| !cell.text.is_empty())
        && cells.iter().filter(|cell| !cell.text.is_empty()).count() >= 2
}

fn row_originates_left_of_table(line: &TextLine, reference: &[f32]) -> bool {
    match (line.cells.first(), reference.first()) {
        (Some(segment), Some(first_column)) => segment.x + 6.0 < *first_column,
        _ => false,
    }
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
