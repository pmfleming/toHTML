use super::super::text;
use super::leader_lines::{
    split_embedded_leader_page_numbers, split_joined_leader_page_number_lines,
};
use super::license::remove_license_artifact_runs;
use super::line_groups::{joined_segment_text, same_visual_text_line, text_line_groups};

pub(in crate::pdf) fn split_segments_at_column_gaps(
    page_width: f32,
    segments: &mut Vec<text::TextSegment>,
) {
    if segments.len() < 8 || page_width <= 0.0 {
        return;
    }

    let anchors = repeated_x_anchors(segments);
    remove_license_artifact_runs(segments, &anchors);
    let anchors = repeated_x_anchors(segments);
    if anchors.len() < 2 {
        return;
    }

    align_segments_to_column_anchors(page_width, segments, &anchors);
    split_embedded_leader_page_numbers(page_width, &anchors, segments);
    split_joined_leader_page_number_lines(page_width, &anchors, segments);

    let original = segments.clone();
    let mut additions = Vec::new();
    for segment in segments.iter_mut() {
        let Some(right_anchor) = column_anchor_inside_segment(page_width, &anchors, segment)
            .or_else(|| {
                inferred_column_anchor_for_segment(page_width, &anchors, &original, segment)
            })
        else {
            continue;
        };
        if !has_nearby_column_band(&original, segment, right_anchor) {
            continue;
        }
        let Some((left_text, right_text)) = split_text_for_column_anchor(segment, right_anchor)
        else {
            continue;
        };

        let mut right = segment.clone();
        right.text = split_compact_acronym_title(&right_text);
        right.x = right_anchor;
        if let Some(previous_y) =
            previous_left_heading_y(&original, segment, &left_text, &right.text)
        {
            right.y = previous_y;
        }
        right.width = text::estimated_text_width(&right.text, right.font_size);

        segment.text = split_compact_acronym_title(&left_text);
        segment.width = text::estimated_text_width(&segment.text, segment.font_size)
            .min((right_anchor - segment.x).max(segment.font_size));

        additions.push(right);
    }

    segments.extend(additions);
    align_right_domain_headings_to_previous_left_headings(&anchors, segments);
}

