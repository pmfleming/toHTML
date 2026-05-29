mod embedded;
mod encoding;
mod font_refs;
mod hex;
mod parser;
mod predefined;
#[cfg(test)]
mod tests;
mod tokens;
mod unicode;

use std::collections::HashMap;

use predefined::PredefinedCMap;
use tokens::cmap_tokens;
use unicode::readable_byte;

#[derive(Debug, Clone, Default)]
pub struct CMap {
    entries: HashMap<Vec<u8>, String>,
    ranges: Vec<RangeMapping>,
    max_code_len: usize,
    fallback: Option<PredefinedCMap>,
    codespaces: Vec<CodeSpaceRange>,
    notdef: Vec<CodeSpaceRange>,
    writing_mode: WritingMode,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CMapDecodeStats {
    pub mapped: usize,
    pub fallback_mapped: usize,
    pub notdef: usize,
    pub raw_fallback: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CMapDecodeResult {
    pub text: String,
    pub stats: CMapDecodeStats,
}

impl CMap {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut cmap = Self::default();
        cmap.parse_tokens(&cmap_tokens(bytes));
        cmap
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        self.decode_with_stats(bytes).text
    }

    pub fn decode_with_stats(&self, bytes: &[u8]) -> CMapDecodeResult {
        let mut output = String::new();
        let mut stats = CMapDecodeStats::default();
        let mut index = 0;

        while index < bytes.len() {
            match self.lookup(bytes, index) {
                Some((text, consumed)) => {
                    output.push_str(&text);
                    stats.mapped += 1;
                    index += consumed;
                }
                None => {
                    if let Some((text, consumed, source)) = self.lookup_fallback(bytes, index) {
                        output.push_str(&text);
                        match source {
                            FallbackSource::Predefined => stats.fallback_mapped += 1,
                            FallbackSource::NotDef => stats.notdef += 1,
                        }
                        index += consumed;
                    } else {
                        if let Some(consumed) = self.unmapped_codespace_len(bytes, index) {
                            stats.raw_fallback += 1;
                            index += consumed;
                            continue;
                        }
                        push_fallback_byte(&mut output, bytes[index]);
                        stats.raw_fallback += 1;
                        index += 1;
                    }
                }
            }
        }

        CMapDecodeResult {
            text: output,
            stats,
        }
    }

    pub(super) fn from_byte_mappings(mappings: impl IntoIterator<Item = (u8, String)>) -> Self {
        let mut cmap = Self::default();
        for (source, target) in mappings {
            cmap.add_mapping(vec![source], target);
        }
        cmap
    }

    pub(super) fn from_code_mappings(
        mappings: impl IntoIterator<Item = (Vec<u8>, String)>,
    ) -> Self {
        let mut cmap = Self::default();
        for (source, target) in mappings {
            cmap.add_mapping(source, target);
        }
        cmap
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    #[allow(dead_code)]
    pub fn writing_mode(&self) -> WritingMode {
        self.writing_mode
    }

    fn with_predefined_fallback(fallback: PredefinedCMap) -> Self {
        Self {
            entries: HashMap::new(),
            ranges: Vec::new(),
            max_code_len: fallback.max_code_len(),
            fallback: Some(fallback),
            codespaces: Vec::new(),
            notdef: Vec::new(),
            writing_mode: WritingMode::Horizontal,
            warnings: Vec::new(),
        }
    }

    pub(super) fn merge(&mut self, other: Self) {
        self.max_code_len = self.max_code_len.max(other.max_code_len);
        if self.fallback.is_none() {
            self.fallback = other.fallback;
        }
        self.codespaces.extend(other.codespaces);
        self.notdef.extend(other.notdef);
        if other.writing_mode == WritingMode::Vertical {
            self.writing_mode = WritingMode::Vertical;
        }
        self.warnings.extend(other.warnings);
        self.entries.extend(other.entries);
        self.ranges.extend(other.ranges);
    }

    fn add_mapping(&mut self, source: Vec<u8>, target: String) {
        self.max_code_len = self.max_code_len.max(source.len());
        self.entries.insert(source, target);
    }

    fn add_range_mapping(&mut self, source: &[u8], start: u32, end: u32, target: u32) {
        self.max_code_len = self.max_code_len.max(source.len());
        self.ranges.push(RangeMapping {
            source_len: source.len().max(1),
            start,
            end,
            target,
        });
    }

    fn lookup(&self, bytes: &[u8], index: usize) -> Option<(String, usize)> {
        let max_len = self.max_code_len.min(bytes.len() - index);
        for len in (1..=max_len).rev() {
            if let Some(text) = self.entries.get(&bytes[index..index + len]) {
                return Some((text.clone(), len));
            }
            if let Some(text) = self
                .ranges
                .iter()
                .find_map(|range| range.lookup(bytes, index, len))
            {
                return Some((text, len));
            }
        }
        None
    }

    fn lookup_fallback(
        &self,
        bytes: &[u8],
        index: usize,
    ) -> Option<(String, usize, FallbackSource)> {
        if let Some(len) = self.notdef_code_len(bytes, index) {
            return Some((String::new(), len, FallbackSource::NotDef));
        }
        self.fallback
            .as_ref()?
            .decode_prefix(bytes, index)
            .map(|(text, len)| (text, len, FallbackSource::Predefined))
    }

    fn notdef_code_len(&self, bytes: &[u8], index: usize) -> Option<usize> {
        self.notdef
            .iter()
            .find(|range| range.matches(bytes, index))
            .map(|range| range.start.len())
    }

    fn unmapped_codespace_len(&self, bytes: &[u8], index: usize) -> Option<usize> {
        self.codespaces
            .iter()
            .filter(|range| range.start.len() > 1)
            .find(|range| range.matches(bytes, index))
            .map(|range| range.start.len())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FallbackSource {
    Predefined,
    NotDef,
}

pub fn font_cmaps(bytes: &[u8]) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    font_refs::font_cmaps(bytes)
}

pub fn font_decoding_warnings(bytes: &[u8]) -> Result<Vec<String>, crate::ConvertError> {
    font_refs::font_decoding_warnings(bytes)
}

pub fn font_cmaps_for_resources(
    bytes: &[u8],
    objects: &super::object::PdfObjects,
    resources: &HashMap<String, super::object::PdfReference>,
) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    font_refs::font_cmaps_for_resources(bytes, objects, resources)
}

fn push_fallback_byte(output: &mut String, byte: u8) {
    if let Some(ch) = readable_byte(byte) {
        output.push(ch);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum WritingMode {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
struct CodeSpaceRange {
    start: Vec<u8>,
    end: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RangeMapping {
    source_len: usize,
    start: u32,
    end: u32,
    target: u32,
}

impl RangeMapping {
    fn lookup(&self, bytes: &[u8], index: usize, len: usize) -> Option<String> {
        if len != self.source_len || index + len > bytes.len() {
            return None;
        }
        let code = hex::code_value(&bytes[index..index + len])?;
        if code < self.start || code > self.end {
            return None;
        }
        let text = unicode::unicode_scalar(self.target + code - self.start);
        (!text.is_empty()).then_some(text)
    }
}

impl CodeSpaceRange {
    fn matches(&self, bytes: &[u8], index: usize) -> bool {
        let len = self.start.len();
        if len != self.end.len() || index + len > bytes.len() {
            return false;
        }
        let code = &bytes[index..index + len];
        code >= self.start.as_slice() && code <= self.end.as_slice()
    }
}
