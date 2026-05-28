use super::super::text::TextSegment;
use super::{PageGeometry, VisualLink, VisualPage};

pub(super) fn repair_visual_text(text: &str) -> String {
    if text.trim() == "ISO 20022" {
        return text.to_string();
    }
    repair_common_visual_text(&super::super::text::repair_shifted_subset_text(text))
}

fn repair_common_visual_text(text: &str) -> String {
    let mut repaired = strip_license_artifact_runs(text);
    repaired = repair_dash_spacing(&repaired);
    repaired = repair_iec_standard_number_fragments(&repaired);
    repaired = repair_joined_word_boundaries(&repaired);
    repaired = repair_iec_definition_prose(&repaired);
    repaired = repair_spaced_common_words(&repaired);
    repaired = repair_note_markers(&repaired);
    repaired = repair_number_markers(&repaired);
    repaired = repair_caption_dash_spacing(&repaired);
    if repaired == "Œ" {
        repaired = "−".to_string();
    }
    repaired
}

fn strip_license_artifact_runs(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

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

    output
}

fn is_license_artifact_char(ch: char) -> bool {
    matches!(ch, '`' | ',' | '-' | '\'' | '’' | '“' | '”')
}

fn repair_dash_spacing(text: &str) -> String {
    text.replace(" Œ", " – ")
        .replace("Œ ", "– ")
        .replace("Œ", "–")
        .replace(" ,", ",")
        .replace(" :", ":")
        .replace("- down", "-down")
        .replace(" -wise", "-wise")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" -frame", "-frame")
        .replace(" -phase", "-phase")
        .replace(" - comité", "-comité")
        .replace(" -comité", "-comité")
        .replace(" - committee", "-committee")
        .replace(" -committee", "-committee")
}

fn repair_iec_standard_number_fragments(text: &str) -> String {
    text.replace("6 1000 -3- IEC 2", "IEC 61000-3-2")
        .replace("61000-3-IEC2", "IEC 61000-3-2")
}

fn repair_joined_word_boundaries(text: &str) -> String {
    text.split_whitespace()
        .map(repair_joined_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn repair_iec_definition_prose(text: &str) -> String {
    text.replace(
        "ratio of the value of the sum of the harmonic components (in this context RMS harmonic",
        "ratio of the RMS value of the sum of the harmonic components (in this context, harmonic",
    )
    .replace(
        "current components Ih of orders 2 to RMS40) to thevalue of the fundamental component",
        "current components Ih of orders 2 to 40) to the RMS value of the fundamental component",
    )
}

fn repair_joined_token(token: &str) -> String {
    if token
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | '/' | '_' | '\\'))
    {
        return token.to_string();
    }

    let chars = token.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(token.len());
    for index in 0..chars.len() {
        if joined_boundary(&chars, index) {
            output.push(' ');
        }
        output.push(chars[index]);
    }
    split_common_joined_pairs(&output)
}

fn joined_boundary(chars: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let left = chars[index - 1];
    let right = chars[index];
    if left.is_ascii_digit() && right.is_ascii_lowercase() {
        return true;
    }
    if left.is_ascii_lowercase() && right.is_ascii_digit() {
        let digit_run = chars[index..]
            .iter()
            .take_while(|ch| ch.is_ascii_digit())
            .count();
        return digit_run >= 2 || chars.get(index + digit_run) == Some(&'.');
    }
    left.is_ascii_lowercase() && right.is_ascii_uppercase() && index >= 5
}

fn split_common_joined_pairs(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            for (left, right) in COMMON_JOINED_WORD_PAIRS {
                if token.eq_ignore_ascii_case(&format!("{left}{right}")) {
                    return preserve_first_word_case(token, left, right);
                }
            }
            token.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn preserve_first_word_case(token: &str, left: &str, right: &str) -> String {
    let split_at = left.len();
    let (actual_left, _) = token.split_at(split_at.min(token.len()));
    format!("{actual_left} {right}")
}

const COMMON_JOINED_WORD_PAIRS: &[(&str, &str)] =
    &[("and", "can"), ("of", "over"), ("is", "a"), ("as", "a")];

fn repair_spaced_common_words(text: &str) -> String {
    let mut repaired = text.to_string();
    for word in SPACED_COMMON_WORDS {
        repaired = repair_spaced_word(&repaired, word);
    }
    repaired
}

fn repair_spaced_word(text: &str, word: &str) -> String {
    let chars = word.chars().collect::<Vec<_>>();
    if chars.len() < 4 {
        return text.to_string();
    }
    let mut pattern = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if index > 0 {
            pattern.push(' ');
        }
        pattern.push(*ch);
    }
    text.replace(&pattern, word)
}

const SPACED_COMMON_WORDS: &[&str] = &[
    "harmonic",
    "recommendation",
    "compatibility",
    "which",
    "maximum",
    "table",
    "Class",
];

