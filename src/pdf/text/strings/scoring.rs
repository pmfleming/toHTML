pub(super) fn text_score(text: &str) -> i32 {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect();
    let common = words.iter().filter(|word| common_word(word)).count() as i32;
    let lower = text.to_ascii_lowercase();
    let embedded_common = [
        "agreement",
        "confidential",
        "equipment",
        "information",
        "party",
        "shall",
        "the",
        "this",
    ]
    .iter()
    .filter(|word| lower.contains(**word))
    .count() as i32;
    let vowel_words = words
        .iter()
        .filter(|word| {
            word.chars()
                .any(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
        })
        .count() as i32;
    let suspicious = words
        .iter()
        .filter(|word| {
            word.len() >= 8
                && !word
                    .chars()
                    .any(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'))
        })
        .count() as i32;
    let weird = text
        .chars()
        .filter(|ch| matches!(ch, '}' | ']' | '^' | '~' | '\u{fffd}'))
        .count() as i32;

    common * 12 + embedded_common * 6 + vowel_words * 3 - suspicious * 8 - weird * 10
}

pub(super) fn shifted_beats_decoded(shifted: &str, decoded: &str, required_gain: i32) -> bool {
    let shifted_score = text_score(shifted);
    let decoded_score = text_score(decoded);
    if required_gain == 0 {
        shifted != decoded && shifted_score > decoded_score
    } else {
        shifted_score > decoded_score + required_gain
    }
}

fn common_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "be"
            | "by"
            | "for"
            | "from"
            | "in"
            | "is"
            | "it"
            | "not"
            | "of"
            | "or"
            | "shall"
            | "the"
            | "this"
            | "to"
            | "with"
            | "agreement"
            | "confidential"
            | "equipment"
            | "information"
            | "party"
    )
}
