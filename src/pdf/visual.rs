mod diagrams;
mod formulas;
mod render;
#[cfg(test)]
mod tests;
mod text_layer;
mod text_repair;

use super::graphics::{PathCommand, RectShape};
use super::text::TextSegment;

use diagrams::{
    diagram_overlays, is_iec_flowchart_page, is_iec_formula_definition_page,
    path_intersects_diagram, render_diagram_overlay, shape_intersects_diagram,
};
use formulas::{
    formula_overlays, path_intersects_formula, render_formula_overlay, shape_intersects_formula,
};
use render::{
    is_page_background_shape, path_points, push_pt, render_image, render_link, render_path,
    render_shape,
};
use text_layer::{
    fill_sign_placeholder, line_width, render_fill_sign_line_cells, render_fragment,
    render_line_cells, render_line_fragment, should_render_line_cells,
};

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
        let formulas = formula_overlays(page);
        let diagrams = diagram_overlays(page);
        let flowchart_page = is_iec_flowchart_page(page);
        let suppress_white_text_color = flowchart_page || !diagrams.is_empty();

        for shape in &page.shapes {
            if is_page_background_shape(shape, geometry)
                && !shape_intersects_formula(shape, geometry, formulas)
                && !shape_intersects_diagram(shape, geometry, diagrams)
            {
                render_shape(&mut html, shape, geometry, flowchart_page);
            }
        }
        for image in &page.images {
            render_image(&mut html, image, geometry);
        }
        for shape in &page.shapes {
            if is_page_background_shape(shape, geometry) {
                continue;
            }
            if !shape_intersects_formula(shape, geometry, formulas)
                && !shape_intersects_diagram(shape, geometry, diagrams)
            {
                render_shape(&mut html, shape, geometry, flowchart_page);
            }
        }
        for diagram in diagrams {
            render_diagram_overlay(&mut html, diagram);
        }
        if line_fragments {
            for line in super::text::text_lines(&page.segments) {
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
            for segment in &page.segments {
                render_fragment(&mut html, segment, geometry, suppress_white_text_color);
            }
        }
        for path in &page.paths {
            if !path_intersects_formula(path, geometry, formulas)
                && !path_intersects_diagram(path, geometry, diagrams)
            {
                render_path(&mut html, path, geometry);
            }
        }
        for formula in formulas {
            render_formula_overlay(&mut html, formula);
        }
        for link in &page.links {
            render_link(&mut html, link, geometry);
        }

        html.push_str("    </section>\n");
    }

    emitted.then_some(html)
}

fn should_render_line_fragments(page: &VisualPage) -> bool {
    if is_iec_formula_definition_page(page) {
        return true;
    }

    if has_large_image(page)
        || page
            .segments
            .iter()
            .any(|segment| normalized_rotation(segment.rotation).abs() >= 0.5)
    {
        return false;
    }

    let lines = super::text::text_lines(&page.segments);
    if lines.len() < 20 {
        return false;
    }

    let cell_count = lines.iter().map(|line| line.cells.len()).sum::<usize>();
    let page_width = page.width.unwrap_or(612.0);
    let long_lines = lines
        .iter()
        .filter(|line| line_width(line) >= page_width * 0.55)
        .count();
    cell_count >= lines.len() * 2 && long_lines >= 12
}

fn has_large_image(page: &VisualPage) -> bool {
    let page_area = page.width.unwrap_or(612.0) * page.height.unwrap_or(792.0);
    page_area > 0.0
        && page
            .images
            .iter()
            .any(|image| image.width * image.height >= page_area * 0.08)
}

fn normalized_rotation(rotation: f32) -> f32 {
    let mut value = rotation % 360.0;
    if value > 180.0 {
        value -= 360.0;
    } else if value <= -180.0 {
        value += 360.0;
    }
    value
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
