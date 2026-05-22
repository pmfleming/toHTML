use std::collections::HashMap;

use super::super::cmap::CMap;
use super::super::fonts::FontMetrics;
use super::reader::{ArrayToken, MarkedProps, Reader, Token};
use super::state::TextState;
use super::strings::{is_readable_text, normalize_whitespace};
use super::syntax::is_text_showing_operator;
use super::types::TextSegment;

pub fn extract_segments_with_fonts(
    stream: &[u8],
    font_cmaps: &HashMap<String, CMap>,
    font_metrics: &HashMap<String, FontMetrics>,
    struct_roles: &HashMap<u32, String>,
) -> Vec<TextSegment> {
    let mut parser = TextParser::new(stream, font_cmaps, font_metrics, struct_roles);
    parser.parse();
    parser.into_segments()
}

const POSITIONAL_JUMP_THRESHOLD: f32 = -1000.0;

#[derive(Debug)]
enum Operand {
    Name(String),
    Text(DecodedText),
    ActualText(DecodedText),
    TextArray(Vec<TextArrayItem>),
    Number(f32),
    Mcid(u32),
}

#[derive(Debug, Clone)]
enum TextArrayItem {
    Text(DecodedText),
    Adjustment(f32),
}

#[derive(Debug, Clone)]
struct DecodedText {
    text: String,
    raw: Vec<u8>,
}

struct TextParser<'a> {
    reader: Reader<'a>,
    operands: Vec<Operand>,
    font_cmaps: &'a HashMap<String, CMap>,
    emitter: SegmentEmitter<'a>,
}

struct SegmentEmitter<'a> {
    segments: Vec<TextSegment>,
    state: TextState,
    state_stack: Vec<TextState>,
    marked_roles: Vec<String>,
    actual_text_stack: Vec<Option<DecodedText>>,
    font_metrics: &'a HashMap<String, FontMetrics>,
    struct_roles: &'a HashMap<u32, String>,
}

impl<'a> TextParser<'a> {
    fn new(
        bytes: &'a [u8],
        font_cmaps: &'a HashMap<String, CMap>,
        font_metrics: &'a HashMap<String, FontMetrics>,
        struct_roles: &'a HashMap<u32, String>,
    ) -> Self {
        Self {
            reader: Reader::new(bytes),
            operands: Vec::new(),
            font_cmaps,
            emitter: SegmentEmitter::new(font_metrics, struct_roles),
        }
    }

    fn parse(&mut self) {
        while let Some(token) = self.reader.next_token() {
            self.apply_token(token);
        }
    }

    fn into_segments(self) -> Vec<TextSegment> {
        self.emitter.into_segments()
    }

    fn apply_token(&mut self, token: Token) {
        match token {
            Token::Literal(bytes) | Token::Hex(bytes) => {
                let text = self.decode_text(bytes);
                self.operands.push(Operand::Text(text));
            }
            Token::Array(items) => {
                let items = self.decode_array(items);
                self.operands.push(Operand::TextArray(items));
            }
            Token::Name(name) => self.operands.push(Operand::Name(name)),
            Token::MarkedProps(props) => self.apply_marked_props(props),
            Token::Word(word) => self.apply_word(&word),
        }
    }

    fn apply_marked_props(&mut self, props: MarkedProps) {
        if let Some(bytes) = props.actual_text {
            let text = self.decode_text(bytes);
            self.operands.push(Operand::ActualText(text));
        }
        if let Some(mcid) = props.mcid {
            self.operands.push(Operand::Mcid(mcid));
        }
    }

    fn apply_word(&mut self, word: &str) {
        if is_text_showing_operator(word) {
            self.apply_text_showing_operator(word);
        } else if let Ok(number) = word.parse::<f32>() {
            self.operands.push(Operand::Number(number));
        } else {
            self.apply_state_operator(word);
            self.operands.clear();
        }
    }

