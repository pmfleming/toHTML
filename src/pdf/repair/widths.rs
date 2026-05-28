use super::super::text;
use super::line_groups::same_visual_text_line;

pub(in crate::pdf) fn tighten_overlapping_text_widths(segments: &mut [text::TextSegment]) {
    let mut indices = (0..segments.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        segments[*right]
            .y
            .total_cmp(&segments[*left].y)
            .then_with(|| segments[*left].x.total_cmp(&segments[*right].x))
    });

    for pair in indices.windows(2) {
        let current = pair[0];
        let next = pair[1];
        if !same_visual_text_line(&segments[current], &segments[next]) {
            continue;
        }
        let available = segments[next].x - segments[current].x;
        if available <= segments[current].font_size * 0.2 {
            continue;
        }
        if segments[current].x + segments[current].width <= segments[next].x {
            continue;
        }
        let minimum = segments[current].width * 0.55;
        segments[current].width = available.max(minimum).min(segments[current].width);
    }
}
