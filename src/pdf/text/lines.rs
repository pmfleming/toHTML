use super::strings::normalize_whitespace;
use super::types::TextSegment;

#[cfg(test)]
pub fn segments_to_text(segments: &[TextSegment]) -> String {
    text_lines(segments)
        .into_iter()
        .map(|line| line.text)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn text_lines(segments: &[TextSegment]) -> Vec<TextLine> {
    let mut segments = segments.to_vec();
    segments.sort_by(|left, right| {
        right
            .y
            .total_cmp(&left.y)
            .then_with(|| left.x.total_cmp(&right.x))
    });
    let segments = dedupe_overlapping_segments(segments);

    group_lines(segments)
        .into_iter()
        .map(to_text_line)
        .filter(|line| !line.text.is_empty())
        .collect()
}

fn dedupe_overlapping_segments(segments: Vec<TextSegment>) -> Vec<TextSegment> {
    let mut unique = Vec::new();
    for segment in segments {
        if !unique
            .iter()
            .any(|existing| duplicate_segment(existing, &segment))
        {
            unique.push(segment);
        }
    }
    unique
}

fn duplicate_segment(left: &TextSegment, right: &TextSegment) -> bool {
    left.text == right.text && (left.x - right.x).abs() <= 1.0 && (left.y - right.y).abs() <= 1.0
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub text: String,
    pub cells: Vec<TextSegment>,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub rotation: f32,
    pub role: Option<String>,
}

pub fn estimated_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.45
}

fn group_lines(segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
    let mut lines: Vec<Vec<TextSegment>> = Vec::new();
    for segment in segments {
        match lines
            .iter_mut()
            .find(|line| same_line(line_y(line), segment.y) && same_rotation(line, &segment))
        {
            Some(line) => line.push(segment),
            None => lines.push(vec![segment]),
        }
    }
    lines
}

fn to_text_line(mut cells: Vec<TextSegment>) -> TextLine {
    cells.sort_by(|left, right| left.x.total_cmp(&right.x));
    let x = cells.first().map(|cell| cell.x).unwrap_or_default();
    let y = line_y(&cells);
    let font_size = cells
        .iter()
        .map(|cell| cell.font_size)
        .fold(0.0_f32, f32::max);
    let role = cells.iter().find_map(|cell| cell.role.clone());
    let rotation = cells.first().map(|cell| cell.rotation).unwrap_or_default();
    TextLine {
        text: join_line_segments(&cells),
        cells,
        x,
        y,
        font_size,
        rotation,
        role,
    }
}

fn line_y(line: &[TextSegment]) -> f32 {
    line.first().map(|item| item.y).unwrap_or_default()
}

fn same_line(left: f32, right: f32) -> bool {
    (left - right).abs() <= 3.0
}

fn same_rotation(line: &[TextSegment], segment: &TextSegment) -> bool {
    line.first().is_none_or(|first| {
        (rotation_bucket(first.rotation) - rotation_bucket(segment.rotation)).abs() <= 1
    })
}

fn rotation_bucket(rotation: f32) -> i32 {
    (rotation / 15.0).round() as i32
}

fn join_line_segments(segments: &[TextSegment]) -> String {
    let mut text = String::new();
    let mut previous_end = None;
    let mut previous_font = 0.0_f32;
    for segment in segments {
        if let Some(end) = previous_end {
            push_gap(
                &mut text,
                segment.x - end,
                previous_font.max(segment.font_size),
            );
        }
        text.push_str(&segment.text);
        previous_end = Some(segment.x + segment.width);
        previous_font = segment.font_size;
    }
    normalize_whitespace(&text)
}

fn push_gap(text: &mut String, gap: f32, font_size: f32) {
    let space_width = (font_size * 0.25).max(2.0);
    if gap >= space_width && !text.ends_with(' ') {
        text.push(' ');
    }
}
