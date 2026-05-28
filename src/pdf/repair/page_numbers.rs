use super::super::text;
use super::line_groups::same_visual_text_line;
use super::segment_top;

pub(in crate::pdf) fn restore_centered_page_number_markers(
    page_width: f32,
    page_height: f32,
    segments: &mut [text::TextSegment],
) {
    if page_width <= 0.0 || page_height <= 0.0 || segments.len() < 2 {
        return;
    }

    let snapshot = segments.to_vec();
    for index in 0..segments.len() {
        let Some(page_number) = centered_page_number_text(&segments[index].text) else {
            continue;
        };
        if !is_centered_page_furniture_number(&segments[index], page_width, page_height)
            || !has_same_line_furniture_anchor(&snapshot, index, page_width)
        {
            continue;
        }

        let marker = format!("– {page_number} –");
        let center = segments[index].x + segments[index].width / 2.0;
        let width = text::estimated_text_width(&marker, segments[index].font_size);
        segments[index].text = marker;
        segments[index].width = width;
        segments[index].x = (center - width / 2.0).clamp(0.0, (page_width - width).max(0.0));
    }
}

fn centered_page_number_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.len() > 4 || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some(trimmed.to_string())
}

fn is_centered_page_furniture_number(
    segment: &text::TextSegment,
    page_width: f32,
    page_height: f32,
) -> bool {
    if segment.font_size > 14.0 {
        return false;
    }
    let top = segment_top(segment, page_height);
    let in_furniture_band = top <= page_height * 0.14 || top >= page_height * 0.84;
    if !in_furniture_band {
        return false;
    }
    let center = segment.x + segment.width / 2.0;
    (center - page_width / 2.0).abs() <= (page_width * 0.08).max(segment.font_size * 3.0)
}

fn has_same_line_furniture_anchor(
    segments: &[text::TextSegment],
    marker_index: usize,
    page_width: f32,
) -> bool {
    let marker = &segments[marker_index];
    segments.iter().enumerate().any(|(index, segment)| {
        if index == marker_index || !same_visual_text_line(marker, segment) {
            return false;
        }
        let text = segment.text.trim();
        !text.is_empty()
            && !text.chars().all(|ch| ch.is_ascii_digit())
            && (segment.x < page_width * 0.35 || segment.x + segment.width > page_width * 0.65)
    })
}
