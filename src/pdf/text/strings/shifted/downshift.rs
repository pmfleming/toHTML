mod fiscal;
mod numeric;

pub(in crate::pdf::text::strings) use fiscal::repair_downshifted_fiscal_period_sequences;
pub(super) use fiscal::repair_mixed_fiscal_period_word;
use numeric::looks_like_short_downshifted_label;
pub(super) use numeric::{
    is_likely_plain_identifier, is_plain_numeric_token, repair_downshifted_subset_word,
    repair_shifted_numeric_word, split_outer_punctuation, starts_with_plain_acronym_before_digits,
};

pub(in crate::pdf::text::strings) fn repair_downshifted_connectors(
    mut words: Vec<String>,
) -> Vec<String> {
    for index in 0..words.len() {
        if words[index] != "C" {
            continue;
        }
        let previous = index
            .checked_sub(1)
            .and_then(|previous| words.get(previous))
            .is_some_and(|word| looks_like_short_downshifted_label(word));
        let next = words
            .get(index + 1)
            .is_some_and(|word| looks_like_short_downshifted_label(word));
        if previous && next {
            words[index] = "&".to_string();
        }
    }
    words
}
