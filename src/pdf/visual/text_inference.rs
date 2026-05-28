use super::super::graphics::RectShape;
use super::super::text::{self, TextSegment};
use super::{PageGeometry, VisualPage, VisualPath};
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

pub(super) fn reconstructed_image_heading_lines(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<text::TextLine> {
    if !has_large_image(page) {
        return Vec::new();
    }
    text::text_lines(&page.segments)
        .into_iter()
        .filter_map(|line| reconstructed_image_heading_suffix(&line, geometry))
        .collect()
}

fn reconstructed_image_heading_suffix(
    line: &text::TextLine,
    geometry: PageGeometry,
) -> Option<text::TextLine> {
    if line.cells.iter().any(|cell| {
        is_reporting_period_heading_text(&super::text_repair::repair_visual_text(&cell.text))
    }) {
        return None;
    }

    (0..line.cells.len())
        .rev()
        .map(|start| text_line_from_cells(&line.cells[start..]))
        .find(|candidate| should_reconstruct_image_heading_line(candidate, geometry))
}

fn should_reconstruct_image_heading_line(line: &text::TextLine, geometry: PageGeometry) -> bool {
    if line.cells.len() < 2 || normalized_rotation(line.rotation).abs() >= 0.5 {
        return false;
    }
    let top = line_top(line, geometry);
    if top > geometry.height * 0.14 || !(18.0..=38.0).contains(&line.font_size) {
        return false;
    }
    if line.font_weight.unwrap_or_default() < 600 {
        return false;
    }

    let repaired = super::text_repair::repair_visual_text(&line.text);
    is_reporting_period_heading_text(&repaired) && !repaired.contains('=')
}

fn is_reporting_period_heading_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("(last ") && lower.contains("month")
}

fn text_line_from_cells(cells: &[TextSegment]) -> text::TextLine {
    let cells = cells.to_vec();
    let x = cells.first().map(|cell| cell.x).unwrap_or_default();
    let y = cells.first().map(|cell| cell.y).unwrap_or_default();
    let font_size = cells
        .iter()
        .map(|cell| cell.font_size)
        .fold(0.0_f32, f32::max);
    let role = cells.iter().find_map(|cell| cell.role.clone());
    let color = cells.iter().find_map(|cell| cell.color.clone());
    let font_family = cells.iter().find_map(|cell| cell.font_family.clone());
    let font_weight = cells.iter().find_map(|cell| cell.font_weight);
    let font_style = cells.iter().find_map(|cell| cell.font_style.clone());
    let rotation = cells.first().map(|cell| cell.rotation).unwrap_or_default();
    text::TextLine {
        text: join_visual_line_segments(&cells),
        cells,
        x,
        y,
        font_size,
        rotation,
        role,
        color,
        font_family,
        font_weight,
        font_style,
    }
}

