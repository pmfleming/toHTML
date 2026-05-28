use super::super::text;
use super::segment_top;

pub(in crate::pdf) fn split_multicolumn_sublabels(
    page_height: f32,
    segments: &mut Vec<text::TextSegment>,
) {
    let mut additions = Vec::new();
    for index in 0..segments.len() {
        let words = segments[index]
            .text
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if words.len() != 2 {
            continue;
        }

        let label_top = segment_top(&segments[index], page_height);
        let Some((left_x, left_width, right_x, right_width)) =
            two_column_sublabel_positions(page_height, segments, &segments[index], label_top)
        else {
            continue;
        };

        segments[index].text = words[0].clone();
        segments[index].x = left_x;
        segments[index].width = left_width;

        let mut right = segments[index].clone();
        right.text = words[1].clone();
        right.x = right_x;
        right.width = right_width;
        additions.push(right);
    }
    segments.extend(additions);
}

fn two_column_sublabel_positions(
    page_height: f32,
    segments: &[text::TextSegment],
    label: &text::TextSegment,
    label_top: f32,
) -> Option<(f32, f32, f32, f32)> {
    let words = label.text.split_whitespace().collect::<Vec<_>>();
    if words.len() != 2 {
        return None;
    }

    let mut anchors = segments
        .iter()
        .filter(|segment| is_header_anchor_above(page_height, segment, label, label_top))
        .collect::<Vec<_>>();
    anchors.sort_by(|left, right| left.x.total_cmp(&right.x));
    anchors.dedup_by(|left, right| (left.x - right.x).abs() <= 2.0);

    if anchors.len() != 2 {
        return None;
    }

    Some((
        centered_label_x(anchors[0], words[0], label.font_size),
        text::estimated_text_width(words[0], label.font_size),
        centered_label_x(anchors[1], words[1], label.font_size),
        text::estimated_text_width(words[1], label.font_size),
    ))
}

fn is_header_anchor_above(
    page_height: f32,
    segment: &text::TextSegment,
    label: &text::TextSegment,
    label_top: f32,
) -> bool {
    let delta = label_top - segment_top(segment, page_height);
    if !(8.0..=25.0).contains(&delta) {
        return false;
    }

    let tolerance = label.font_size.max(segment.font_size) * 1.5;
    let center = segment.x + segment.width.max(0.0) / 2.0;
    center >= label.x - tolerance && center <= label.x + label.width + tolerance
}

fn centered_label_x(segment: &text::TextSegment, text: &str, font_size: f32) -> f32 {
    let width = text::estimated_text_width(text, font_size);
    segment.x + segment.width.max(0.0) / 2.0 - width / 2.0
}
