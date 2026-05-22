mod font_refs;
mod hex;
#[cfg(test)]
mod tests;
mod unicode;

use std::collections::HashMap;

use hex::{code_bytes, code_range, code_value, hex_tokens};
use unicode::{readable_byte, unicode_scalar, unicode_string};

#[derive(Debug, Clone, Default)]
pub struct CMap {
    entries: HashMap<Vec<u8>, String>,
    max_code_len: usize,
}

impl CMap {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut cmap = Self::default();
        for line in String::from_utf8_lossy(bytes).lines() {
            cmap.add_line(line);
        }
        cmap
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        let mut output = String::new();
        let mut index = 0;

        while index < bytes.len() {
            match self.lookup(bytes, index) {
                Some((text, consumed)) => {
                    output.push_str(text);
                    index += consumed;
                }
                None => {
                    push_fallback_byte(&mut output, bytes[index]);
                    index += 1;
                }
            }
        }

        output
    }

    fn add_line(&mut self, line: &str) {
        if is_cmap_marker(line) {
            return;
        }

        match hex_tokens(line).as_slice() {
            [source, target] => self.add_mapping(source.clone(), unicode_string(target)),
            tokens @ [_, _, _, ..] if line.contains('[') => self.add_array_range(tokens),
            [start, end, target, ..] => self.add_range(start, end, target),
            _ => {}
        }
    }

    fn add_mapping(&mut self, source: Vec<u8>, target: String) {
        self.max_code_len = self.max_code_len.max(source.len());
        self.entries.insert(source, target);
    }

    fn add_range(&mut self, start: &[u8], end: &[u8], target: &[u8]) {
        let source = start;
        let Some((start, end)) = code_range(source, end) else {
            return;
        };
        let Some(target) = code_value(target) else {
            return;
        };

        for code in start..=end {
            self.add_mapping(
                code_bytes(code, source),
                unicode_scalar(target + code - start),
            );
        }
    }

    fn add_array_range(&mut self, tokens: &[Vec<u8>]) {
        let source = &tokens[0];
        let Some((start, end)) = code_range(source, &tokens[1]) else {
            return;
        };

        for (offset, target) in tokens
            .iter()
            .skip(2)
            .take(range_len(start, end))
            .enumerate()
        {
            self.add_mapping(
                code_bytes(start + offset as u32, source),
                unicode_string(target),
            );
        }
    }

    fn lookup(&self, bytes: &[u8], index: usize) -> Option<(&str, usize)> {
        let max_len = self.max_code_len.min(bytes.len() - index);
        for len in (1..=max_len).rev() {
            if let Some(text) = self.entries.get(&bytes[index..index + len]) {
                return Some((text, len));
            }
        }
        None
    }
}

pub fn font_cmaps(bytes: &[u8]) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    font_refs::font_cmaps(bytes)
}

fn push_fallback_byte(output: &mut String, byte: u8) {
    if let Some(ch) = readable_byte(byte) {
        output.push(ch);
    }
}

fn range_len(start: u32, end: u32) -> usize {
    end.saturating_sub(start).saturating_add(1) as usize
}

fn is_cmap_marker(line: &str) -> bool {
    line.contains("beginbfchar")
        || line.contains("endbfchar")
        || line.contains("beginbfrange")
        || line.contains("endbfrange")
}
