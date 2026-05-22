use crate::{Block, BlockQuote, CodeBlock, List, Paragraph};

use super::block_markers::{
    fence_start, horizontal_rule, parse_heading, quote_start, strip_quote_marker,
};
use super::inlines::parse_inlines;
use super::lists::{parse_list_item, ParsedListItem};
use super::source::markdown_source;
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
