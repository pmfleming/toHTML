mod lines;
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

#[cfg(test)]
pub fn extract_text(stream: &[u8]) -> Option<String> {
    let segments = extract_segments_with_fonts(stream, &HashMap::new(), &HashMap::new());
    let text = segments_to_text(&segments);
    strings::is_readable_text(&text).then_some(text)
}

pub fn extract_segments_with_fonts(
    stream: &[u8],
    font_cmaps: &HashMap<String, CMap>,
    font_metrics: &HashMap<String, FontMetrics>,
) -> Vec<TextSegment> {
    parser::extract_segments_with_fonts(stream, font_cmaps, font_metrics)
}

#[cfg(test)]
pub fn segments_to_text(segments: &[TextSegment]) -> String {
    lines::segments_to_text(segments)
}
