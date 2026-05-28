mod emitter;
mod lines;
mod operands;
mod parser;
mod reader;
mod state;
mod strings;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use std::collections::HashMap;

use super::cmap::CMap;
use super::fonts::FontMetrics;

pub use lines::{text_lines, TextLine};
pub use types::TextSegment;

pub(super) use lines::estimated_text_width;

pub fn decode_string(bytes: &[u8]) -> String {
    strings::decode_pdf_string(bytes)
}

#[cfg(test)]
pub fn extract_text(stream: &[u8]) -> Option<String> {
    let segments =
        extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new(), &HashMap::new());
    let segments = non_artifact_segments(&segments);
    let text = segments_to_text(&segments);
    strings::is_readable_text(&text).then_some(text)
}

pub fn extract_segments_with_fonts(
    stream: &[u8],
    font_cmaps: &HashMap<String, CMap>,
    font_metrics: &HashMap<String, FontMetrics>,
    struct_roles: &HashMap<u32, String>,
) -> Vec<TextSegment> {
    parser::extract_segments_with_fonts(stream, font_cmaps, font_metrics, struct_roles)
}

pub(super) fn repair_shifted_subset_text(text: &str) -> String {
    strings::repair_shifted_subset_words(text)
}

pub(super) fn repair_segment_text(segments: &mut [TextSegment]) {
    for segment in &mut *segments {
        segment.text = repair_shifted_subset_text(&segment.text);
    }
    repair_split_fiscal_quarter_fragments(segments);
}

fn repair_split_fiscal_quarter_fragments(segments: &mut [TextSegment]) {
    for index in 0..segments.len() {
        let Some(quarter) = compact_quarter_fragment(&segments[index].text) else {
            continue;
        };
        let Some(previous_index) = adjacent_previous_year_segment(segments, index) else {
            continue;
        };
        if segments[previous_index].text.starts_with("20") {
            segments[index].text = quarter.to_string();
        }
    }
}

fn compact_quarter_fragment(text: &str) -> Option<&'static str> {
    match text {
        "43" => Some("Q3"),
        "44" => Some("Q4"),
        _ => None,
    }
}

fn adjacent_previous_year_segment(segments: &[TextSegment], index: usize) -> Option<usize> {
    let segment = &segments[index];
    if segment.rotation.abs() >= 0.5 {
        return None;
    }

    segments
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            *candidate_index != index
                && candidate.rotation.abs() < 0.5
                && is_four_digit_year(&candidate.text)
                && same_fiscal_header_line(candidate, segment)
                && candidate.x <= segment.x
        })
        .min_by(|(_, left), (_, right)| {
            let left_gap = segment.x - (left.x + left.width);
            let right_gap = segment.x - (right.x + right.width);
            left_gap
                .abs()
                .partial_cmp(&right_gap.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(candidate_index, _)| candidate_index)
}

fn is_four_digit_year(text: &str) -> bool {
    text.len() == 4
        && text.chars().all(|ch| ch.is_ascii_digit())
        && matches!(text.get(..2), Some("20"))
}

fn same_fiscal_header_line(left: &TextSegment, right: &TextSegment) -> bool {
    let font_size = left.font_size.max(right.font_size).max(1.0);
    let baseline_delta = (left.y - right.y).abs();
    let font_delta = (left.font_size - right.font_size).abs();
    let gap = right.x - (left.x + left.width);

    baseline_delta <= font_size * 0.25
        && font_delta <= font_size * 0.15
        && gap >= -font_size * 0.15
        && gap <= font_size * 0.75
}

pub(super) fn non_artifact_segments(segments: &[TextSegment]) -> Vec<TextSegment> {
    segments
        .iter()
        .filter(|segment| segment.role.as_deref() != Some("Artifact"))
        .cloned()
        .collect()
}

#[cfg(test)]
pub fn segments_to_text(segments: &[TextSegment]) -> String {
    lines::segments_to_text(segments)
}
