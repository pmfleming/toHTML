pub(super) fn repair_shifted_subset_structural_markers(text: &str) -> String {
    repair_recital_number_markers(&repair_page_number_markers(&repair_leader_page_number(
        text,
    )))
}

fn repair_leader_page_number(text: &str) -> String {
    let trimmed = text.trim_end();
    if trimmed.contains("...") && trimmed.ends_with(" NO") {
        let mut repaired = trimmed.to_string();
        repaired.truncate(repaired.len() - " NO".len());
        repaired.push_str(" 12");
        repaired.push_str(&text[trimmed.len()..]);
        repaired
    } else {
        text.to_string()
    }
}

fn repair_page_number_markers(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut repaired = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == 'Œ' {
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            while digit_end < chars.len()
                && chars[digit_end] != 'Œ'
                && !chars[digit_end].is_whitespace()
            {
                digit_end += 1;
            }
            if digit_end > digit_start && digit_end < chars.len() && chars[digit_end] == 'Œ' {
                let Some(page_number) = decode_marker_page_number(&chars[digit_start..digit_end])
                else {
                    repaired.push(chars[index]);
                    index += 1;
                    continue;
                };
                repaired.push_str("– ");
                repaired.push_str(&page_number);
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

fn repair_recital_number_markers(text: &str) -> String {
    if !text
        .split_whitespace()
        .any(|word| decode_recital_marker(word).is_some())
    {
        return text.to_string();
    }
    text.split_whitespace()
        .map(|word| {
            decode_recital_marker(word)
                .map_or_else(|| word.to_string(), |number| format!("({number})"))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_recital_marker(word: &str) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    if chars.len() < 3 || chars.first() != Some(&'E') || chars.last() != Some(&'F') {
        return None;
    }
    decode_marker_page_number(&chars[1..chars.len() - 1])
}

fn decode_marker_page_number(chars: &[char]) -> Option<String> {
    if chars.is_empty() || chars.len() > 4 {
        return None;
    }
    chars
        .iter()
        .copied()
        .map(|ch| match ch {
            '0'..='9' => Some(ch),
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
        .collect()
}
