use crate::{Block, Heading};

use super::inlines::parse_inlines;
use super::source::markdown_source;

pub struct Fence {
    pub marker: &'static str,
    pub language: Option<String>,
}

pub fn parse_heading(line: &str) -> Option<Block> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) || !trimmed[level..].starts_with(' ') {
        return None;
    }

    Some(Block::Heading(Heading {
        level: level as u8,
        content: parse_inlines(trimmed[level..].trim()),
        source: markdown_source(),
    }))
}

pub fn horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(marker) = trimmed.chars().next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '_') {
        return false;
    }
    trimmed.len() >= 3 && trimmed.chars().all(|ch| ch == marker)
}

pub fn quote_start(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

pub fn strip_quote_marker(line: &str) -> &str {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix('>')
        .unwrap_or(trimmed)
        .strip_prefix(' ')
        .unwrap_or_else(|| trimmed.strip_prefix('>').unwrap_or(trimmed))
}

pub fn fence_start(line: &str) -> Option<Fence> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("```") {
        return Some(Fence {
            marker: "```",
            language: language(rest),
        });
    }
    if let Some(rest) = trimmed.strip_prefix("~~~") {
        return Some(Fence {
            marker: "~~~",
            language: language(rest),
        });
    }
    None
}

fn language(rest: &str) -> Option<String> {
    let language = rest.trim();
    (!language.is_empty()).then(|| language.to_string())
}