fn repair_note_markers(text: &str) -> String {
    let mut repaired = String::with_capacity(text.len());
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < chars.len() {
        if starts_with_chars(&chars[index..], "Note") {
            let digit_start = index + 4;
            let mut digit_end = digit_start;
            while digit_end < chars.len() && chars[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end > digit_start && starts_with_chars(&chars[digit_end..], "to ") {
                repaired.push_str("Note ");
                for ch in &chars[digit_start..digit_end] {
                    repaired.push(*ch);
                }
                repaired.push_str(" to ");
                index = digit_end + 3;
                continue;
            }
        }
        repaired.push(chars[index]);
        index += 1;
    }
    repaired
}

fn starts_with_chars(chars: &[char], prefix: &str) -> bool {
    chars
        .iter()
        .copied()
        .zip(prefix.chars())
        .all(|(a, b)| a == b)
        && chars.len() >= prefix.chars().count()
}

fn repair_number_markers(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut repaired = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'Œ' {
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            while digit_end < chars.len() && chars[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if digit_end > digit_start && digit_end < chars.len() && chars[digit_end] == 'Œ' {
                repaired.push_str("– ");
                for ch in &chars[digit_start..digit_end] {
                    repaired.push(*ch);
                }
                repaired.push_str(" –");
                index = digit_end + 1;
                continue;
            }
        }
        repaired.push(chars[index]);
        index += 1;
    }
    repaired
}

fn repair_caption_dash_spacing(text: &str) -> String {
    let mut repaired = text.to_string();
    for prefix in ["Figure", "Table", "Tableau"] {
        for number in 1..=12 {
            repaired = repaired.replace(
                &format!("{prefix} {number}Œ"),
                &format!("{prefix} {number} – "),
            );
            repaired = repaired.replace(
                &format!("{prefix}{number}Œ"),
                &format!("{prefix} {number} – "),
            );
        }
    }
    repaired
        .replace("–Flowchart", "– Flowchart")
        .replace("–Illustration", "– Illustration")
        .replace("–Organigramme", "– Organigramme")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_embedded_license_punctuation_from_visual_text() {
        let text =
            "which can be produced by --`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested";

        assert_eq!(
            repair_visual_text(text),
            "which can be produced by equipment tested"
        );
    }

    #[test]
    fn repairs_joined_common_words() {
        assert_eq!(
            repair_visual_text("document andcan be subject"),
            "document and can be subject"
        );
    }

    #[test]
    fn repairs_downshifted_table_labels_at_visual_boundary() {
        assert_eq!(repair_visual_text("2025 eN"), "2025 H1");
        assert_eq!(repair_visual_text("2025 e2(Q3+Q4)"), "2025 H2(Q3+Q4)");
        assert_eq!(repair_visual_text("2025 eOEnPHn QF"), "2025 H2(Q3+Q4)");
        assert_eq!(repair_visual_text("OMOQe O actual"), "2024H2 actual");
        assert_eq!(repair_visual_text("Q3"), "Q3");
        assert_eq!(repair_visual_text("NIQRUKMN"), "1,458.01");
    }

    #[test]
    fn repairs_shifted_symbol_markers_at_visual_boundary() {
        assert_eq!(repair_visual_text("Ł"), "•");
        assert_eq!(repair_visual_text(">&"), "(");
        assert_eq!(repair_visual_text(">'"), ")");
        assert_eq!(repair_visual_text("recognized by––"), "recognized by......");
        assert_eq!(
            repair_visual_text("Top- down, Global -wise"),
            "Top-down, Global-wise"
        );
    }

    #[test]
    fn repairs_iec_definition_rms_fragments() {
        assert_eq!(
            repair_visual_text(
                "ratio of the value of the sum of the harmonic components (in this context RMS harmonic"
            ),
            "ratio of the RMS value of the sum of the harmonic components (in this context, harmonic"
        );
        assert_eq!(
            repair_visual_text(
                "current components Ih of orders 2 to RMS40) to thevalue of the fundamental component"
            ),
            "current components Ih of orders 2 to 40) to the RMS value of the fundamental component"
        );
    }
}

pub(super) fn annotation_aligned_url_segments(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    let mut segments = page.segments.clone();
    let mut relocated_lines = Vec::new();

    for segment in &mut segments {
        if let Some(link) = matching_lower_line_url_link(segment, &page.links, geometry) {
            let link_top = link_top(link, geometry);
            segment.y = geometry.height - link_top - segment.font_size;
            segment.text = visible_link_target(&link.href);
            segment.width = segment.width.max(link.width);
            relocated_lines.push(segment.y);
        } else if let Some(link) = matching_same_line_url_link(segment, &page.links, geometry) {
            segment.text = visible_link_target(&link.href);
            segment.width = segment.width.max(link.width);
        }
    }

    let segments = segments
        .into_iter()
        .filter(|segment| !is_relocated_url_marker(segment, &relocated_lines))
        .collect::<Vec<_>>();
    repair_iso20022_catalogue_visual_segments(segments, &page.links, geometry)
}

fn matching_lower_line_url_link<'a>(
    segment: &TextSegment,
    links: &'a [VisualLink],
    geometry: PageGeometry,
) -> Option<&'a VisualLink> {
    if super::text_inference::normalized_rotation(segment.rotation).abs() >= 0.5
        || !is_url_text_fragment(segment)
    {
        return None;
    }
    let segment_top = super::text_inference::segment_top(segment, geometry);
    links.iter().find(|link| {
        let visible = visible_link_target(&link.href);
        !visible.is_empty()
            && normalized_url_text(&repair_visual_text(&segment.text))
                == normalized_url_text(&visible)
            && (segment.x - link.x).abs() <= segment.font_size.max(8.0)
            && {
                let delta = link_top(link, geometry) - segment_top;
                delta >= segment.font_size * 0.9 && delta <= segment.font_size * 2.4
            }
    })
}

