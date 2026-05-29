use super::super::PageGeometry;
use super::{push_pt, render_fragment};
use crate::pdf::text::{TextLine, TextSegment};

pub(in crate::pdf::visual) fn render_fill_sign_line_cells(
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

pub(in crate::pdf::visual) fn fill_sign_placeholder(text: &str) -> bool {
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
