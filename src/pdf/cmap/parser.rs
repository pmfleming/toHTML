use super::hex::{code_bytes, code_range, code_value};
use super::predefined;
use super::tokens::CMapToken;
use super::unicode::unicode_string;
use super::{CMap, CodeSpaceRange, WritingMode};

impl CMap {
    pub(super) fn parse_tokens(&mut self, tokens: &[CMapToken]) {
        let mut index = 0;
        while index < tokens.len() {
            match tokens.get(index) {
                Some(CMapToken::Word(word)) if word == "begincodespacerange" => {
                    index = self.parse_codespace_ranges(tokens, index + 1);
                }
                Some(CMapToken::Word(word)) if word == "beginbfchar" => {
                    index = self.parse_bfchar_mappings(tokens, index + 1);
                }
                Some(CMapToken::Word(word)) if word == "beginbfrange" => {
                    index = self.parse_bfrange_mappings(tokens, index + 1);
                }
                Some(CMapToken::Word(word)) if word == "beginnotdefchar" => {
                    index = self.parse_notdef_chars(tokens, index + 1);
                }
                Some(CMapToken::Word(word)) if word == "beginnotdefrange" => {
                    index = self.parse_notdef_ranges(tokens, index + 1);
                }
                Some(CMapToken::Name(name)) if name == "WMode" => {
                    if matches!(tokens.get(index + 1), Some(CMapToken::Integer(1))) {
                        self.writing_mode = WritingMode::Vertical;
                    }
                    index += 1;
                }
                Some(CMapToken::Name(name)) if matches!(tokens.get(index + 1), Some(CMapToken::Word(word)) if word == "usecmap") =>
                {
                    if let Some(predefined) = predefined::predefined_cmap(name) {
                        self.max_code_len = self.max_code_len.max(predefined.max_code_len());
                        if self.fallback.is_none() {
                            self.fallback = Some(predefined);
                        }
                    } else {
                        self.warnings
                            .push(format!("unsupported CMap usecmap /{name}"));
                    }
                    index += 2;
                }
                _ => index += 1,
            }
        }
    }

    fn parse_codespace_ranges(&mut self, tokens: &[CMapToken], mut index: usize) -> usize {
        while !section_ends(tokens.get(index), "endcodespacerange") {
            let Some((start, end, next)) = hex_pair_range(tokens, index) else {
                index += 1;
                continue;
            };
            self.max_code_len = self.max_code_len.max(start.len()).max(end.len());
            self.codespaces.push(CodeSpaceRange { start, end });
            index = next;
        }
        index.saturating_add(1)
    }

    fn parse_bfchar_mappings(&mut self, tokens: &[CMapToken], mut index: usize) -> usize {
        while !section_ends(tokens.get(index), "endbfchar") {
            match (tokens.get(index), tokens.get(index + 1)) {
                (Some(CMapToken::Hex(source)), Some(CMapToken::Hex(target))) => {
                    self.add_mapping(source.clone(), unicode_string(target));
                    index += 2;
                }
                _ => index += 1,
            }
        }
        index.saturating_add(1)
    }

    fn parse_bfrange_mappings(&mut self, tokens: &[CMapToken], mut index: usize) -> usize {
        while !section_ends(tokens.get(index), "endbfrange") {
            let (Some(CMapToken::Hex(start)), Some(CMapToken::Hex(end))) =
                (tokens.get(index), tokens.get(index + 1))
            else {
                index += 1;
                continue;
            };
            match tokens.get(index + 2) {
                Some(CMapToken::ArrayStart) => {
                    index = self.add_array_range_tokens(start, end, tokens, index + 3);
                }
                Some(CMapToken::Hex(target)) => {
                    self.add_range(start, end, target);
                    index += 3;
                }
                _ => index += 1,
            }
        }
        index.saturating_add(1)
    }

    fn parse_notdef_chars(&mut self, tokens: &[CMapToken], mut index: usize) -> usize {
        while !section_ends(tokens.get(index), "endnotdefchar") {
            if let Some(CMapToken::Hex(source)) = tokens.get(index) {
                self.notdef.push(CodeSpaceRange {
                    start: source.clone(),
                    end: source.clone(),
                });
            }
            index += 1;
        }
        index.saturating_add(1)
    }

    fn parse_notdef_ranges(&mut self, tokens: &[CMapToken], mut index: usize) -> usize {
        while !section_ends(tokens.get(index), "endnotdefrange") {
            let Some((start, end, next)) = hex_pair_range(tokens, index) else {
                index += 1;
                continue;
            };
            self.notdef.push(CodeSpaceRange { start, end });
            index = next;
        }
        index.saturating_add(1)
    }

    fn add_range(&mut self, start: &[u8], end: &[u8], target: &[u8]) {
        let source = start;
        let Some((start, end)) = code_range(source, end) else {
            return;
        };
        let Some(target) = code_value(target) else {
            return;
        };

        self.add_range_mapping(source, start, end, target);
    }

    fn add_array_range_tokens(
        &mut self,
        source_start: &[u8],
        source_end: &[u8],
        tokens: &[CMapToken],
        mut index: usize,
    ) -> usize {
        let Some((start, end)) = code_range(source_start, source_end) else {
            return index;
        };

        for offset in 0..range_len(start, end) {
            let Some(CMapToken::Hex(target)) = tokens.get(index) else {
                break;
            };
            self.add_mapping(
                code_bytes(start + offset as u32, source_start),
                unicode_string(target),
            );
            index += 1;
        }
        while !matches!(tokens.get(index), Some(CMapToken::ArrayEnd) | None) {
            index += 1;
        }
        index.saturating_add(1)
    }
}

fn range_len(start: u32, end: u32) -> usize {
    end.saturating_sub(start).saturating_add(1) as usize
}

fn section_ends(token: Option<&CMapToken>, end_word: &str) -> bool {
    match token {
        None => true,
        Some(CMapToken::Word(word)) => word == end_word,
        _ => false,
    }
}

fn hex_pair_range(tokens: &[CMapToken], index: usize) -> Option<(Vec<u8>, Vec<u8>, usize)> {
    match (tokens.get(index), tokens.get(index + 1)) {
        (Some(CMapToken::Hex(start)), Some(CMapToken::Hex(end))) => {
            Some((start.clone(), end.clone(), index + 2))
        }
        _ => None,
    }
}
