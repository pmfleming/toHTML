use super::super::super::text::{self, TextSegment};
use super::super::text_repair;
use super::super::{PageGeometry, VisualPage};
use super::{has_large_image, line_top, normalized_rotation};

pub(in crate::pdf::visual) fn reconstructed_image_heading_lines(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<text::TextLine> {
    if !has_large_image(page) {
        return Vec::new();
    }
    text::text_lines(&page.segments)
        .into_iter()
        .filter_map(|line| reconstructed_image_heading_suffix(&line, geometry))
        .collect()
}

fn reconstructed_image_heading_suffix(
    line: &text::TextLine,
    geometry: PageGeometry,
) -> Option<text::TextLine> {
    if line
        .cells
        .iter()
        .any(|cell| is_reporting_period_heading_text(&text_repair::repair_visual_text(&cell.text)))
    {
        return None;
    }

    (0..line.cells.len())
        .rev()
        .map(|start| text_line_from_cells(&line.cells[start..]))
        .find(|candidate| should_reconstruct_image_heading_line(candidate, geometry))
}

fn should_reconstruct_image_heading_line(line: &text::TextLine, geometry: PageGeometry) -> bool {
    if line.cells.len() < 2 || normalized_rotation(line.rotation).abs() >= 0.5 {
        return false;
    }
    let top = line_top(line, geometry);
    if top > geometry.height * 0.14 || !(18.0..=38.0).contains(&line.font_size) {
        return false;
    }
    if line.font_weight.unwrap_or_default() < 600 {
        return false;
    }

    let repaired = text_repair::repair_visual_text(&line.text);
    is_reporting_period_heading_text(&repaired) && !repaired.contains('=')
}

fn is_reporting_period_heading_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("(last ") && lower.contains("month")
}

fn text_line_from_cells(cells: &[TextSegment]) -> text::TextLine {
    let cells = cells.to_vec();
    let x = cells.first().map(|cell| cell.x).unwrap_or_default();
    let y = cells.first().map(|cell| cell.y).unwrap_or_default();
    let font_size = cells
        .iter()
        .map(|cell| cell.font_size)
        .fold(0.0_f32, f32::max);
    let role = cells.iter().find_map(|cell| cell.role.clone());
    let color = cells.iter().find_map(|cell| cell.color.clone());
    let font_family = cells.iter().find_map(|cell| cell.font_family.clone());
    let font_weight = cells.iter().find_map(|cell| cell.font_weight);
    let font_style = cells.iter().find_map(|cell| cell.font_style.clone());
    let rotation = cells.first().map(|cell| cell.rotation).unwrap_or_default();
    text::TextLine {
        text: join_visual_line_segments(&cells),
        cells,
        x,
        y,
        font_size,
        rotation,
        role,
        color,
        font_family,
        font_weight,
        font_style,
    }
}

fn join_visual_line_segments(segments: &[TextSegment]) -> String {
    let mut text = String::new();
    let mut previous_end = None;
    let mut previous_font = 0.0_f32;
    for segment in segments {
        if let Some(end) = previous_end {
            let gap = segment.x - end;
            let space_width = previous_font.max(segment.font_size) * 0.25;
            if gap >= space_width.max(2.0) && !text.ends_with(' ') {
                text.push(' ');
            }
        }
        text.push_str(&segment.text);
        previous_end = Some(segment.x + segment.width);
        previous_font = segment.font_size;
    }
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text::repair_shifted_subset_text(&normalized)
}
