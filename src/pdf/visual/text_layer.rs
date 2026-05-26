use super::super::text::{TextLine, TextSegment};
use super::render::{push_number, push_pt, push_text_color};
use super::text_repair::repair_visual_text;
use super::{normalized_rotation, PageGeometry};
use crate::html::escape::push_attr_escaped;

pub(super) fn render_line_fragment(
    html: &mut String,
    line: &TextLine,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    let text = repair_visual_text(&line.text);
    let font_size = line.font_size.clamp(4.0, 48.0);
    if is_iec_license_artifact(&text, font_size) {
        return;
    }
    let left = (line.x - geometry.min_x).max(0.0);
    let top = (geometry.height - line.y - font_size).max(0.0);
    if left > geometry.width || top > geometry.height {
        return;
    }

    html.push_str("      <span class=\"pdf-text-fragment\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";font-size:");
    push_pt(html, font_size);
    html.push_str(";width:");
    push_pt(html, line_width(line).min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, font_size * 1.12);
    match line.role.as_deref() {
        Some("Strong") => html.push_str(";font-weight:700"),
        Some("Em") => html.push_str(";font-style:italic"),
        _ => {}
    }
    push_pdf_font_style(
        html,
        line.font_family.as_deref(),
        line.font_weight,
        line.font_style.as_deref(),
    );
    push_text_color(html, line.color.as_deref(), flowchart_page);
    html.push_str("\">");
    push_attr_escaped(html, &text);
    html.push_str("</span>\n");
}

pub(super) fn should_render_line_cells(line: &TextLine) -> bool {
    if has_email_address(&line.text) {
        return true;
    }
    if line.text.chars().count() > 70 {
        return false;
    }
    if line.text.contains('©') {
        return false;
    }
    line.cells.windows(2).any(|cells| {
        let left_end = cells[0].x + cells[0].width.max(0.0);
        cells[1].x - left_end >= line.font_size.max(8.0) * 4.0
    })
}

fn has_email_address(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let trimmed = word.trim_matches(|ch: char| {
            matches!(ch, ',' | ';' | ':' | '<' | '>' | '(' | ')' | '[' | ']')
        });
        trimmed.contains('@') && trimmed.rsplit_once('.').is_some()
    })
}

pub(super) fn render_line_cells(
    html: &mut String,
    line: &TextLine,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    for cell in &line.cells {
        render_fragment(html, cell, geometry, flowchart_page);
    }
}

pub(super) fn render_fill_sign_line_cells(
    html: &mut String,
    line: &TextLine,
    geometry: PageGeometry,
    flowchart_page: bool,
) {
    for cell in &line.cells {
        let text = without_fill_sign_placeholders(&cell.text);
        if fill_sign_placeholder(&cell.text) {
            render_placeholder_line(html, cell, geometry);
        }
        if !text.trim().is_empty() {
            let mut segment = cell.clone();
            segment.text = text.trim().to_string();
            render_fragment(html, &segment, geometry, flowchart_page);
        }
    }
}

fn render_placeholder_line(html: &mut String, segment: &TextSegment, geometry: PageGeometry) {
    let left = (segment.x - geometry.min_x).max(0.0);
    let top = (geometry.height - segment.y + 1.0).max(0.0);
    if left > geometry.width || top > geometry.height {
        return;
    }

    html.push_str("      <div class=\"pdf-shape\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";width:");
    push_pt(
        html,
        segment
            .width
            .min(geometry.width - left)
            .max(segment.font_size * 2.0),
    );
    html.push_str(";height:0.75pt;background:#000000\"></div>\n");
}

pub(super) fn fill_sign_placeholder(text: &str) -> bool {
    let mut run = 0;
    for ch in text.chars() {
        if ch == 'B' {
            run += 1;
            if run >= 4 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn without_fill_sign_placeholders(text: &str) -> String {
    let mut cleaned = String::new();
    let mut run = String::new();
    for ch in text.chars() {
        if ch == 'B' {
            run.push(ch);
            continue;
        }
        flush_fill_sign_run(&mut cleaned, &mut run);
        cleaned.push(ch);
    }
    flush_fill_sign_run(&mut cleaned, &mut run);
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn flush_fill_sign_run(cleaned: &mut String, run: &mut String) {
    if run.len() < 4 {
        cleaned.push_str(run);
    } else if !cleaned.ends_with(' ') {
        cleaned.push(' ');
    }
    run.clear();
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
    let text = repair_visual_text(&segment.text);
    let font_size = segment.font_size.clamp(4.0, 48.0);
    if is_iec_license_artifact(&text, font_size) {
        return;
    }
    let width = segment.width.max(font_size * 0.5);
    let left = (segment.x - geometry.min_x).max(0.0);
    let top = (geometry.height - segment.y - font_size).max(0.0);
    let rotation = normalized_rotation(segment.rotation);
    let natural_width = super::super::text::estimated_text_width(&text, font_size).max(1.0);
    let scale_x = (width / natural_width).clamp(0.25, 1.75);
    let should_scale_x = scale_x < 0.8 || (scale_x > 1.25 && font_size > 9.5);
    if left > geometry.width || top > geometry.height {
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
    }
    html.push_str("\">");
    push_attr_escaped(html, &text);
    html.push_str("</span>\n");
}

fn is_iec_license_artifact(text: &str, font_size: f32) -> bool {
    font_size <= 5.0
        && (text.contains("Provided by IHS Markit")
            || text.contains("Not for Resale")
            || text.contains("No reproduction or networking permitted")
            || text.contains("Copyright International Electrotechnical Commission")
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
