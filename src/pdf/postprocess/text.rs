use crate::{Block, Inline, Link, Paragraph, TableCell};

use super::code::{looks_like_xml_continuation, looks_like_xml_line};

pub(super) fn repair_block(block: Block) -> Block {
    match block {
        Block::Paragraph(paragraph) => Block::Paragraph(repair_paragraph(paragraph)),
        Block::Table(mut table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    repair_cell(cell);
                }
            }
            Block::Table(table)
        }
        Block::Heading(mut heading) => {
            heading.content = repair_inlines(heading.content);
            Block::Heading(heading)
        }
        other => other,
    }
}

fn repair_paragraph(paragraph: Paragraph) -> Paragraph {
    Paragraph {
        content: repair_inlines(paragraph.content),
        source: paragraph.source,
    }
}

fn repair_cell(cell: &mut TableCell) {
    cell.content = repair_inlines(std::mem::take(&mut cell.content));
}

fn repair_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    inlines.into_iter().map(repair_inline).collect()
}

fn repair_inline(inline: Inline) -> Inline {
    match inline {
        Inline::Text(text) => Inline::Text(repair_text(&text)),
        Inline::Emphasis(content) => Inline::Emphasis(repair_inlines(content)),
        Inline::Strong(content) => Inline::Strong(repair_inlines(content)),
        Inline::Strikethrough(content) => Inline::Strikethrough(repair_inlines(content)),
        Inline::Link(link) => Inline::Link(repair_link(link)),
        other => other,
    }
}

fn repair_link(link: Link) -> Link {
    Link {
        content: repair_inlines(link.content),
        ..link
    }
}

fn repair_text(text: &str) -> String {
    let repaired = strip_license_artifact_runs(text);
    if looks_like_xml_line(&repaired) || looks_like_xml_continuation(&repaired) {
        return repaired;
    }
    repair_joined_word_boundaries(&repaired)
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

fn repair_joined_word_boundaries(text: &str) -> String {
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