    fn decode_array(&self, items: Vec<ArrayToken>) -> Vec<TextArrayItem> {
        items
            .into_iter()
            .map(|item| match item {
                ArrayToken::Text(bytes) => TextArrayItem::Text(self.decode_text(bytes)),
                ArrayToken::Adjustment(value) => TextArrayItem::Adjustment(value),
            })
            .collect()
    }

    fn apply_text_showing_operator(&mut self, operator: &str) {
        match operator {
            "TJ" => self.push_latest_array_split(),
            "Tj" => self.push_latest_text(),
            "'" => {
                self.emitter.next_line();
                self.push_latest_text();
            }
            "\"" => {
                self.apply_quote_spacing();
                self.emitter.next_line();
                self.push_latest_text();
            }
            _ => {}
        }
        self.operands.clear();
    }

    fn push_latest_text(&mut self) {
        if let Some(text) = self.latest_text() {
            self.emitter.push_decoded_segment(&text);
        }
    }

    fn push_latest_array_split(&mut self) {
        let Some(items) = self.latest_array() else {
            return;
        };
        let mut current = DecodedText {
            text: String::new(),
            raw: Vec::new(),
        };
        let mut pending_space = false;

        for item in &items {
            match item {
                TextArrayItem::Text(value) => {
                    push_array_text(&mut current, value, &mut pending_space);
                }
                TextArrayItem::Adjustment(value) => {
                    if *value <= POSITIONAL_JUMP_THRESHOLD {
                        if !current.text.is_empty() {
                            self.emitter.push_decoded_segment(&current);
                            current = DecodedText {
                                text: String::new(),
                                raw: Vec::new(),
                            };
                        }
                        self.emitter.apply_tj_adjustment(*value);
                        pending_space = false;
                    } else {
                        pending_space = pending_space || *value <= -120.0;
                    }
                }
            }
        }

        if !current.text.is_empty() {
            self.emitter.push_decoded_segment(&current);
        }
    }

    fn apply_state_operator(&mut self, operator: &str) {
        match operator {
            "BT" => self.emitter.begin_text_object(),
            "BMC" | "BDC" => self.begin_marked_content(),
            "EMC" => self.emitter.end_marked_content(),
            "q" => self.emitter.save_graphics_state(),
            "Q" => self.emitter.restore_graphics_state(),
            "cm" => self.concat_matrix(),
            "Tf" => self.apply_font(),
            "Tc" => self.apply_character_spacing(),
            "Tw" => self.apply_word_spacing(),
            "Tz" => self.apply_horizontal_scaling(),
            "TL" => self.apply_leading(),
            "Tr" => self.apply_rendering_mode(),
            "Ts" => self.apply_text_rise(),
            "Td" => self.move_text_position(false),
            "TD" => self.move_text_position(true),
            "Tm" => self.set_text_matrix(),
            "T*" => self.emitter.next_line(),
            _ => {}
        }
    }

    fn begin_marked_content(&mut self) {
        let Some(tag) = self.latest_name() else {
            return;
        };
        let role = self
            .latest_mcid()
            .and_then(|mcid| self.emitter.struct_role(mcid))
            .unwrap_or(tag);
        self.emitter
            .begin_marked_content(role, self.latest_actual_text());
    }

