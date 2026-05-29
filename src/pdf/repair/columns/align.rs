use super::{
    has_domain_like_text, has_nearby_column_band, is_standalone_list_marker,
    looks_like_column_start, plausible_text_column_anchor,
};
use crate::pdf::repair::line_groups::{
    joined_segment_text, same_visual_text_line, text_line_groups,
};
use crate::pdf::text;

pub(super) fn align_right_domain_headings_to_previous_left_headings(
    anchors: &[f32],
    segments: &mut [text::TextSegment],
) {
    let snapshot = segments.to_vec();
    for anchor in anchors.iter().copied().skip(1) {
        for line in text_line_groups(&snapshot) {
            let Some(first) = line.first().copied() else {
                continue;
            };
            let font_size = snapshot[first].font_size.max(8.0);
            let right_indices = line
                .iter()
                .copied()
                .filter(|index| snapshot[*index].x + font_size * 1.5 >= anchor)
                .collect::<Vec<_>>();
            if right_indices.is_empty() {
                continue;
            }
            let right_text = joined_segment_text(&snapshot, &right_indices);
            if !has_domain_like_text(&right_text) {
                continue;
            }

            let left_indices = line
                .iter()
                .copied()
                .filter(|index| snapshot[*index].x + font_size * 1.5 < anchor)
                .collect::<Vec<_>>();
            let left_text = joined_segment_text(&snapshot, &left_indices);
            if left_text.trim().is_empty() || has_domain_like_text(&left_text) {
                continue;
            }

            let Some(previous_y) =
                previous_left_domain_heading_line_y(&snapshot, first, anchor, &line)
            else {
                continue;
            };
            for index in right_indices {
                segments[index].y = previous_y;
            }
        }
    }
}

fn previous_left_domain_heading_line_y(
    segments: &[text::TextSegment],
    line_first: usize,
    anchor: f32,
    current_line: &[usize],
) -> Option<f32> {
    let line = &segments[line_first];
    let font_size = line.font_size.max(8.0);
    text_line_groups(segments)
        .into_iter()
        .filter(|candidate| {
            candidate.first().is_some_and(|first| {
                segments[*first].y > line.y && segments[*first].y - line.y <= font_size * 2.0
            }) && !same_index_set(candidate, current_line)
        })
        .filter_map(|candidate| {
            let left = candidate
                .iter()
                .copied()
                .filter(|index| segments[*index].x + font_size * 1.5 < anchor)
                .collect::<Vec<_>>();
            let right = candidate
                .iter()
                .copied()
                .filter(|index| segments[*index].x + font_size * 1.5 >= anchor)
                .collect::<Vec<_>>();
            let left_text = joined_segment_text(segments, &left);
            let right_text = joined_segment_text(segments, &right);
            (has_domain_like_text(&left_text) && !has_domain_like_text(&right_text))
                .then_some(segments[*candidate.first()?].y)
        })
        .min_by(|left, right| (left - line.y).abs().total_cmp(&(right - line.y).abs()))
}

fn same_index_set(left: &[usize], right: &[usize]) -> bool {
    left.len() == right.len() && left.iter().all(|index| right.contains(index))
}

pub(super) fn align_segments_to_column_anchors(
    page_width: f32,
    segments: &mut [text::TextSegment],
    anchors: &[f32],
) {
    let mut indices = (0..segments.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        segments[*right]
            .y
            .total_cmp(&segments[*left].y)
            .then_with(|| segments[*left].x.total_cmp(&segments[*right].x))
    });

    let mut line = Vec::new();
    for index in indices {
        if line
            .first()
            .is_none_or(|first| same_visual_text_line(&segments[*first], &segments[index]))
        {
            line.push(index);
        } else {
            align_line_segments(page_width, segments, anchors, &line);
            line.clear();
            line.push(index);
        }
    }
    align_line_segments(page_width, segments, anchors, &line);
}

fn align_line_segments(
    page_width: f32,
    segments: &mut [text::TextSegment],
    anchors: &[f32],
    line: &[usize],
) {
    if line.len() < 2 {
        return;
    }
    if line
        .iter()
        .skip(1)
        .any(|index| is_standalone_list_marker(&segments[*index].text))
    {
        return;
    }
    let first = line[0];
    let line_right = line
        .iter()
        .map(|index| segments[*index].x + segments[*index].width.max(0.0))
        .fold(segments[first].x, f32::max);
    let Some(right_anchor) = anchors.iter().copied().find(|anchor| {
        plausible_text_column_anchor(page_width, *anchor)
            && *anchor > segments[first].x + page_width * 0.22
            && *anchor < line_right
    }) else {
        return;
    };
    if !has_nearby_column_band(segments, &segments[first], right_anchor) {
        return;
    }

    let font_size = segments[first].font_size.max(6.0);
    for index in line.iter().skip(1) {
        if segments[*index].x >= right_anchor - font_size * 1.5 {
            continue;
        }
        if segments[*index].x <= segments[first].x + font_size * 4.0 {
            continue;
        }
        if !looks_like_column_start(&segments[*index].text) {
            continue;
        }
        if line_anchor_is_occupied(segments, line, right_anchor, *index) {
            continue;
        }
        segments[*index].x = right_anchor;
        break;
    }
}

fn line_anchor_is_occupied(
    segments: &[text::TextSegment],
    line: &[usize],
    anchor: f32,
    moving_index: usize,
) -> bool {
    let tolerance = segments[moving_index].font_size.max(6.0) * 1.5;
    line.iter()
        .copied()
        .any(|index| index != moving_index && (segments[index].x - anchor).abs() <= tolerance)
}
