mod fill_sign;
mod line_rules;

pub(super) use fill_sign::{fill_sign_placeholder, render_fill_sign_line_cells};
pub(super) use line_rules::{is_definition_prose_line, should_render_line_cells};

use super::super::text::{TextLine, TextSegment};
use super::render::{push_number, push_pt, push_text_color};
use super::text_repair::repair_visual_text;
use super::{
    text_inference::{normalized_rotation, split_inferred_two_column_label},
    PageGeometry,
};
use crate::html::escape::push_attr_escaped;
use crate::pdf::text::estimated_text_width;

pub(super) fn render_line_fragment(
    html: &mut String,
    line: &TextLine,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    let text = repair_visual_text(&line.text);
    let font_size = line.font_size.clamp(4.0, 48.0);
    if is_tiny_license_artifact(&text, font_size) {
        return;
    }
    let left = (line.x - geometry.min_x).max(0.0);
    let top = (geometry.height - line.y - font_size).max(0.0);
    if left > geometry.width || top > geometry.height {
        return;
    }

    let available_width = geometry.width - left;
    let natural_width = visual_text_width(&text, font_size, line.font_family.as_deref()).max(1.0);
    let scale_x = if natural_width > available_width {
        Some((available_width / natural_width).clamp(0.25, 1.0))
    } else {
        None
    };
    let width = if scale_x.is_some() {
        available_width
    } else {
        line_width(line).min(available_width)
    };

    if let Some((left_text, right_text, right_left)) =
        split_inferred_two_column_label(&text, left, font_size, geometry)
    {
        render_positioned_text_span(
            html,
            &left_text,
            left,
            top,
            font_size,
            (right_left - left).max(font_size),
            line.role.as_deref(),
            line.font_family.as_deref(),
            line.font_weight,
            line.font_style.as_deref(),
            line.color.as_deref(),
            flowchart_page,
            None,
        );
        render_positioned_text_span(
            html,
            &right_text,
            right_left,
            top,
            font_size,
            (geometry.width - right_left).max(font_size),
            line.role.as_deref(),
            line.font_family.as_deref(),
            line.font_weight,
            line.font_style.as_deref(),
            line.color.as_deref(),
            flowchart_page,
            None,
        );
        return;
    }

    render_positioned_text_span(
        html,
        &text,
        left,
        top,
        font_size,
        width,
        line.role.as_deref(),
        line.font_family.as_deref(),
        line.font_weight,
        if is_definition_prose_line(line) {
            None
        } else {
            line.font_style.as_deref()
        },
        line.color.as_deref(),
        flowchart_page,
        scale_x,
    );
}

