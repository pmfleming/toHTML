mod columns;
mod hairlines;
mod leader_lines;
mod license;
mod line_groups;
mod page_numbers;
mod sublabels;
mod widths;

pub(super) use columns::split_segments_at_column_gaps;
pub(super) use hairlines::remove_redundant_header_hairlines;
pub(super) use page_numbers::restore_centered_page_number_markers;
pub(super) use sublabels::split_multicolumn_sublabels;
pub(super) use widths::tighten_overlapping_text_widths;

fn segment_top(segment: &super::text::TextSegment, page_height: f32) -> f32 {
    page_height - segment.y - segment.font_size
}

#[cfg(test)]
mod column_tests;
#[cfg(test)]
mod furniture_tests;
