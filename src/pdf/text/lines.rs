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

    group_lines(segments)
        .into_iter()
        .map(to_text_line)
        .filter(|line| !line.text.is_empty())
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub text: String,
    pub cells: Vec<TextSegment>,
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
}

pub fn estimated_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.45
}

fn group_lines(segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
    let mut lines: Vec<Vec<TextSegment>> = Vec::new();
    for segment in segments {
        match lines
            .iter_mut()
            .find(|line| same_line(line_y(line), segment.y))
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
    TextLine {
        text: join_line_segments(&cells),
        cells,
        x,
        y,
        font_size,
    }
}

fn line_y(line: &[TextSegment]) -> f32 {
    line.first().map(|item| item.y).unwrap_or_default()
}

fn same_line(left: f32, right: f32) -> bool {
    (left - right).abs() <= 3.0
}

fn join_line_segments(segments: &[TextSegment]) -> String {
    let mut text = String::new();
    let mut previous_end = None;
    for segment in segments {
        if let Some(end) = previous_end {
            push_gap(&mut text, segment.x - end);
        }
        text.push_str(&segment.text);
        previous_end = Some(segment.x + segment.width);
    }
    normalize_whitespace(&text)
}

fn push_gap(text: &mut String, gap: f32) {
    if gap > 6.0 && !text.ends_with(' ') {
        text.push(' ');
    }
}
