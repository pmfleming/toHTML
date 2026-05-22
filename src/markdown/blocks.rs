use crate::{
    Block, BlockQuote, CodeBlock, Heading, List, ListItem, Paragraph, SourceFormat, SourceSpan,
};

use super::inlines::parse_inlines;
use super::tables::{parse_table, table_start};

pub fn parse_blocks(input: &str) -> Vec<Block> {
    let lines: Vec<&str> = input.lines().collect();
    let mut parser = BlockParser { lines, index: 0 };
    parser.parse_until_end()
}

struct BlockParser<'a> {
    lines: Vec<&'a str>,
    index: usize,
}

impl BlockParser<'_> {
    fn parse_until_end(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        while let Some(line) = self.current() {
            if line.trim().is_empty() {
                self.index += 1;
                continue;
            }
            blocks.push(self.parse_block());
        }
        blocks
    }

    fn parse_block(&mut self) -> Block {
        let line = self.current().unwrap_or_default().to_string();
        if let Some(block) = self.parse_fenced_code(&line) {
            return block;
        }
        if let Some(block) = parse_heading(&line) {
            self.index += 1;
            return block;
        }
        if horizontal_rule(&line) {
            self.index += 1;
            return Block::HorizontalRule;
        }
        if quote_start(&line) {
            return self.parse_block_quote();
        }
        if let Some(item) = parse_list_item(&line) {
            return self.parse_list(item);
        }
        if table_start(&self.lines, self.index) {
            return self.parse_table_block();
        }
        self.parse_paragraph()
    }

    fn parse_fenced_code(&mut self, line: &str) -> Option<Block> {
        let fence = fence_start(line)?;
        self.index += 1;
        let mut code = String::new();
        while let Some(next) = self.current() {
            if next.trim_start().starts_with(fence.marker) {
                self.index += 1;
                break;
            }
            code.push_str(next);
            code.push('\n');
            self.index += 1;
        }
        Some(Block::CodeBlock(CodeBlock {
            language: fence.language,
            code,
            source: markdown_source(),
        }))
    }

    fn parse_block_quote(&mut self) -> Block {
        let mut quoted = String::new();
        while let Some(line) = self.current() {
            if !quote_start(line) {
                break;
            }
            quoted.push_str(strip_quote_marker(line));
            quoted.push('\n');
            self.index += 1;
        }
        Block::BlockQuote(BlockQuote {
            blocks: parse_blocks(&quoted),
            source: markdown_source(),
        })
    }

    fn parse_list(&mut self, first: ParsedListItem) -> Block {
        let ordered = first.ordered;
        let start = first.number;
        let mut items = Vec::new();
        items.push(first.into_item());
        self.index += 1;

        while let Some(line) = self.current() {
            let Some(item) = parse_list_item(line) else {
                break;
            };
            if item.ordered != ordered {
                break;
            }
            items.push(item.into_item());
            self.index += 1;
        }

        Block::List(List {
            ordered,
            start,
            items,
            source: markdown_source(),
        })
    }

    fn parse_table_block(&mut self) -> Block {
        let (table, consumed) = parse_table(&self.lines[self.index..]);
        self.index += consumed;
        Block::Table(table)
    }

    fn parse_paragraph(&mut self) -> Block {
        let mut text = String::new();
        while let Some(line) = self.current() {
            if paragraph_boundary(&self.lines, self.index) {
                break;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(line.trim());
            self.index += 1;
        }
        Block::Paragraph(Paragraph {
            content: parse_inlines(&text),
            source: markdown_source(),
        })
    }

    fn current(&self) -> Option<&str> {
        self.lines.get(self.index).copied()
    }
}

fn parse_heading(line: &str) -> Option<Block> {
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

fn horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(marker) = trimmed.chars().next() else {
        return false;
    };
    if !matches!(marker, '-' | '*' | '_') {
        return false;
    }
    trimmed.len() >= 3 && trimmed.chars().all(|ch| ch == marker)
}

fn quote_start(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

fn strip_quote_marker(line: &str) -> &str {
    line.trim_start()
        .strip_prefix('>')
        .unwrap_or(line)
        .strip_prefix(' ')
        .unwrap_or(line.trim_start().strip_prefix('>').unwrap_or(line))
}

struct Fence {
    marker: &'static str,
    language: Option<String>,
}

fn fence_start(line: &str) -> Option<Fence> {
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

struct ParsedListItem {
    ordered: bool,
    number: Option<u64>,
    checked: Option<bool>,
    text: String,
}

impl ParsedListItem {
    fn into_item(self) -> ListItem {
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

fn parse_list_item(line: &str) -> Option<ParsedListItem> {
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

fn paragraph_boundary(lines: &[&str], index: usize) -> bool {
    let Some(line) = lines.get(index).copied() else {
        return true;
    };
    line.trim().is_empty()
        || fence_start(line).is_some()
        || parse_heading(line).is_some()
        || horizontal_rule(line)
        || quote_start(line)
        || parse_list_item(line).is_some()
        || table_start(lines, index)
}

fn markdown_source() -> Option<SourceSpan> {
    Some(SourceSpan {
        format: SourceFormat::Markdown,
        page: None,
        path: None,
        byte_range: None,
    })
}
