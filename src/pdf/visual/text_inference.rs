mod formula;
mod heading;
mod title;

pub(super) use formula::inferred_formula_sum_markers;
pub(super) use heading::reconstructed_image_heading_lines;
pub(super) use title::inferred_centered_title_divider;

use super::super::graphics::RectShape;
use super::super::text::{self, TextSegment};
use super::{PageGeometry, VisualPage};
use crate::pdf::text::estimated_text_width;

pub(super) fn should_render_line_fragments(page: &VisualPage) -> bool {
    if has_large_image(page)
        || page
            .segments
            .iter()
            .any(|segment| normalized_rotation(segment.rotation).abs() >= 0.5)
    {
        return false;
    }
    if has_dense_grid(page) {
        return false;
    }

    let lines = text::text_lines(&page.segments);
    if has_bulleted_prose_lines(page, &lines) {
        return true;
    }
    if has_short_style_fragmented_prose(page, &lines) {
        return true;
    }
    if lines
        .iter()
        .any(super::text_layer::is_definition_prose_line)
        && page.shapes.len() <= 20
    {
        return true;
    }

    if lines.len() < 20 {
        return false;
    }

    let cell_count = lines.iter().map(|line| line.cells.len()).sum::<usize>();
    let page_width = page.width.unwrap_or(612.0);
    let long_lines = lines
        .iter()
        .filter(|line| super::text_layer::line_width(line) >= page_width * 0.55)
        .count();
    cell_count >= lines.len() * 2 && long_lines >= 12
}

pub(super) fn line_contains_segment(line: &text::TextLine, segment: &TextSegment) -> bool {
    line.cells.iter().any(|cell| {
        cell.text == segment.text
            && (cell.x - segment.x).abs() <= 0.01
            && (cell.y - segment.y).abs() <= 0.01
            && (cell.font_size - segment.font_size).abs() <= 0.01
    })
}

fn has_dense_grid(page: &VisualPage) -> bool {
    let horizontal_rules = page
        .shapes
        .iter()
        .filter(|shape| is_horizontal_rule(shape))
        .count();
    let vertical_rules = page
        .shapes
        .iter()
        .filter(|shape| is_vertical_rule(shape))
        .count();

    horizontal_rules >= 8 && vertical_rules >= 6
}

fn is_horizontal_rule(shape: &RectShape) -> bool {
    shape.height <= 1.5 && shape.width >= 20.0 && (shape.fill.is_some() || shape.stroke.is_some())
}

fn is_vertical_rule(shape: &RectShape) -> bool {
    shape.width <= 1.5 && shape.height >= 20.0 && (shape.fill.is_some() || shape.stroke.is_some())
}

fn has_bulleted_prose_lines(page: &VisualPage, lines: &[text::TextLine]) -> bool {
    let bullet_lines = lines
        .iter()
        .filter(|line| line.text.trim_start().starts_with("• "))
        .count();
    bullet_lines >= 4 && page.shapes.len() <= 20
}

fn has_short_style_fragmented_prose(page: &VisualPage, lines: &[text::TextLine]) -> bool {
    if page.shapes.len() > 20 {
        return false;
    }
    let page_width = page.width.unwrap_or(612.0);
    let prose_lines = lines
        .iter()
        .filter(|line| {
            line.cells.len() >= 2
                && (10.0..=13.5).contains(&line.font_size)
                && super::text_layer::line_width(line) >= page_width * 0.45
                && line
                    .text
                    .split_whitespace()
                    .filter(|word| word.chars().filter(|ch| ch.is_alphabetic()).count() >= 2)
                    .count()
                    >= 8
        })
        .count();
    prose_lines >= 3
}

pub(super) fn has_large_image(page: &VisualPage) -> bool {
    let page_area = page.width.unwrap_or(612.0) * page.height.unwrap_or(792.0);
    page_area > 0.0
        && page
            .images
            .iter()
            .any(|image| image.width * image.height >= page_area * 0.08)
}

pub(super) fn line_top(line: &text::TextLine, geometry: PageGeometry) -> f32 {
    geometry.height - line.y - line.font_size
}

pub(super) fn segment_top(segment: &TextSegment, geometry: PageGeometry) -> f32 {
    geometry.height - segment.y - segment.font_size
}

pub(super) fn normalized_rotation(rotation: f32) -> f32 {
    let mut value = rotation % 360.0;
    if value > 180.0 {
        value -= 360.0;
    } else if value <= -180.0 {
        value += 360.0;
    }
    value
}

pub(super) fn split_inferred_two_column_label(
    text: &str,
    left: f32,
    font_size: f32,
    geometry: PageGeometry,
) -> Option<(String, String, f32)> {
    if font_size > 10.0 || !has_domain_like_text(text) {
        return None;
    }
    let right_left = geometry.width * 0.51;
    if right_left <= left + geometry.width * 0.22 {
        return None;
    }

    let words = text.split_whitespace().collect::<Vec<_>>();
    let mut best = None;
    let target_width = right_left - left - font_size * 2.0;
    for split_at in 1..words.len() {
        let left_text = words[..split_at].join(" ");
        let right_text = words[split_at..].join(" ");
        if !looks_like_column_start(&right_text) || !has_domain_like_text(&right_text) {
            continue;
        }
        let error = (estimated_text_width(&left_text, font_size) - target_width).abs();
        if error <= font_size * 12.0
            && best
                .as_ref()
                .is_none_or(|(_, _, best_error)| error < *best_error)
        {
            best = Some((left_text, right_text, error));
        }
    }

    best.map(|(left_text, right_text, _)| (left_text, right_text, right_left))
}

pub(super) fn has_domain_like_text(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let word = word.trim_matches(|ch: char| {
            matches!(ch, ',' | ';' | ':' | ')' | '(' | '[' | ']' | '<' | '>')
        });
        word.contains("://")
            || word.starts_with("www.")
            || word.contains('@')
            || word.split('.').count() >= 3
    })
}

fn looks_like_column_start(text: &str) -> bool {
    let trimmed =
        text.trim_start_matches(|ch: char| matches!(ch, '-' | ':' | ';' | ',' | '(' | '['));
    trimmed
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        || trimmed.starts_with("www.")
        || trimmed.starts_with("http")
}