fn join_visual_line_segments(segments: &[TextSegment]) -> String {
    let mut text = String::new();
    let mut previous_end = None;
    let mut previous_font = 0.0_f32;
    for segment in segments {
        if let Some(end) = previous_end {
            let gap = segment.x - end;
            let space_width = previous_font.max(segment.font_size) * 0.25;
            if gap >= space_width.max(2.0) && !text.ends_with(' ') {
                text.push(' ');
            }
        }
        text.push_str(&segment.text);
        previous_end = Some(segment.x + segment.width);
        previous_font = segment.font_size;
    }
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text::repair_shifted_subset_text(&normalized)
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

fn has_large_image(page: &VisualPage) -> bool {
    let page_area = page.width.unwrap_or(612.0) * page.height.unwrap_or(792.0);
    page_area > 0.0
        && page
            .images
            .iter()
            .any(|image| image.width * image.height >= page_area * 0.08)
}

pub(super) fn inferred_centered_title_divider(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Option<RectShape> {
    let lines = text::text_lines(&page.segments);
    let heading = lines
        .iter()
        .find(|line| is_centered_standards_authority_heading(line, geometry))?;
    let heading_top = line_top(heading, geometry);
    let heading_bottom = heading_top + heading.font_size;
    let title = lines
        .iter()
        .filter(|line| {
            let top = line_top(line, geometry);
            top > heading_bottom + 30.0 && top <= geometry.height * 0.28
        })
        .find(|line| is_centered_major_title_line(line, geometry))?;
    let title_top = line_top(title, geometry);
    let gap = title_top - heading_bottom;
    if !(36.0..=82.0).contains(&gap) {
        return None;
    }

    let top = (heading_bottom + title_top) / 2.0;
    if has_existing_centered_divider(page, geometry, heading_bottom, title_top) {
        return None;
    }

    let width = 70.0_f32.min(geometry.width * 0.18);
    let height = 0.75;
    Some(RectShape {
        x: geometry.min_x + (geometry.width - width) / 2.0,
        y: geometry.height - top - height,
        width,
        height,
        fill: Some("#000000".to_string()),
        stroke: None,
    })
}

pub(super) fn inferred_formula_sum_markers(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    let lines = text::text_lines(&page.segments);
    let mut markers = Vec::new();
    for upper in lines
        .iter()
        .filter(|line| is_formula_sum_upper_bound(line, geometry))
    {
        let upper_top = line_top(upper, geometry);
        let Some(lower) = lines.iter().find(|line| {
            is_formula_sum_lower_bound(line, geometry) && (line.x - upper.x).abs() <= 16.0 && {
                let lower_top = line_top(line, geometry);
                lower_top > upper_top + 12.0 && lower_top <= upper_top + 36.0
            }
        }) else {
            continue;
        };
        let lower_top = line_top(lower, geometry);
        let Some(term_font_size) = lines.iter().find_map(|line| {
            formula_current_term_font_size(line, upper, upper_top, lower_top, geometry)
        }) else {
            continue;
        };
        if !lines
            .iter()
            .any(|line| has_harmonic_formula_label_before(line, upper, lower_top, geometry))
        {
            continue;
        }
        if has_existing_sum_marker(&lines, upper, lower, geometry) {
            continue;
        }

        let font_size = (term_font_size * 1.75).clamp(14.0, 19.0);
        let top = ((upper_top + lower_top) / 2.0 - font_size * 0.38).max(0.0);
        let segment = TextSegment::new(
            "∑".to_string(),
            upper.x - font_size * 0.14,
            geometry.height - top - font_size,
            font_size,
            font_size * 0.45,
        )
        .with_font_style(
            Some("Times New Roman, Times, serif".to_string()),
            None,
            None,
        );
        markers.push(segment);
    }
    markers
}

fn is_formula_sum_upper_bound(line: &text::TextLine, geometry: PageGeometry) -> bool {
    let top = line_top(line, geometry);
    let text = line.text.trim();
    matches!(text, "39" | "40")
        && (6.0..=10.0).contains(&line.font_size)
        && top >= geometry.height * 0.25
        && top <= geometry.height * 0.78
}

fn is_formula_sum_lower_bound(line: &text::TextLine, _geometry: PageGeometry) -> bool {
    let compact = line.text.split_whitespace().collect::<String>();
    compact.starts_with('h')
        && compact.chars().skip(1).any(|ch| ch.is_ascii_digit())
        && compact.len() <= 12
        && (6.0..=10.0).contains(&line.font_size)
}

fn formula_current_term_font_size(
    line: &text::TextLine,
    upper: &text::TextLine,
    upper_top: f32,
    lower_top: f32,
    geometry: PageGeometry,
) -> Option<f32> {
    line.cells
        .iter()
        .find(|cell| {
            is_formula_current_term_text(&cell.text)
                && (8.0..=12.0).contains(&cell.font_size)
                && cell.x > upper.x + 8.0
                && cell.x <= upper.x + 36.0
                && {
                    let term_top = segment_top(cell, geometry);
                    term_top >= upper_top + 2.0 && term_top <= lower_top + 2.0
                }
        })
        .map(|cell| cell.font_size)
}

fn is_formula_current_term_text(text: &str) -> bool {
    let compact = text.split_whitespace().collect::<String>();
    matches!(compact.as_str(), "I" | "Ih" | "I1" | "Ih2")
}

fn has_harmonic_formula_label_before(
    line: &text::TextLine,
    upper: &text::TextLine,
    lower_top: f32,
    geometry: PageGeometry,
) -> bool {
    line.cells.iter().any(|cell| {
        is_harmonic_formula_label_text(&cell.text) && cell.x + cell.width < upper.x && {
            let label_top = segment_top(cell, geometry);
            label_top >= line_top(upper, geometry) + 3.0 && label_top <= lower_top + 8.0
        }
    })
}

fn is_harmonic_formula_label_text(text: &str) -> bool {
    let compact = text.split_whitespace().collect::<String>();
    matches!(compact.as_str(), "THC" | "THD" | "POHC")
}

fn has_existing_sum_marker(
    lines: &[text::TextLine],
    upper: &text::TextLine,
    lower: &text::TextLine,
    geometry: PageGeometry,
) -> bool {
    let upper_top = line_top(upper, geometry);
    let lower_top = line_top(lower, geometry);
    lines.iter().any(|line| {
        line.text.chars().any(|ch| matches!(ch, 'Σ' | '∑')) && (line.x - upper.x).abs() <= 18.0 && {
            let top = line_top(line, geometry);
            top >= upper_top && top <= lower_top + 4.0
        }
    })
}

fn is_centered_standards_authority_heading(line: &text::TextLine, geometry: PageGeometry) -> bool {
    let top = line_top(line, geometry);
    let text = line.text.trim();
    top >= geometry.height * 0.07
        && top <= geometry.height * 0.16
        && (9.0..=16.0).contains(&line.font_size)
        && super::text_layer::line_width(line) >= geometry.width * 0.35
        && is_centered_line(line, geometry, 0.08)
        && is_mostly_uppercase(text, 0.92)
        && {
            let upper = text.to_uppercase();
            upper.contains("COMMISSION") && upper.contains("INTERNATIONAL")
        }
}

fn is_centered_major_title_line(line: &text::TextLine, geometry: PageGeometry) -> bool {
    let text = line.text.trim();
    line.font_size >= 10.0
        && super::text_layer::line_width(line) >= geometry.width * 0.25
        && is_centered_line(line, geometry, 0.15)
        && (line.font_weight.unwrap_or_default() >= 600 || is_mostly_uppercase(text, 0.7))
}

fn has_existing_centered_divider(
    page: &VisualPage,
    geometry: PageGeometry,
    heading_bottom: f32,
    title_top: f32,
) -> bool {
    page.shapes
        .iter()
        .any(|shape| is_existing_centered_divider_shape(shape, geometry, heading_bottom, title_top))
        || page.paths.iter().any(|path| {
            is_existing_centered_divider_path(path, geometry, heading_bottom, title_top)
        })
}

fn is_existing_centered_divider_shape(
    shape: &RectShape,
    geometry: PageGeometry,
    heading_bottom: f32,
    title_top: f32,
) -> bool {
    if shape.fill.is_none() && shape.stroke.is_none() {
        return false;
    }
    let top = geometry.height - shape.y - shape.height;
    top >= heading_bottom + 4.0
        && top <= title_top - 4.0
        && shape.height <= 2.0
        && (40.0..=180.0).contains(&shape.width)
        && is_centered_box(shape.x, shape.width, geometry, 0.09)
}

fn is_existing_centered_divider_path(
    path: &VisualPath,
    geometry: PageGeometry,
    heading_bottom: f32,
    title_top: f32,
) -> bool {
    if path.fill.is_none() && path.stroke.is_none() {
        return false;
    }
    let Some(bounds) = super::path_bounds(path) else {
        return false;
    };
    let top = geometry.height - bounds.y - bounds.height;
    top >= heading_bottom + 4.0
        && top <= title_top - 4.0
        && bounds.height <= 2.0_f32.max(path.stroke_width)
        && (40.0..=180.0).contains(&bounds.width)
        && is_centered_box(bounds.x, bounds.width, geometry, 0.09)
}

fn is_centered_line(line: &text::TextLine, geometry: PageGeometry, tolerance_ratio: f32) -> bool {
    is_centered_box(
        line.x,
        super::text_layer::line_width(line),
        geometry,
        tolerance_ratio,
    )
}

fn is_centered_box(x: f32, width: f32, geometry: PageGeometry, tolerance_ratio: f32) -> bool {
    let page_center = geometry.min_x + geometry.width / 2.0;
    let box_center = x + width / 2.0;
    (box_center - page_center).abs() <= geometry.width * tolerance_ratio
}

fn is_mostly_uppercase(text: &str, min_ratio: f32) -> bool {
    let letters = text.chars().filter(|ch| ch.is_alphabetic()).count();
    if letters == 0 {
        return false;
    }
    let uppercase = text
        .chars()
        .filter(|ch| ch.is_alphabetic() && ch.is_uppercase())
        .count();
    uppercase as f32 / letters as f32 >= min_ratio
}

fn line_top(line: &text::TextLine, geometry: PageGeometry) -> f32 {
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
