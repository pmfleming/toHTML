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