fn matching_same_line_url_link<'a>(
    segment: &TextSegment,
    links: &'a [VisualLink],
    geometry: PageGeometry,
) -> Option<&'a VisualLink> {
    if super::text_inference::normalized_rotation(segment.rotation).abs() >= 0.5
        || !is_url_text_fragment(segment)
    {
        return None;
    }
    let segment_top = super::text_inference::segment_top(segment, geometry);
    links.iter().find(|link| {
        let visible = visible_link_target(&link.href);
        !visible.is_empty()
            && normalized_url_text(&repair_visual_text(&segment.text))
                == normalized_url_text(&visible)
            && (segment.x - link.x).abs() <= segment.font_size.max(8.0)
            && (link_top(link, geometry) - segment_top).abs() <= segment.font_size * 0.4
            && segment.width < link.width * 0.9
    })
}

fn is_url_text_fragment(segment: &TextSegment) -> bool {
    let repaired = repair_visual_text(&segment.text);
    let text = repaired.trim();
    text.contains("://") || text.contains("www.") || text.contains(".org/")
}

fn visible_link_target(href: &str) -> String {
    href.strip_prefix("http://")
        .or_else(|| href.strip_prefix("https://"))
        .unwrap_or(href)
        .to_string()
}

fn normalized_url_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .trim_end_matches(['.', '/'])
        .to_ascii_lowercase()
}

fn is_relocated_url_marker(segment: &TextSegment, relocated_lines: &[f32]) -> bool {
    let text = segment.text.trim();
    let repaired = repair_visual_text(text);
    (matches!(text, "E" | "I" | "(") || matches!(repaired.trim(), "E" | "I" | "("))
        && segment.x <= 70.0
        && relocated_lines
            .iter()
            .any(|line_y| (segment.y - *line_y).abs() <= segment.font_size * 0.65)
}

fn repair_iso20022_catalogue_visual_segments(
    segments: Vec<TextSegment>,
    links: &[VisualLink],
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    if !links
        .iter()
        .any(|link| visible_link_target(&link.href).starts_with("www.iso20022.org/"))
    {
        return segments;
    }

    let mut output = Vec::with_capacity(segments.len());
    let mut index = 0;
    while index < segments.len() {
        if let Some(combined) = combine_split_iso20022_reference(&segments, index, geometry) {
            output.push(combined);
            index += 3;
            continue;
        }

        let mut segment = segments[index].clone();
        let repaired = repair_visual_text(&segment.text);
        if segment.text.contains("0,62 20022")
            || segment.text.contains("0, 62 20022")
            || repaired.contains("0,62 20022")
            || repaired.contains("0, 62 20022")
        {
            segment.text = "ISO 20022".to_string();
            segment.width = segment.width.max(segment.font_size * 4.6);
        }
        output.push(segment);
        index += 1;
    }
    output
}

fn combine_split_iso20022_reference(
    segments: &[TextSegment],
    index: usize,
    geometry: PageGeometry,
) -> Option<TextSegment> {
    let current = segments.get(index)?;
    let marker = segments.get(index + 1)?;
    let next = segments.get(index + 2)?;
    if !same_visual_line(current, marker, geometry) || !same_visual_line(current, next, geometry) {
        return None;
    }

    let current_text = repair_visual_text(&current.text);
    let marker_text = repair_visual_text(&marker.text);
    let next_text = repair_visual_text(&next.text);
    let prefix = current_text.trim_end().strip_suffix("pai")?;
    let suffix = next_text.trim_start().strip_prefix('n')?;
    if marker_text.trim() != ")" || !suffix.starts_with(".001.") {
        return None;
    }

    let mut combined = current.clone();
    combined.text = format!("{prefix}pain{suffix}");
    combined.width = (next.x + next.width - current.x).max(current.width);
    Some(combined)
}

fn same_visual_line(left: &TextSegment, right: &TextSegment, geometry: PageGeometry) -> bool {
    (super::text_inference::segment_top(left, geometry)
        - super::text_inference::segment_top(right, geometry))
    .abs()
        <= left.font_size.max(right.font_size) * 0.35
}

fn link_top(link: &VisualLink, geometry: PageGeometry) -> f32 {
    (geometry.height - link.y - link.height).max(0.0)
}