    fn latest_mcid(&self) -> Option<u32> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Mcid(mcid) => Some(*mcid),
                _ => None,
            })
    }

    fn apply_font(&mut self) {
        if let Some(size) = self.latest_number() {
            self.emitter.set_font_size(size);
        }
        if let Some(name) = self.latest_name() {
            self.emitter.set_font_name(name);
        }
    }

    fn apply_leading(&mut self) {
        if let Some(leading) = self.latest_number() {
            self.emitter.set_leading(leading);
        }
    }

    fn apply_character_spacing(&mut self) {
        if let Some(spacing) = self.latest_number() {
            self.emitter.set_character_spacing(spacing);
        }
    }

    fn apply_word_spacing(&mut self) {
        if let Some(spacing) = self.latest_number() {
            self.emitter.set_word_spacing(spacing);
        }
    }

    fn apply_horizontal_scaling(&mut self) {
        if let Some(scaling) = self.latest_number() {
            self.emitter.set_horizontal_scaling(scaling);
        }
    }

    fn apply_text_rise(&mut self) {
        if let Some(rise) = self.latest_number() {
            self.emitter.set_text_rise(rise);
        }
    }

    fn apply_rendering_mode(&mut self) {
        if let Some(mode) = self.latest_number() {
            self.emitter.set_rendering_mode(mode as i32);
        }
    }

    fn apply_quote_spacing(&mut self) {
        let values: Vec<f32> = self.operands.iter().filter_map(number_operand).collect();
        if values.len() >= 2 {
            self.emitter.set_word_spacing(values[values.len() - 2]);
            self.emitter.set_character_spacing(values[values.len() - 1]);
        }
    }

    fn move_text_position(&mut self, update_leading: bool) {
        let Some((tx, ty)) = self.latest_two_numbers() else {
            return;
        };
        self.emitter.move_position(tx, ty);
        if update_leading {
            self.emitter.set_leading(ty);
        }
    }

    fn set_text_matrix(&mut self) {
        if let Some(values) = self.latest_six_numbers() {
            self.emitter.set_text_matrix(values);
        }
    }

    fn concat_matrix(&mut self) {
        if let Some(values) = self.latest_six_numbers() {
            self.emitter.concat_matrix(values);
        }
    }

    fn latest_text(&self) -> Option<DecodedText> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Text(text) => Some(text.clone()),
                _ => None,
            })
    }

    fn latest_array(&self) -> Option<Vec<TextArrayItem>> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::TextArray(items) => Some(items.clone()),
                _ => None,
            })
    }

    fn latest_number(&self) -> Option<f32> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Number(number) => Some(*number),
                _ => None,
            })
    }

    fn latest_two_numbers(&self) -> Option<(f32, f32)> {
        let values: Vec<f32> = self.operands.iter().filter_map(number_operand).collect();
        if values.len() < 2 {
            return None;
        }
        let last = values.len() - 1;
        Some((values[last - 1], values[last]))
    }

    fn latest_six_numbers(&self) -> Option<[f32; 6]> {
        let values: Vec<f32> = self.operands.iter().filter_map(number_operand).collect();
        last_six_numbers(&values)
    }

    fn latest_name(&self) -> Option<String> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::Name(name) => Some(name.clone()),
                _ => None,
            })
    }

    fn latest_actual_text(&self) -> Option<DecodedText> {
        self.operands
            .iter()
            .rev()
            .find_map(|operand| match operand {
                Operand::ActualText(text) => Some(text.clone()),
                _ => None,
            })
    }

    fn decode_text(&self, bytes: Vec<u8>) -> DecodedText {
        let text = self
            .emitter
            .font_name()
            .and_then(|name| self.font_cmaps.get(name))
            .map(|cmap| cmap.decode(&bytes))
            .unwrap_or_else(|| super::strings::decode_pdf_text_string(&bytes));
        DecodedText { text, raw: bytes }
    }
}

impl<'a> SegmentEmitter<'a> {
    fn new(
        font_metrics: &'a HashMap<String, FontMetrics>,
        struct_roles: &'a HashMap<u32, String>,
    ) -> Self {
        Self {
            segments: Vec::new(),
            state: TextState::default(),
            state_stack: Vec::new(),
            marked_roles: Vec::new(),
            actual_text_stack: Vec::new(),
            font_metrics,
            struct_roles,
        }
    }

    fn into_segments(self) -> Vec<TextSegment> {
        self.segments
    }

    fn push_decoded_segment(&mut self, decoded: &DecodedText) {
        let replacement = self.current_actual_text();
        let decoded = replacement.as_ref().unwrap_or(decoded);
        let text =
            super::strings::repair_shifted_subset_words(&normalize_whitespace(&decoded.text));
        if !is_readable_text(&text) || !self.state.is_visible_text() || self.in_artifact() {
            return;
        }

        let width = self.current_metrics().map(|metrics| {
            metrics.text_width(&decoded.raw, self.state.font_size(), text.chars().count())
        });
        let segment = self
            .state
            .segment(text, width)
            .with_role(self.current_role());
        let advance = width.unwrap_or(segment.width);
        self.state.advance_text(&segment.text, advance);
        self.segments.push(segment);
    }

