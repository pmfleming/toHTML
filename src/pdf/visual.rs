mod render;
#[cfg(test)]
mod tests;
mod text_inference;
mod text_layer;
mod text_repair;

use super::graphics::{PathCommand, RectShape};
use super::text::TextSegment;

use render::{
    is_page_background_shape, path_points, push_pt, render_image, render_link, render_path,
    render_shape,
};
use text_inference::{
    inferred_centered_title_divider, inferred_formula_sum_markers, line_contains_segment,
    reconstructed_image_heading_lines, should_render_line_fragments,
};
use text_layer::{
    fill_sign_placeholder, render_fill_sign_line_cells, render_fragment, render_line_cells,
    render_line_fragment, should_render_line_cells,
};
use text_repair::annotation_aligned_url_segments;

#[derive(Debug, Clone)]
pub(super) struct VisualPage {
    pub page_number: u32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub segments: Vec<TextSegment>,
    pub shapes: Vec<RectShape>,
    pub images: Vec<VisualImage>,
    pub paths: Vec<VisualPath>,
    pub links: Vec<VisualLink>,
}

#[derive(Debug, Clone)]
pub(super) struct VisualImage {
    pub src: String,
    pub mask_src: Option<String>,
    pub alt: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub(super) struct VisualPath {
    pub commands: Vec<PathCommand>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub stroke_dasharray: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub(super) struct VisualLink {
    pub href: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub(super) fn render_pages(pages: &[VisualPage]) -> Option<String> {
    let mut html = String::new();
    let mut emitted = false;

    for page in pages.iter().filter(|page| {
        !page.segments.is_empty()
            || !page.shapes.is_empty()
            || !page.images.is_empty()
            || !page.paths.is_empty()
            || !page.links.is_empty()
    }) {
        emitted = true;
        let geometry = PageGeometry::from_page(page);
        let line_fragments = should_render_line_fragments(page);
        let text_segments = annotation_aligned_url_segments(page, geometry);
        let reconstructed_heading_lines = if line_fragments {
            Vec::new()
        } else {
            reconstructed_image_heading_lines(page, geometry)
        };
        html.push_str("    <section class=\"pdf-recreated-page");
        if line_fragments {
            html.push_str(" pdf-prose-page");
        }
        html.push_str("\" data-page=\"");
        html.push_str(&page.page_number.to_string());
        html.push_str("\" style=\"width:");
        push_pt(&mut html, geometry.width);
        html.push_str(";height:");
        push_pt(&mut html, geometry.height);
        html.push_str("\">\n");
        let suppress_white_text_color = false;
        let outline_filled_boxes = should_outline_filled_boxes(page, geometry);

        for shape in &page.shapes {
            if is_page_background_shape(shape, geometry) {
                render_shape(
                    &mut html,
                    shape,
                    geometry,
                    suppress_white_text_color,
                    outline_filled_boxes,
                );
            }
        }
        for path in page
            .paths
            .iter()
            .filter(|path| should_prepaint_large_filled_path(path, page, geometry))
        {
            render_path(&mut html, path, geometry, outline_filled_boxes);
        }
        for image in &page.images {
            render_image(&mut html, image, geometry);
        }
        for path in page
            .paths
            .iter()
            .filter(|path| should_prepaint_container_path(path, geometry))
            .filter(|path| !should_prepaint_large_filled_path(path, page, geometry))
        {
            render_path(&mut html, path, geometry, outline_filled_boxes);
        }
        for shape in &page.shapes {
            if is_page_background_shape(shape, geometry) {
                continue;
            }
            render_shape(
                &mut html,
                shape,
                geometry,
                suppress_white_text_color,
                outline_filled_boxes,
            );
        }
        if let Some(shape) = inferred_centered_title_divider(page, geometry) {
            render_shape(
                &mut html,
                &shape,
                geometry,
                suppress_white_text_color,
                outline_filled_boxes,
            );
        }
        for segment in inferred_formula_sum_markers(page, geometry) {
            render_fragment(&mut html, &segment, geometry, suppress_white_text_color);
        }
        for line in &reconstructed_heading_lines {
            render_line_fragment(&mut html, line, geometry, suppress_white_text_color);
        }
        if line_fragments {
            for line in super::text::text_lines(&text_segments) {
                if line
                    .cells
                    .iter()
                    .any(|cell| fill_sign_placeholder(&cell.text))
                {
                    render_fill_sign_line_cells(
                        &mut html,
                        &line,
                        geometry,
                        suppress_white_text_color,
                    );
                } else if should_render_line_cells(&line) {
                    render_line_cells(&mut html, &line, geometry, suppress_white_text_color);
                } else {
                    render_line_fragment(&mut html, &line, geometry, suppress_white_text_color);
                }
            }
        } else {
            for segment in &text_segments {
                if reconstructed_heading_lines
                    .iter()
                    .any(|line| line_contains_segment(line, segment))
                {
                    continue;
                }
                render_fragment(&mut html, segment, geometry, suppress_white_text_color);
            }
        }
        for path in page
            .paths
            .iter()
            .filter(|path| !should_prepaint_large_filled_path(path, page, geometry))
            .filter(|path| !should_prepaint_container_path(path, geometry))
        {
            render_path(&mut html, path, geometry, outline_filled_boxes);
        }
        for link in &page.links {
            render_link(&mut html, link, geometry);
        }

        html.push_str("    </section>\n");
    }

    emitted.then_some(html)
}

fn should_prepaint_container_path(path: &VisualPath, geometry: PageGeometry) -> bool {
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

fn should_prepaint_large_filled_path(
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
struct VisualBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn path_bounds(path: &VisualPath) -> Option<VisualBounds> {
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

fn should_outline_filled_boxes(page: &VisualPage, geometry: PageGeometry) -> bool {
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

#[derive(Debug, Clone, Copy)]
struct PageGeometry {
    width: f32,
    height: f32,
    min_x: f32,
}

impl PageGeometry {
    fn from_page(page: &VisualPage) -> Self {
        let max_x = page
            .segments
            .iter()
            .map(|segment| segment.x + segment.width.max(0.0))
            .chain(
                page.shapes
                    .iter()
                    .map(|shape| shape.x + shape.width.max(0.0)),
            )
            .chain(
                page.images
                    .iter()
                    .map(|image| image.x + image.width.max(0.0)),
            )
            .chain(
                page.paths
                    .iter()
                    .flat_map(|path| path_points(&path.commands).map(|point| point.0)),
            )
            .chain(page.links.iter().map(|link| link.x + link.width.max(0.0)))
            .fold(0.0_f32, f32::max);
        let max_y = page
            .segments
            .iter()
            .map(|segment| segment.y + segment.font_size.max(0.0))
            .chain(
                page.shapes
                    .iter()
                    .map(|shape| shape.y + shape.height.max(0.0)),
            )
            .chain(
                page.images
                    .iter()
                    .map(|image| image.y + image.height.max(0.0)),
            )
            .chain(
                page.paths
                    .iter()
                    .flat_map(|path| path_points(&path.commands).map(|point| point.1)),
            )
            .chain(page.links.iter().map(|link| link.y + link.height.max(0.0)))
            .fold(0.0_f32, f32::max);
        let min_x = page
            .segments
            .iter()
            .map(|segment| segment.x)
            .chain(page.shapes.iter().map(|shape| shape.x))
            .chain(page.images.iter().map(|image| image.x))
            .chain(
                page.paths
                    .iter()
                    .flat_map(|path| path_points(&path.commands).map(|point| point.0)),
            )
            .chain(page.links.iter().map(|link| link.x))
            .fold(f32::INFINITY, f32::min)
            .min(0.0);

        Self {
            width: page
                .width
                .unwrap_or_else(|| (max_x + 72.0).max(612.0))
                .ceil(),
            height: page
                .height
                .unwrap_or_else(|| (max_y + 72.0).max(792.0))
                .ceil(),
            min_x,
        }
    }
}