fn align_right_domain_headings_to_previous_left_headings(
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

fn align_segments_to_column_anchors(
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

fn repeated_x_anchors(segments: &[text::TextSegment]) -> Vec<f32> {
    let mut buckets: Vec<(f32, usize)> = Vec::new();
    for segment in segments {
        if segment.text.trim().is_empty() || segment.width <= 0.0 {
            continue;
        }
        if is_standalone_list_marker(&segment.text) {
            continue;
        }
        match buckets
            .iter_mut()
            .find(|(x, _)| (*x - segment.x).abs() <= 3.0)
        {
            Some((x, count)) => {
                *x = ((*x * *count as f32) + segment.x) / (*count as f32 + 1.0);
                *count += 1;
            }
            None => buckets.push((segment.x, 1)),
        }
    }
    buckets.retain(|(_, count)| *count >= 3);
    buckets.sort_by(|left, right| left.0.total_cmp(&right.0));
    buckets.into_iter().map(|(x, _)| x).collect()
}

fn column_anchor_inside_segment(
    page_width: f32,
    anchors: &[f32],
    segment: &text::TextSegment,
) -> Option<f32> {
    let min_gap = page_width * 0.22;
    anchors
        .iter()
        .copied()
        .filter(|anchor| {
            plausible_text_column_anchor(page_width, *anchor)
                && *anchor > segment.x + min_gap
                && *anchor < segment.x + segment.width - segment.font_size * 2.0
        })
        .min_by(|left, right| left.total_cmp(right))
}

fn inferred_column_anchor_for_segment(
    page_width: f32,
    anchors: &[f32],
    segments: &[text::TextSegment],
    segment: &text::TextSegment,
) -> Option<f32> {
    if segment.x > page_width * 0.4 || segment.text.split_whitespace().count() < 4 {
        return None;
    }
    anchors
        .iter()
        .copied()
        .filter(|anchor| {
            plausible_text_column_anchor(page_width, *anchor)
                && *anchor > segment.x + page_width * 0.22
        })
        .filter(|anchor| has_nearby_column_band(segments, segment, *anchor))
        .find(|anchor| split_text_for_column_anchor(segment, *anchor).is_some())
}

fn plausible_text_column_anchor(page_width: f32, anchor: f32) -> bool {
    anchor >= page_width * 0.35 && anchor <= page_width * 0.68
}

fn has_nearby_column_band(
    segments: &[text::TextSegment],
    segment: &text::TextSegment,
    right_anchor: f32,
) -> bool {
    let x_tolerance = segment.font_size.max(6.0);
    let max_vertical_gap = segment.font_size.max(6.0) * 16.0;
    let mut has_above = false;
    let mut has_below = false;
    for other in segments {
        if (other.x - right_anchor).abs() > x_tolerance {
            continue;
        }
        let delta = other.y - segment.y;
        if delta > segment.font_size * 0.5 && delta <= max_vertical_gap {
            has_above = true;
        } else if delta < -segment.font_size * 0.5 && -delta <= max_vertical_gap {
            has_below = true;
        }
    }
    has_above && has_below
}

fn split_text_for_column_anchor(
    segment: &text::TextSegment,
    right_anchor: f32,
) -> Option<(String, String)> {
    let words = segment.text.split_whitespace().collect::<Vec<_>>();
    if words.len() < 2 {
        return None;
    }

    let target_width = (right_anchor - segment.x - segment.font_size * 2.0).max(segment.font_size);
    let mut best = None;
    for split_at in 1..words.len() {
        let left = words[..split_at].join(" ");
        let right = words[split_at..].join(" ");
        if !looks_like_column_start(&right) {
            continue;
        }
        let error = (text::estimated_text_width(&left, segment.font_size) - target_width).abs();
        if error <= segment.font_size * 12.0
            && best
                .as_ref()
                .is_none_or(|(_, _, best_error)| error < *best_error)
        {
            best = Some((left, right, error));
        }
    }

    best.map(|(left, right, _)| (left, right))
}

fn previous_left_heading_y(
    segments: &[text::TextSegment],
    segment: &text::TextSegment,
    left_text: &str,
    right_text: &str,
) -> Option<f32> {
    if has_domain_like_text(left_text) || !has_domain_like_text(right_text) {
        return None;
    }

    let line_height = segment.font_size.max(8.0);
    segments
        .iter()
        .filter(|other| {
            (other.x - segment.x).abs() <= line_height
                && other.y > segment.y
                && other.y - segment.y <= line_height * 2.0
                && has_domain_like_text(&other.text)
        })
        .min_by(|left, right| {
            (left.y - segment.y)
                .abs()
                .total_cmp(&(right.y - segment.y).abs())
        })
        .map(|other| other.y)
}

fn looks_like_column_start(text: &str) -> bool {
    let trimmed = text
        .trim_start_matches(|ch: char| matches!(ch, '-' | '–' | '—' | ':' | ';' | ',' | '(' | '['));
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    first.is_ascii_digit()
        || first.is_ascii_uppercase()
        || trimmed.starts_with("www.")
        || trimmed.starts_with("http")
        || trimmed.contains("://")
}

fn is_standalone_list_marker(text: &str) -> bool {
    matches!(text.trim(), "□" | "☐" | "☑" | "☒" | "✓" | "✔" | "✕" | "✗")
}

fn has_domain_like_text(text: &str) -> bool {
    text.contains("www.")
        || text.contains("://")
        || text.contains(".com")
        || text.contains(".org")
        || text.contains(".net")
        || text.contains(".ch")
        || text.contains(".io")
}

fn split_compact_acronym_title(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    for index in 2..chars.len().min(6) {
        if chars[..index].iter().all(|ch| ch.is_ascii_uppercase())
            && chars.get(index).is_some_and(|ch| ch.is_ascii_uppercase())
            && chars
                .get(index + 1)
                .is_some_and(|ch| ch.is_ascii_lowercase())
        {
            let mut repaired = String::with_capacity(text.len() + 1);
            for ch in &chars[..index] {
                repaired.push(*ch);
            }
            repaired.push(' ');
            for ch in &chars[index..] {
                repaired.push(*ch);
            }
            return repaired;
        }
    }
    text.to_string()
}
