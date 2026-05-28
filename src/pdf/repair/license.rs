use super::super::text;

pub(super) fn remove_license_artifact_runs(segments: &mut Vec<text::TextSegment>, anchors: &[f32]) {
    for segment in segments.iter_mut() {
        let (text, removed_leading_artifact) = strip_license_artifact_runs(&segment.text);
        if text == segment.text {
            continue;
        }
        if removed_leading_artifact {
            if let Some(anchor) =
                nearest_text_anchor_to_right(segment.x, segment.font_size, anchors)
            {
                segment.x = anchor;
            }
        }
        segment.text = text;
        segment.width = text::estimated_text_width(&segment.text, segment.font_size);
    }
    segments.retain(|segment| !segment.text.trim().is_empty());
}

fn nearest_text_anchor_to_right(x: f32, font_size: f32, anchors: &[f32]) -> Option<f32> {
    let max_shift = (font_size.max(6.0) * 3.0).max(18.0);
    anchors
        .iter()
        .copied()
        .filter(|anchor| *anchor > x && *anchor - x <= max_shift)
        .min_by(|left, right| left.total_cmp(right))
}

fn strip_license_artifact_runs(text: &str) -> (String, bool) {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    let mut removed_leading = false;

    while index < chars.len() {
        if is_license_artifact_char(chars[index]) {
            let run_start = index;
            while index < chars.len() && is_license_artifact_char(chars[index]) {
                index += 1;
            }

            let mut next_text = index;
            while next_text < chars.len() && chars[next_text].is_whitespace() {
                next_text += 1;
            }

            if index - run_start >= 12
                && (next_text == chars.len() || chars[next_text].is_alphanumeric())
            {
                if output.trim().is_empty() {
                    output.clear();
                    removed_leading = true;
                } else if next_text == chars.len() {
                    while output.ends_with(char::is_whitespace) {
                        output.pop();
                    }
                } else if next_text < chars.len() && !output.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
                index = next_text;
                continue;
            }

            for ch in &chars[run_start..index] {
                output.push(*ch);
            }
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    (output, removed_leading)
}

fn is_license_artifact_char(ch: char) -> bool {
    matches!(ch, '`' | ',' | '-' | '\'' | '’' | '“' | '”')
}
