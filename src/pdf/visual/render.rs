mod path;

pub(super) use path::{path_points, render_path};

use super::super::graphics::RectShape;
use super::{PageGeometry, VisualImage, VisualLink};
use crate::html::escape::push_attr_escaped;

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
    outline_filled_boxes: bool,
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
    if (flowchart_page && is_misfilled_flowchart_box(shape))
        || (outline_filled_boxes && is_filled_box_outline_candidate(shape))
    {
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

fn is_filled_box_outline_candidate(shape: &RectShape) -> bool {
    matches!(shape.fill.as_deref(), Some("#000000" | "#000"))
        && shape.stroke.is_none()
        && shape.width >= 6.0
        && shape.height >= 8.0
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
