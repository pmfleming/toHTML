use super::super::encoding::decode_shifted_subset_text;
use super::super::scoring::text_score;
use super::has_shifted_subset_marker;

pub(in crate::pdf::text::strings) fn repair_mixed_shifted_subset_word(word: &str) -> String {
    let mut repaired = String::new();
    let mut chunk = String::new();
    let mut shifted_chunk = None;

    for ch in word.chars() {
        let is_shifted = is_shifted_subset_byte(ch);
        match shifted_chunk {
            Some(current) if current == is_shifted => chunk.push(ch),
            Some(_) => {
                repaired.push_str(&repair_shifted_subset_chunk(&chunk));
                chunk.clear();
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
            None => {
                chunk.push(ch);
                shifted_chunk = Some(is_shifted);
            }
        }
    }

    if !chunk.is_empty() {
        repaired.push_str(&repair_shifted_subset_chunk(&chunk));
    }

    repaired
}

fn repair_shifted_subset_chunk(chunk: &str) -> String {
    if !chunk.is_ascii() {
        return chunk.to_string();
    }
    if !chunk.chars().any(is_shifted_subset_byte) {
        return chunk.to_string();
    }
    if chunk.chars().all(|ch| ch.is_ascii_alphabetic())
        && !has_shifted_subset_marker(chunk)
        && !shifted_candidate_is_useful(chunk)
    {
        return chunk.to_string();
    }

    let shifted = decode_shifted_subset_text(chunk.as_bytes());
    if shifted != chunk
        && (text_score(&shifted) >= text_score(chunk) || shifted_candidate_is_useful(chunk))
    {
        shifted
    } else {
        chunk.to_string()
    }
}

fn is_shifted_subset_byte(ch: char) -> bool {
    ch.is_ascii() && ('!'..='`').contains(&ch)
}

pub(super) fn shifted_candidate_is_useful(chunk: &str) -> bool {
    let shifted = decode_shifted_subset_text(chunk.as_bytes()).to_ascii_lowercase();
    USEFUL_SHIFTED_TERMS
        .split_whitespace()
        .any(|word| shifted.contains(word))
}

const USEFUL_SHIFTED_TERMS: &str = "agreement agreements changed either made other accountants accounts agents applicable authorized available breach contained covenant certain confidential consideration conclusion construed contract copies disclosing disclosure documents delivered destroyed developed drawings employees exchange efforts furnished forth information including independently instructions limitation limited mutual officers party parties process property provided produced product promptly prevent purpose receiving relating remain representatives requisitions results specifications shall sole such survive the therefore this transaction under warranties will with written";
