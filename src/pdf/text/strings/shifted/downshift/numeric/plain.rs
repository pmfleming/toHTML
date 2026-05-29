pub(in crate::pdf::text::strings::shifted) fn is_plain_numeric_token(word: &str) -> bool {
    let trimmed = word.trim_matches(|ch: char| matches!(ch, ',' | ';' | ':' | '(' | ')'));
    if trimmed.is_empty() {
        return false;
    }

    let without_trailing_period = trimmed
        .strip_suffix('.')
        .filter(|value| !value.contains('.'))
        .unwrap_or(trimmed);
    let numeric = without_trailing_period
        .strip_prefix('-')
        .unwrap_or(without_trailing_period);
    if numeric.is_empty() {
        return false;
    }

    let decimal_separator = if numeric.contains(',') { ',' } else { '.' };
    let mut parts = numeric.split(decimal_separator);
    let Some(whole) = parts.next() else {
        return false;
    };
    if whole.is_empty() || !whole.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    match (parts.next(), parts.next()) {
        (None, None) => true,
        (Some(fraction), None) => {
            !fraction.is_empty() && fraction.chars().all(|ch| ch.is_ascii_digit())
        }
        _ => false,
    }
}

pub(in crate::pdf::text::strings::shifted) fn is_likely_plain_identifier(word: &str) -> bool {
    let has_digit = word.chars().any(|ch| ch.is_ascii_digit());
    let allowed = word.chars().all(|ch| {
        ch.is_ascii_uppercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.' | '/' | '$')
    });
    word.len() >= 4 && has_digit && allowed
}

pub(in crate::pdf::text::strings::shifted) fn starts_with_plain_acronym_before_digits(
    text: &str,
) -> bool {
    for split_at in 2..=3 {
        let Some((prefix, suffix)) = text
            .char_indices()
            .nth(split_at)
            .map(|(index, _)| text.split_at(index))
        else {
            continue;
        };
        if prefix.chars().all(|ch| ch.is_ascii_uppercase())
            && suffix.chars().next().is_some_and(|ch| ch.is_ascii_digit())
        {
            return true;
        }
    }
    false
}
