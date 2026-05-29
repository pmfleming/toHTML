pub(in crate::pdf::visual::text_repair) fn repair_joined_word_boundaries(text: &str) -> String {
    text.split_whitespace()
        .map(repair_joined_token)
        .collect::<Vec<_>>()
        .join(" ")
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