fn visual_text_width(text: &str, font_size: f32, font_family: Option<&str>) -> f32 {
    let monospaced = font_family.is_some_and(|family| {
        let family = family.to_ascii_lowercase();
        family.contains("mono") || family.contains("courier") || family.contains("consolas")
    });
    if monospaced {
        text.chars().count() as f32 * font_size * 0.6
    } else {
        estimated_text_width(text, font_size)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_positioned_text_span(
    html: &mut String,
    text: &str,
    left: f32,
    top: f32,
    font_size: f32,
    width: f32,
    role: Option<&str>,
    font_family: Option<&str>,
    font_weight: Option<u16>,
    font_style: Option<&str>,
    color: Option<&str>,
    flowchart_page: bool,
    scale_x: Option<f32>,
) {
    html.push_str("      <span class=\"pdf-text-fragment\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";font-size:");
    push_pt(html, font_size);
    html.push_str(";width:");
    push_pt(html, width);
    html.push_str(";height:");
    push_pt(html, font_size * 1.12);
    match role {
        Some("Strong") => html.push_str(";font-weight:700"),
        Some("Em") => html.push_str(";font-style:italic"),
        _ => {}
    }
    push_pdf_font_style(html, font_family, font_weight, font_style);
    push_text_color(html, color, flowchart_page);
    if let Some(scale_x) = scale_x {
        html.push_str(";transform:scaleX(");
        push_number(html, scale_x);
        html.push(')');
    }
    html.push_str("\">");
    push_attr_escaped(html, &text);
    html.push_str("</span>\n");
}

pub(super) fn render_line_cells(
    html: &mut String,
    line: &TextLine,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    let mut previous = None;
    let mut inferred_gap_offset = 0.0;
    for cell in &line.cells {
        let mut segment = cell.clone();
        if previous.is_some_and(|previous| needs_inserted_word_gap(previous, cell)) {
            inferred_gap_offset += segment.font_size * 0.25;
            segment.x += inferred_gap_offset;
            segment.text = format!(" {}", segment.text);
            segment.width += segment.font_size * 0.25;
        } else {
            segment.x += inferred_gap_offset;
        }
        render_fragment(html, &segment, geometry, flowchart_page);
        previous = Some(cell);
    }
}

pub(super) fn needs_inserted_word_gap(left: &TextSegment, right: &TextSegment) -> bool {
    let gap = right.x - (left.x + left.width.max(0.0));
    gap.abs() <= left.font_size.max(right.font_size) * 0.08
        && left
            .text
            .trim_end()
            .chars()
            .last()
            .is_some_and(|ch| ch.is_alphanumeric())
        && right
            .text
            .trim_start()
            .chars()
            .next()
            .is_some_and(|ch| ch.is_alphanumeric())
}

pub(super) fn line_width(line: &TextLine) -> f32 {
    let right = line
        .cells
        .iter()
        .map(|cell| cell.x + cell.width.max(0.0))
        .fold(line.x, f32::max);
    (right - line.x).max(line.font_size * 0.5)
}

pub(super) fn render_fragment(
    html: &mut String,
    segment: &TextSegment,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    let mut text = repair_visual_text(&segment.text);
    if segment.text.starts_with(char::is_whitespace) && !text.starts_with(char::is_whitespace) {
        text.insert(0, '\u{00a0}');
    }
    let font_size = segment.font_size.clamp(4.0, 48.0);
    if is_tiny_license_artifact(&text, font_size) {
        return;
    }
    let width = segment.width.max(font_size * 0.5);
    let left = (segment.x - geometry.min_x).max(0.0);
    let top = (geometry.height - segment.y - font_size).max(0.0);
    let rotation = normalized_rotation(segment.rotation);
    let natural_width = super::super::text::estimated_text_width(&text, font_size).max(1.0);
    let edge_aligned = left + width >= geometry.width - font_size * 0.5;
    let scale_x = if edge_aligned && text.chars().count() >= 40 {
        (width / natural_width).clamp(0.25, 0.92)
    } else {
        (width / natural_width).clamp(0.25, 1.75)
    };
    let should_scale_x =
        scale_x < 0.8 || (edge_aligned && scale_x < 1.0) || (scale_x > 1.25 && font_size > 9.5);
    if left > geometry.width || top > geometry.height {
        return;
    }
    if is_standalone_checkbox_marker(&text) {
        render_checkbox_marker(html, left, top, font_size);
        return;
    }

    html.push_str("      <span class=\"pdf-text-fragment\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";font-size:");
    push_pt(html, font_size);
    html.push_str(";width:");
    push_pt(html, width.min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, font_size * 1.12);
    match segment.role.as_deref() {
        Some("Strong") => html.push_str(";font-weight:700"),
        Some("Em") => html.push_str(";font-style:italic"),
        _ => {}
    }
    push_pdf_font_style(
        html,
        segment.font_family.as_deref(),
        segment.font_weight,
        segment.font_style.as_deref(),
    );
    push_text_color(html, segment.color.as_deref(), flowchart_page);
    if rotation.abs() >= 0.5 || should_scale_x {
        html.push_str(";transform:");
        if rotation.abs() >= 0.5 {
            html.push_str("rotate(");
            push_number(html, rotation);
            html.push_str("deg)");
        }
        if should_scale_x {
            if rotation.abs() >= 0.5 {
                html.push(' ');
            }
            html.push_str("scaleX(");
            push_number(html, scale_x);
            html.push(')');
        }
        html.push_str(";transform-origin:left top");
    }
    html.push_str("\">");
    push_attr_escaped(html, &text);
    html.push_str("</span>\n");
}

fn is_standalone_checkbox_marker(text: &str) -> bool {
    matches!(text.trim(), "□" | "☐")
}

fn render_checkbox_marker(html: &mut String, left: f32, top: f32, font_size: f32) {
    let size = (font_size * 0.6).clamp(8.0, 18.0);
    let border = (font_size * 0.1).clamp(1.25, 3.0);
    html.push_str("      <span class=\"pdf-text-fragment pdf-checkbox-marker\" style=\"left:");
    push_pt(html, left + font_size * 0.06);
    html.push_str(";top:");
    push_pt(html, top + font_size * 0.24);
    html.push_str(";width:");
    push_pt(html, size);
    html.push_str(";height:");
    push_pt(html, size);
    html.push_str(";border:");
    push_pt(html, border);
    html.push_str(" solid #000000;box-sizing:border-box;background:#ffffff\"></span>\n");
}

fn is_tiny_license_artifact(text: &str, font_size: f32) -> bool {
    font_size <= 5.0
        && (text.contains("Provided by IHS Markit")
            || text.contains("Not for Resale")
            || text.contains("No reproduction or networking permitted")
            || text.contains("Copyright ")
            || text.contains("--`,```"))
}

fn push_pdf_font_style(
    html: &mut String,
    family: Option<&str>,
    weight: Option<u16>,
    style: Option<&str>,
) {
    if let Some(family) = family {
        html.push_str(";font-family:");
        html.push_str(family);
    }
    if let Some(weight) = weight {
        html.push_str(";font-weight:");
        html.push_str(&weight.to_string());
    }
    if let Some(style) = style {
        html.push_str(";font-style:");
        html.push_str(style);
    }
}
