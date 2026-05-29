mod paint;
mod render;
#[cfg(test)]
mod tests;
mod text_inference;
mod text_layer;
mod text_repair;

use super::graphics::{PathCommand, RectShape};
use super::text::TextSegment;

pub(in crate::pdf::visual) use paint::path_bounds;
use paint::{
    should_outline_filled_boxes, should_prepaint_container_path, should_prepaint_large_filled_path,
};
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
