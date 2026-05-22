use crate::{Block, ListItem, Paragraph};

use super::inlines::parse_inlines;
use super::source::markdown_source;

pub struct ParsedListItem {
    pub ordered: bool,
    pub number: Option<u64>,
    pub checked: Option<bool>,
    pub text: String,
}

impl ParsedListItem {
    pub fn into_item(self) -> ListItem {
        ListItem {
            checked: self.checked,
            blocks: vec![Block::Paragraph(Paragraph {
                content: parse_inlines(&self.text),
                source: markdown_source(),
            })],
            source: markdown_source(),
        }
    }
}

pub fn parse_list_item(line: &str) -> Option<ParsedListItem> {
    let trimmed = line.trim_start();
    parse_unordered_item(trimmed).or_else(|| parse_ordered_item(trimmed))
}

fn parse_unordered_item(trimmed: &str) -> Option<ParsedListItem> {
    let marker = trimmed.chars().next()?;
    if !matches!(marker, '-' | '*' | '+') || !trimmed[1..].starts_with(' ') {
        return None;
    }
    let (checked, text) = parse_task_marker(trimmed[2..].trim_start());
    Some(ParsedListItem {
        ordered: false,
        number: None,
        checked,
        text: text.to_string(),
    })
}

fn parse_ordered_item(trimmed: &str) -> Option<ParsedListItem> {
    let digits = trimmed.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits == 0 || !trimmed[digits..].starts_with(". ") {
        return None;
    }
    let number = trimmed[..digits].parse().ok();
    let (checked, text) = parse_task_marker(trimmed[digits + 2..].trim_start());
    Some(ParsedListItem {
        ordered: true,
        number,
        checked,
        text: text.to_string(),
    })
}

fn parse_task_marker(text: &str) -> (Option<bool>, &str) {
    if let Some(rest) = text.strip_prefix("[ ] ") {
        return (Some(false), rest);
    }
    if let Some(rest) = text
        .strip_prefix("[x] ")
        .or_else(|| text.strip_prefix("[X] "))
    {
        return (Some(true), rest);
    }
    (None, text)
}
