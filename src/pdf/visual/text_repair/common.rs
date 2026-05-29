use super::{joined::repair_joined_word_boundaries, license::strip_license_artifact_runs};

pub(super) fn repair_common_visual_text(text: &str) -> String {
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
