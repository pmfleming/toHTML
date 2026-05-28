use super::super::text;
use super::line_groups::{joined_segment_text, text_line_groups};

pub(super) fn split_embedded_leader_page_numbers(
    page_width: f32,
    anchors: &[f32],
    segments: &mut Vec<text::TextSegment>,
) {
    let Some(page_number_anchor) = right_page_number_anchor(page_width, anchors) else {
        return;
    };

    let mut additions = Vec::new();
    for segment in segments.iter_mut() {
        if segment.x >= page_number_anchor - segment.font_size * 4.0 {
            continue;
        }
        let Some((label, leaders, page_number)) = split_leader_page_number_text(&segment.text)
        else {
            continue;
        };
        let label_width = text::estimated_text_width(&label, segment.font_size);
        let leader_x = leader_x(
            segment.x,
            label_width,
            segment.font_size,
            page_number_anchor,
        );

        let mut leader = segment.clone();
        leader.text = leaders;
        leader.x = leader_x;
        leader.width = (page_number_anchor - leader_x - segment.font_size)
            .max(segment.font_size)
            .min(segment.width.max(segment.font_size));

        let mut page = segment.clone();
        page.text = page_number;
        page.x = page_number_anchor;
        page.width = text::estimated_text_width(&page.text, page.font_size);

        segment.text = label;
        segment.width = label_width.min((leader_x - segment.x).max(segment.font_size));

        additions.push(leader);
        additions.push(page);
    }

    segments.extend(additions);
}

pub(super) fn split_joined_leader_page_number_lines(
    page_width: f32,
    anchors: &[f32],
    segments: &mut Vec<text::TextSegment>,
) {
    let Some(page_number_anchor) = right_page_number_anchor(page_width, anchors) else {
        return;
    };
    let snapshot = segments.clone();
    let mut remove = vec![false; segments.len()];
    let mut additions = Vec::new();

    for line in text_line_groups(&snapshot) {
        if line.len() < 2 {
            continue;
        }
        let font_size = line
            .iter()
            .map(|index| snapshot[*index].font_size)
            .fold(0.0_f32, f32::max)
            .max(8.0);
        if line.iter().any(|index| {
            (snapshot[*index].x - page_number_anchor).abs() <= font_size * 2.0
                && decode_leader_page_number_token(&snapshot[*index].text).is_some()
        }) {
            continue;
        }

        let joined = joined_segment_text(&snapshot, &line);
        let Some((label, leaders, page_number)) = split_leader_page_number_text(&joined) else {
            continue;
        };
        let Some(first) = line.first().copied() else {
            continue;
        };
        let label_width = text::estimated_text_width(&label, snapshot[first].font_size);
        let leader_x = leader_x(
            snapshot[first].x,
            label_width,
            snapshot[first].font_size,
            page_number_anchor,
        );

        segments[first].text = label;
        segments[first].width = label_width.min((leader_x - snapshot[first].x).max(font_size));
        for index in line.iter().copied().skip(1) {
            remove[index] = true;
        }

        let mut leader = snapshot[first].clone();
        leader.text = leaders;
        leader.x = leader_x;
        leader.width = (page_number_anchor - leader_x - font_size)
            .max(font_size)
            .min(page_width - leader_x);

        let mut page = snapshot[first].clone();
        page.text = page_number;
        page.x = page_number_anchor;
        page.width = text::estimated_text_width(&page.text, page.font_size);

        additions.push(leader);
        additions.push(page);
    }

    for index in (0..segments.len()).rev() {
        if remove[index] {
            segments.remove(index);
        }
    }
    segments.extend(additions);
}

fn right_page_number_anchor(page_width: f32, anchors: &[f32]) -> Option<f32> {
    anchors
        .iter()
        .copied()
        .find(|anchor| *anchor > page_width * 0.75)
}

fn leader_x(segment_x: f32, label_width: f32, font_size: f32, page_number_anchor: f32) -> f32 {
    (segment_x + label_width + font_size * 0.5)
        .min(page_number_anchor - font_size * 6.0)
        .max(segment_x + font_size)
}

fn split_leader_page_number_text(text: &str) -> Option<(String, String, String)> {
    if !text.contains("...") {
        return None;
    }
    let trimmed = text.trim_end();
    let page_token = trimmed.split_whitespace().last()?;
    let page_number = decode_leader_page_number_token(page_token)?;
    let page_start = trimmed.rfind(page_token)?;
    let before_page = trimmed[..page_start].trim_end();
    let leader_start = before_page
        .char_indices()
        .rfind(|(_, ch)| *ch != '.' && !ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())?;
    let label = before_page[..leader_start].trim_end();
    let leaders = before_page[leader_start..].trim_start();
    if label.is_empty() || !leaders.chars().any(|ch| ch == '.') {
        return None;
    }
    Some((label.to_string(), leaders.to_string(), page_number))
}

fn decode_leader_page_number_token(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(trimmed.to_string());
    }
    let decoded = trimmed
        .chars()
        .map(|ch| match ch {
            'M' => Some('0'),
            'N' => Some('1'),
            'O' => Some('2'),
            'P' => Some('3'),
            'Q' => Some('4'),
            'R' => Some('5'),
            'S' => Some('6'),
            'T' => Some('7'),
            'U' => Some('8'),
            'V' => Some('9'),
            _ => None,
        })
        .collect::<Option<String>>()?;
    (!decoded.is_empty()).then_some(decoded)
}
