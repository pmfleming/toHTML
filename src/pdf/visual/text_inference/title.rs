use super::super::super::graphics::RectShape;
use super::super::super::text;
use super::super::path_bounds;
use super::super::text_layer::line_width;
use super::super::{PageGeometry, VisualPage, VisualPath};
use super::line_top;

pub(in crate::pdf::visual) fn inferred_centered_title_divider(
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

fn is_centered_standards_authority_heading(line: &text::TextLine, geometry: PageGeometry) -> bool {
    let top = line_top(line, geometry);
    let text = line.text.trim();
    top >= geometry.height * 0.07
        && top <= geometry.height * 0.16
        && (9.0..=16.0).contains(&line.font_size)
        && line_width(line) >= geometry.width * 0.35
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
        && line_width(line) >= geometry.width * 0.25
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
    let Some(bounds) = path_bounds(path) else {
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
    is_centered_box(line.x, line_width(line), geometry, tolerance_ratio)
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
