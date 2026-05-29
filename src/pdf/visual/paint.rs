use super::super::graphics::{PathCommand, RectShape};
use super::render::{is_page_background_shape, path_points};
use super::{PageGeometry, VisualImage, VisualPage, VisualPath};

pub(super) fn should_prepaint_container_path(path: &VisualPath, geometry: PageGeometry) -> bool {
    if path.fill.is_none() || path.stroke.is_some() {
        return false;
    }
    let Some(bounds) = path_bounds(path) else {
        return false;
    };
    let page_area = geometry.width * geometry.height;
    page_area > 0.0
        && bounds.width * bounds.height >= page_area * 0.015
        && bounds.width >= geometry.width * 0.12
        && bounds.height >= geometry.height * 0.035
}

pub(super) fn should_prepaint_large_filled_path(
    path: &VisualPath,
    page: &VisualPage,
    geometry: PageGeometry,
) -> bool {
    if path.fill.is_none() || path.stroke.is_some() {
        return false;
    }
    let Some(bounds) = path_bounds(path) else {
        return false;
    };
    let page_area = geometry.width * geometry.height;
    if page_area <= 0.0 {
        return false;
    }
    let path_area = bounds.width * bounds.height;
    let has_large_image = page
        .images
        .iter()
        .any(|image| image.width * image.height >= page_area * 0.25);
    if has_large_image
        && is_neutral_fill(path.fill.as_deref())
        && bounds.width >= geometry.width * 0.2
        && bounds.height >= geometry.height * 0.2
    {
        return true;
    }

    if has_curve(path) || path_area < page_area * 0.18 {
        return false;
    }

    page.images
        .iter()
        .filter(|image| image.width * image.height >= page_area * 0.25)
        .any(|image| rect_overlap_area(bounds, image_rect(image)) >= path_area * 0.85)
}

fn is_neutral_fill(fill: Option<&str>) -> bool {
    let Some(fill) = fill else {
        return false;
    };
    let Some((red, green, blue)) = hex_color(fill) else {
        return false;
    };
    red.abs_diff(green) <= 6 && red.abs_diff(blue) <= 6 && green.abs_diff(blue) <= 6
}

fn hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some((
        u8::from_str_radix(&value[0..2], 16).ok()?,
        u8::from_str_radix(&value[2..4], 16).ok()?,
        u8::from_str_radix(&value[4..6], 16).ok()?,
    ))
}

fn has_curve(path: &VisualPath) -> bool {
    path.commands
        .iter()
        .any(|command| matches!(command, PathCommand::CubicTo(..)))
}

#[derive(Debug, Clone, Copy)]
pub(in crate::pdf::visual) struct VisualBounds {
    pub(in crate::pdf::visual) x: f32,
    pub(in crate::pdf::visual) y: f32,
    pub(in crate::pdf::visual) width: f32,
    pub(in crate::pdf::visual) height: f32,
}

pub(in crate::pdf::visual) fn path_bounds(path: &VisualPath) -> Option<VisualBounds> {
    let mut points = path_points(&path.commands);
    let first = points.next()?;
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (first.0, first.0, first.1, first.1);
    for (x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    Some(VisualBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

fn image_rect(image: &VisualImage) -> VisualBounds {
    VisualBounds {
        x: image.x,
        y: image.y,
        width: image.width,
        height: image.height,
    }
}

fn rect_overlap_area(left: VisualBounds, right: VisualBounds) -> f32 {
    let x1 = left.x.max(right.x);
    let y1 = left.y.max(right.y);
    let x2 = (left.x + left.width).min(right.x + right.width);
    let y2 = (left.y + left.height).min(right.y + right.height);
    if x2 <= x1 || y2 <= y1 {
        0.0
    } else {
        (x2 - x1) * (y2 - y1)
    }
}

pub(super) fn should_outline_filled_boxes(page: &VisualPage, geometry: PageGeometry) -> bool {
    let black_boxes = page
        .shapes
        .iter()
        .filter(|shape| is_black_filled_box_candidate(shape, geometry))
        .count();
    if black_boxes < 2 {
        return false;
    }

    let white_inner_boxes = page
        .shapes
        .iter()
        .filter(|shape| is_white_filled_box(shape))
        .filter(|white| {
            page.shapes.iter().any(|black| {
                is_black_filled_box_candidate(black, geometry) && contains_box(black, white)
            })
        })
        .count();

    white_inner_boxes >= 2
}

fn is_black_filled_box_candidate(shape: &RectShape, geometry: PageGeometry) -> bool {
    !is_page_background_shape(shape, geometry)
        && matches!(shape.fill.as_deref(), Some("#000000" | "#000"))
        && shape.stroke.is_none()
        && shape.width >= 6.0
        && shape.height >= 8.0
}

fn is_white_filled_box(shape: &RectShape) -> bool {
    matches!(shape.fill.as_deref(), Some("#ffffff" | "#fff"))
        && shape.stroke.is_none()
        && shape.width >= 3.0
        && shape.height >= 3.0
}

fn contains_box(outer: &RectShape, inner: &RectShape) -> bool {
    let tolerance = 1.5;
    inner.x + tolerance >= outer.x
        && inner.y + tolerance >= outer.y
        && inner.x + inner.width <= outer.x + outer.width + tolerance
        && inner.y + inner.height <= outer.y + outer.height + tolerance
        && inner.width <= outer.width
        && inner.height <= outer.height
}
