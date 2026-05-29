use std::collections::{HashMap, HashSet};

use crate::Block;

use super::paragraph_text;

pub(super) fn repeated_short_paragraphs(blocks: &[Block], page_count: usize) -> Vec<String> {
    let threshold = repeated_furniture_threshold(page_count);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for text in blocks.iter().filter_map(paragraph_text) {
        if let Some(key) = page_furniture_key(&text) {
            *counts.entry(key).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(text, count)| (count >= threshold).then_some(text))
        .collect()
}

pub(super) fn is_page_furniture(block: &Block, repeated: &[String]) -> bool {
    let Some(text) = paragraph_text(block) else {
        return false;
    };
    let repeated: HashSet<&str> = repeated.iter().map(String::as_str).collect();
    page_furniture_key(&text).is_some_and(|key| repeated.contains(key.as_str()))
        || is_page_number_footer(&text)
        || is_dash_wrapped_page_number(&text)
        || is_fraction_page_footer(&text)
}

fn repeated_furniture_threshold(page_count: usize) -> usize {
    if page_count <= 1 {
        return 2;
    }
    (page_count / 3).max(2)
}

fn page_furniture_key(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() > 100 || trimmed.is_empty() {
        return None;
    }
    let key = trimmed
        .split_whitespace()
        .map(|token| {
            if is_page_fraction(token) || token.chars().all(|ch| ch.is_ascii_digit()) {
                "#"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!key.is_empty()).then_some(key)
}

fn is_page_number_footer(text: &str) -> bool {
    let Some(number) = text.strip_suffix(" P a g e") else {
        return false;
    };
    number.trim().chars().all(|ch| ch.is_ascii_digit())
}

fn is_dash_wrapped_page_number(text: &str) -> bool {
    let trimmed = text.trim();
    let Some(without_left) = trimmed
        .strip_prefix('–')
        .or_else(|| trimmed.strip_prefix('-'))
    else {
        return false;
    };
    let Some(number) = without_left
        .trim()
        .strip_suffix('–')
        .or_else(|| without_left.trim().strip_suffix('-'))
    else {
        return false;
    };
    let number = number.trim();
    !number.is_empty() && number.len() <= 4 && number.chars().all(|ch| ch.is_ascii_digit())
}

fn is_fraction_page_footer(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed
        .split_whitespace()
        .last()
        .is_some_and(is_page_fraction)
        && trimmed.len() <= 100
}

fn is_page_fraction(token: &str) -> bool {
    let Some((page, total)) = token.split_once('/') else {
        return false;
    };
    let Ok(page) = page.parse::<usize>() else {
        return false;
    };
    let Ok(total) = total.parse::<usize>() else {
        return false;
    };
    page >= 1 && total >= page
}