    fn begin_text_object(&mut self) {
        self.state.begin_text_object();
    }

    fn next_line(&mut self) {
        self.state.next_line();
    }

    fn apply_tj_adjustment(&mut self, value: f32) {
        self.state.apply_tj_adjustment(value);
    }

    fn save_graphics_state(&mut self) {
        self.state_stack.push(self.state.clone());
    }

    fn restore_graphics_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
    }

    fn begin_marked_content(&mut self, role: String, actual_text: Option<DecodedText>) {
        self.marked_roles.push(role);
        self.actual_text_stack.push(actual_text);
    }

    fn end_marked_content(&mut self) {
        self.marked_roles.pop();
        self.actual_text_stack.pop();
    }

    fn set_font_size(&mut self, size: f32) {
        self.state.set_font_size(size);
    }

    fn set_font_name(&mut self, name: String) {
        self.state.font_name = Some(name);
    }

    fn set_leading(&mut self, leading: f32) {
        self.state.set_leading(leading);
    }

    fn set_character_spacing(&mut self, spacing: f32) {
        self.state.set_character_spacing(spacing);
    }

    fn set_word_spacing(&mut self, spacing: f32) {
        self.state.set_word_spacing(spacing);
    }

    fn set_horizontal_scaling(&mut self, scaling: f32) {
        self.state.set_horizontal_scaling(scaling);
    }

    fn set_text_rise(&mut self, rise: f32) {
        self.state.set_text_rise(rise);
    }

    fn set_rendering_mode(&mut self, mode: i32) {
        self.state.set_rendering_mode(mode);
    }

    fn move_position(&mut self, tx: f32, ty: f32) {
        self.state.move_position(tx, ty);
    }

    fn set_text_matrix(&mut self, values: [f32; 6]) {
        self.state.set_text_matrix(values);
    }

    fn concat_matrix(&mut self, values: [f32; 6]) {
        self.state.concat_matrix(values);
    }

    fn font_name(&self) -> Option<&str> {
        self.state.font_name.as_deref()
    }

    fn struct_role(&self, mcid: u32) -> Option<String> {
        self.struct_roles.get(&mcid).cloned()
    }

    fn current_metrics(&self) -> Option<&FontMetrics> {
        self.font_name()
            .and_then(|name| self.font_metrics.get(name))
    }

    fn current_role(&self) -> Option<String> {
        self.marked_roles.last().cloned()
    }

    fn current_actual_text(&self) -> Option<DecodedText> {
        self.actual_text_stack.last().cloned().flatten()
    }

    fn in_artifact(&self) -> bool {
        self.marked_roles
            .last()
            .is_some_and(|role| role == "Artifact")
    }
}

fn number_operand(operand: &Operand) -> Option<f32> {
    match operand {
        Operand::Number(number) => Some(*number),
        _ => None,
    }
}

fn last_six_numbers(values: &[f32]) -> Option<[f32; 6]> {
    let values = values.get(values.len().checked_sub(6)?..)?;
    Some([
        values[0], values[1], values[2], values[3], values[4], values[5],
    ])
}

fn push_array_text(text: &mut DecodedText, value: &DecodedText, pending_space: &mut bool) {
    if *pending_space && needs_inserted_space(&text.text, &value.text) {
        text.text.push(' ');
        text.raw.push(b' ');
    }
    text.text.push_str(&value.text);
    text.raw.extend_from_slice(&value.raw);
    *pending_space = false;
}

fn needs_inserted_space(text: &str, value: &str) -> bool {
    !text.ends_with(' ') && !value.starts_with(' ') && !text.is_empty() && !value.is_empty()
}
