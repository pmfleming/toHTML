use super::super::text;

pub(super) fn same_visual_text_line(left: &text::TextSegment, right: &text::TextSegment) -> bool {
    (left.y - right.y).abs() <= left.font_size.max(right.font_size) * 0.35
        && (left.rotation - right.rotation).abs() < 0.5
}

pub(super) fn text_line_groups(segments: &[text::TextSegment]) -> Vec<Vec<usize>> {
    let mut indices = (0..segments.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        segments[*right]
            .y
            .total_cmp(&segments[*left].y)
            .then_with(|| segments[*left].x.total_cmp(&segments[*right].x))
    });

    let mut groups: Vec<Vec<usize>> = Vec::new();
    for index in indices {
        if groups.last().is_some_and(|group| {
            group
                .first()
                .is_some_and(|first| same_visual_text_line(&segments[*first], &segments[index]))
        }) {
            groups.last_mut().expect("group exists").push(index);
        } else {
            groups.push(vec![index]);
        }
    }
    groups
}

pub(super) fn joined_segment_text(segments: &[text::TextSegment], indices: &[usize]) -> String {
    let mut indices = indices.to_vec();
    indices.sort_by(|left, right| segments[*left].x.total_cmp(&segments[*right].x));
    indices
        .into_iter()
        .map(|index| segments[index].text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}
