use std::collections::HashMap;

use super::super::cmap::CMap;
use super::super::fonts::FontMetrics;
use super::reader::{ArrayToken, Reader, Token};
use super::state::TextState;
use super::strings::{decode_pdf_string, is_readable_text, normalize_whitespace};
use super::syntax::is_text_showing_operator;
use super::types::TextSegment;

pub fn extract_segments_with_fonts(
    stream: &[u8],
    font_cmaps: &HashMap<String, CMap>,
    font_metrics: &HashMap<String, FontMetrics>,
) -> Vec<TextSegment> {
    let mut parser = TextParser::new(stream, font_cmaps, font_metrics);
    parser.parse();
    parser.segments
}

#[derive(Debug)]
enum Operand {
    Name(String),
    Text(DecodedText),
    ActualText(DecodedText),
    TextArray(Vec<TextArrayItem>),
    Number(f32),
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
    segments: Vec<TextSegment>,
    state: TextState,
    state_stack: Vec<TextState>,
    marked_roles: Vec<String>,
    actual_text_stack: Vec<Option<DecodedText>>,
    font_cmaps: &'a HashMap<String, CMap>,
    font_metrics: &'a HashMap<String, FontMetrics>,
}

impl<'a> TextParser<'a> {
    fn new(
        bytes: &'a [u8],
        font_cmaps: &'a HashMap<String, CMap>,
        font_metrics: &'a HashMap<String, FontMetrics>,
    ) -> Self {
        Self {
            reader: Reader::new(bytes),
            operands: Vec::new(),
            segments: Vec::new(),
            state: TextState::default(),
            state_stack: Vec::new(),
            marked_roles: Vec::new(),
            actual_text_stack: Vec::new(),
            font_cmaps,
            font_metrics,
        }
    }

    fn parse(&mut self) {
        while let Some(token) = self.reader.next_token() {
            self.apply_token(token);
        }
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
            Token::ActualText(bytes) => {
                let text = self.decode_text(bytes);
                self.operands.push(Operand::ActualText(text));
            }
            Token::Word(word) => self.apply_word(&word),
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
            "TJ" => self.push_latest_array(),
            "Tj" => self.push_latest_text(),
            "'" => {
                self.state.next_line();
                self.push_latest_text();
            }
            "\"" => {
                self.apply_quote_spacing();
                self.state.next_line();
                self.push_latest_text();
            }
            _ => {}
        }
        self.operands.clear();
    }

    fn push_latest_text(&mut self) {
        if let Some(text) = self.latest_text() {
            self.push_decoded_segment(&text);
        }
    }

    fn push_latest_array(&mut self) {
        if let Some(items) = self.latest_array() {
            self.push_decoded_segment(&joined_text_array(&items));
        }
    }

    fn push_decoded_segment(&mut self, decoded: &DecodedText) {
        let replacement = self.current_actual_text();
        let decoded = replacement.as_ref().unwrap_or(decoded);
        let text = normalize_whitespace(&decoded.text);
        if is_readable_text(&text) && self.state.is_visible_text() {
            let width = self.current_metrics().map(|metrics| {
                metrics.text_width(&decoded.raw, self.state.font_size(), text.chars().count())
            });
            self.segments.push(
                self.state
                    .segment(text, width)
                    .with_role(self.current_role()),
            );
            let segment = self.segments.last().unwrap();
            self.state
                .advance_text(&segment.text, width.unwrap_or(segment.width));
        }
    }

    fn apply_state_operator(&mut self, operator: &str) {
        match operator {
            "BT" => self.state.begin_text_object(),
            "BMC" | "BDC" => self.begin_marked_content(),
            "EMC" => self.end_marked_content(),
            "q" => self.state_stack.push(self.state.clone()),
            "Q" => self.restore_graphics_state(),
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
            "T*" => self.state.next_line(),
            _ => {}
        }
    }

    fn restore_graphics_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
    }

    fn begin_marked_content(&mut self) {
        if let Some(role) = self.latest_name() {
            self.marked_roles.push(role);
            self.actual_text_stack.push(self.latest_actual_text());
        }
    }

    fn end_marked_content(&mut self) {
        self.marked_roles.pop();
        self.actual_text_stack.pop();
    }

    fn apply_font(&mut self) {
        if let Some(size) = self.latest_number() {
            self.state.set_font_size(size);
        }
        if let Some(name) = self.latest_name() {
            self.state.font_name = Some(name);
        }
    }

    fn apply_leading(&mut self) {
        if let Some(leading) = self.latest_number() {
            self.state.set_leading(leading);
        }
    }

    fn apply_character_spacing(&mut self) {
        if let Some(spacing) = self.latest_number() {
            self.state.set_character_spacing(spacing);
        }
    }

    fn apply_word_spacing(&mut self) {
        if let Some(spacing) = self.latest_number() {
            self.state.set_word_spacing(spacing);
        }
    }

    fn apply_horizontal_scaling(&mut self) {
        if let Some(scaling) = self.latest_number() {
            self.state.set_horizontal_scaling(scaling);
        }
    }

    fn apply_text_rise(&mut self) {
        if let Some(rise) = self.latest_number() {
            self.state.set_text_rise(rise);
        }
    }

    fn apply_rendering_mode(&mut self) {
        if let Some(mode) = self.latest_number() {
            self.state.set_rendering_mode(mode as i32);
        }
    }

    fn apply_quote_spacing(&mut self) {
        let values: Vec<f32> = self.operands.iter().filter_map(number_operand).collect();
        if values.len() >= 2 {
            self.state.set_word_spacing(values[values.len() - 2]);
            self.state.set_character_spacing(values[values.len() - 1]);
        }
    }

    fn move_text_position(&mut self, update_leading: bool) {
        let Some((tx, ty)) = self.latest_two_numbers() else {
            return;
        };
        self.state.move_position(tx, ty);
        if update_leading {
            self.state.set_leading(ty);
        }
    }

    fn set_text_matrix(&mut self) {
        if let Some(values) = self.latest_six_numbers() {
            self.state.set_position(values[4], values[5]);
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
            .state
            .font_name
            .as_ref()
            .and_then(|name| self.font_cmaps.get(name))
            .map(|cmap| cmap.decode(&bytes))
            .unwrap_or_else(|| decode_pdf_string(&bytes));
        DecodedText { text, raw: bytes }
    }

    fn current_metrics(&self) -> Option<&FontMetrics> {
        self.state
            .font_name
            .as_ref()
            .and_then(|name| self.font_metrics.get(name))
    }

    fn current_role(&self) -> Option<String> {
        self.marked_roles.last().cloned()
    }

    fn current_actual_text(&self) -> Option<DecodedText> {
        self.actual_text_stack.last().cloned().flatten()
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

fn joined_text_array(items: &[TextArrayItem]) -> DecodedText {
    let mut decoded = DecodedText {
        text: String::new(),
        raw: Vec::new(),
    };
    let mut pending_space = false;

    for item in items {
        match item {
            TextArrayItem::Text(value) => push_array_text(&mut decoded, value, &mut pending_space),
            TextArrayItem::Adjustment(value) => {
                pending_space = pending_space || *value <= -120.0;
            }
        }
    }

    decoded
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
