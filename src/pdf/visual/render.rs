use super::super::graphics::{PathCommand, RectShape};
use super::{PageGeometry, VisualImage, VisualLink, VisualPath};
use crate::html::escape::push_attr_escaped;

pub(super) fn render_path(html: &mut String, path: &VisualPath, geometry: PageGeometry) {
    if path.commands.len() < 2 {
        return;
    }

    html.push_str("      <svg class=\"pdf-ink\" style=\"left:0;top:0;width:");
    push_pt(html, geometry.width);
    html.push_str(";height:");
    push_pt(html, geometry.height);
    html.push_str("\" viewBox=\"0 0 ");
    push_number(html, geometry.width);
    html.push(' ');
    push_number(html, geometry.height);
    html.push_str("\" aria-hidden=\"true\"><path d=\"");
    render_path_data(html, path, geometry);
    html.push_str("\" fill=\"");
    if let Some(fill) = &path.fill {
        push_attr_escaped(html, fill);
    } else {
        html.push_str("none");
    }
    html.push('"');
    if let Some(stroke) = &path.stroke {
        html.push_str(" stroke=\"");
        push_attr_escaped(html, stroke);
        html.push_str("\" stroke-width=\"");
        push_number(html, path.stroke_width.max(0.25));
        html.push_str("\" stroke-linecap=\"round\" stroke-linejoin=\"round\"");
        if let Some(dasharray) = &path.stroke_dasharray {
            if !dasharray.is_empty() {
                html.push_str(" stroke-dasharray=\"");
                for (index, value) in dasharray.iter().enumerate() {
                    if index > 0 {
                        html.push(' ');
                    }
                    push_number(html, *value);
                }
                html.push('"');
            }
        }
    }
    html.push_str("/></svg>\n");
}

fn render_path_data(html: &mut String, path: &VisualPath, geometry: PageGeometry) {
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                html.push('M');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::LineTo(x, y) => {
                html.push('L');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
                html.push('C');
                push_path_point(html, x1, y1, geometry);
                html.push(' ');
                push_path_point(html, x2, y2, geometry);
                html.push(' ');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::Close => html.push('Z'),
        }
    }
}

fn push_path_point(html: &mut String, x: f32, y: f32, geometry: PageGeometry) {
    push_number(html, (x - geometry.min_x).max(0.0));
    html.push(' ');
    push_number(html, (geometry.height - y).max(0.0));
}

pub(super) fn path_points(commands: &[PathCommand]) -> impl Iterator<Item = (f32, f32)> + '_ {
    commands.iter().flat_map(|command| match *command {
        PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => {
            vec![(x, y)]
        }
        PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
            vec![(x1, y1), (x2, y2), (x, y)]
        }
        PathCommand::Close => Vec::new(),
    })
}

pub(super) fn render_image(html: &mut String, image: &VisualImage, geometry: PageGeometry) {
    let left = (image.x - geometry.min_x).max(0.0);
    let top = (geometry.height - image.y - image.height).max(0.0);
    let width = image.width.max(0.0);
    let height = image.height.max(0.0);
    if left > geometry.width || top > geometry.height || width < 0.25 || height < 0.25 {
        return;
    }

    html.push_str("      <img class=\"pdf-image\"");
    html.push_str(" src=\"");
    push_attr_escaped(html, &image.src);
    html.push_str("\" alt=\"");
    push_attr_escaped(html, &image.alt);
    html.push('"');
    html.push_str(" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";width:");
    push_pt(html, width.min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, height.min(geometry.height - top));
    if let Some(mask_src) = &image.mask_src {
        html.push_str(";-webkit-mask-image:url(&quot;");
        push_attr_escaped(html, mask_src);
        html.push_str("&quot;);-webkit-mask-size:100% 100%;mask-image:url(&quot;");
        push_attr_escaped(html, mask_src);
        html.push_str("&quot;);mask-size:100% 100%");
    }
    html.push_str("\">\n");
}

pub(super) fn render_link(html: &mut String, link: &VisualLink, geometry: PageGeometry) {
    let left = (link.x - geometry.min_x).max(0.0);
    let top = (geometry.height - link.y - link.height).max(0.0);
    let width = link.width.max(0.0);
    let height = link.height.max(0.0);
    if left > geometry.width || top > geometry.height || width < 0.25 || height < 0.25 {
        return;
    }

    let clipped_width = width.min(geometry.width - left);
    let clipped_height = height.min(geometry.height - top);

    html.push_str("      <a class=\"pdf-link-overlay\" href=\"");
    push_attr_escaped(html, &link.href);
    html.push_str("\" aria-label=\"");
    push_attr_escaped(html, &link.href);
    html.push_str("\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";width:");
    push_pt(html, clipped_width);
    html.push_str(";height:");
    push_pt(html, clipped_height);
    html.push_str("\"></a>\n");
}

pub(super) fn render_shape(
    html: &mut String,
    shape: &RectShape,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    let left = (shape.x - geometry.min_x).max(0.0);
    let top = (geometry.height - shape.y - shape.height).max(0.0);
    let width = shape.width.max(0.0);
    let height = shape.height.max(0.0);
    if left > geometry.width || top > geometry.height || width < 0.25 || height < 0.25 {
        return;
    }

    html.push_str("      <div class=\"pdf-shape\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";width:");
    push_pt(html, width.min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, height.min(geometry.height - top));
    if flowchart_page && is_misfilled_flowchart_box(shape) {
        html.push_str(";background:#ffffff;border:0.75pt solid #000000");
    } else {
        if let Some(fill) = &shape.fill {
            html.push_str(";background:");
            push_css_color(html, fill);
        }
        if let Some(stroke) = &shape.stroke {
            html.push_str(";border:0.75pt solid ");
            push_css_color(html, stroke);
        }
    }
    html.push_str("\"></div>\n");
}

pub(super) fn is_page_background_shape(shape: &RectShape, geometry: PageGeometry) -> bool {
    shape.fill.is_some()
        && shape.stroke.is_none()
        && shape.width >= geometry.width * 0.95
        && shape.height >= geometry.height * 0.95
}

fn is_misfilled_flowchart_box(shape: &RectShape) -> bool {
    matches!(shape.fill.as_deref(), Some("#000000" | "#000"))
        && matches!(shape.stroke.as_deref(), Some("#000000" | "#000") | None)
        && shape.width > 10.0
        && shape.height > 8.0
}

pub(super) fn push_pt(html: &mut String, value: f32) {
    push_number(html, value.max(0.0));
    html.push_str("pt");
}

pub(super) fn push_number(html: &mut String, value: f32) {
    html.push_str(&format!("{value:.2}"));
}

pub(super) fn push_css_color(html: &mut String, value: &str) {
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    {
        html.push_str(value);
    }
}

pub(super) fn push_text_color(html: &mut String, value: Option<&str>, flowchart_page: bool) {
    let Some(value) = value else {
        return;
    };
    if flowchart_page && value.eq_ignore_ascii_case("#ffffff") {
        return;
    }
    html.push_str(";color:");
    push_css_color(html, value);
}
