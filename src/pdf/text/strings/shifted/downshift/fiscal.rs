use super::numeric::{can_downshift_subset_char, downshift_subset_text, split_outer_punctuation};

pub(in crate::pdf::text::strings) fn repair_downshifted_fiscal_period_sequences(
    words: Vec<String>,
) -> Vec<String> {
    let mut repaired = Vec::with_capacity(words.len());
    let mut index = 0;
    while index < words.len() {
        if let Some(period) = words
            .get(index..index + 2)
            .and_then(repair_downshifted_fiscal_period_sequence)
        {
            repaired.push(period);
            index += 2;
            continue;
        }
        if let Some(period) = words
            .get(index..index + 1)
            .and_then(repair_downshifted_fiscal_period_sequence)
        {
            repaired.push(period);
            index += 1;
            continue;
        }

        repaired.push(words[index].clone());
        index += 1;
    }
    repaired
}

fn repair_downshifted_fiscal_period_sequence(words: &[String]) -> Option<String> {
    if words.is_empty()
        || !words
            .iter()
            .any(|word| word.chars().any(can_downshift_subset_char))
    {
        return None;
    }

    let mut combined = String::new();
    let mut prefix = "";
    let mut suffix = "";
    for (index, word) in words.iter().enumerate() {
        let (word_prefix, core, word_suffix) = split_outer_punctuation(word);
        if index == 0 {
            prefix = word_prefix;
        } else if !word_prefix.is_empty() {
            return None;
        }
        if index + 1 == words.len() {
            suffix = word_suffix;
        } else if !word_suffix.is_empty() {
            return None;
        }
        combined.push_str(core);
    }

    let repaired = downshift_subset_text(&combined);
    if looks_like_fiscal_period_expression(&repaired)
        || looks_like_fiscal_half_label(&repaired)
        || looks_like_prefixed_fiscal_half_label(&repaired)
    {
        Some(format!("{prefix}{repaired}{suffix}"))
    } else {
        None
    }
}

fn looks_like_fiscal_period_expression(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    chars.len() == 9
        && chars[0] == 'H'
        && matches!(chars[1], '1' | '2')
        && chars[2] == '('
        && chars[3] == 'Q'
        && matches!(chars[4], '1'..='4')
        && chars[5] == '+'
        && chars[6] == 'Q'
        && matches!(chars[7], '1'..='4')
        && chars[8] == ')'
}

fn looks_like_fiscal_half_label(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    chars.len() == 6
        && chars[0] == '2'
        && chars[1] == '0'
        && chars[2].is_ascii_digit()
        && chars[3].is_ascii_digit()
        && chars[4] == 'H'
        && matches!(chars[5], '1' | '2')
}

fn looks_like_prefixed_fiscal_half_label(text: &str) -> bool {
    text.char_indices().skip(1).any(|(index, _)| {
        let (prefix, label) = text.split_at(index);
        !prefix.is_empty()
            && prefix.chars().all(is_formula_prefix_char)
            && looks_like_fiscal_half_label(label)
    })
}

fn is_formula_prefix_char(ch: char) -> bool {
    matches!(ch, '=' | '(' | '[' | '<' | '>' | '/' | '≤' | '≥')
}

pub(in crate::pdf::text::strings::shifted) fn repair_mixed_fiscal_period_word(
    word: &str,
) -> Option<String> {
    let (prefix, core, suffix) = split_outer_punctuation(word);
    let repaired = repair_mixed_fiscal_period_core(core)?;
    Some(format!("{prefix}{repaired}{suffix}"))
}

fn repair_mixed_fiscal_period_core(core: &str) -> Option<String> {
    let mut chars = core.char_indices();
    let (_, first) = chars.next()?;
    let (second_index, second) = chars.next()?;
    if first != 'e' || !second.is_ascii_digit() {
        return None;
    }

    let rest = &core[second_index + second.len_utf8()..];
    if !rest.starts_with('(') || !(rest.contains("Q3") || rest.contains("Q4")) {
        return None;
    }

    let mut repaired = String::with_capacity(core.len());
    repaired.push('H');
    repaired.push(second);
    repaired.push_str(rest);
    Some(repaired)
}
