use std::collections::HashMap;

use super::super::cmap::CMap;
use super::super::fonts::FontMetrics;
use super::emitter::SegmentEmitter;
use super::operands::{push_array_text, DecodedText, Operand, OperandStack, TextArrayItem};
use super::reader::{ArrayToken, MarkedProps, Reader, Token};
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

struct TextParser<'a> {
    reader: Reader<'a>,
    operands: OperandStack,
    font_cmaps: &'a HashMap<String, CMap>,
    emitter: SegmentEmitter<'a>,
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
            operands: OperandStack::default(),
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
        if let Some(text) = self.operands.latest_text() {
            self.emitter.push_decoded_segment(&text);
        }
    }

    fn push_latest_array_split(&mut self) {
        let Some(items) = self.operands.latest_array() else {
            return;
        };
        let mut current = DecodedText {
            text: String::new(),
            raw: Vec::new(),
        };
        let mut pending_space = false;
        let mut pending_adjustment = 0.0;

        for item in &items {
            match item {
                TextArrayItem::Text(value) => {
                    push_array_text(&mut current, value, &mut pending_space);
                }
                TextArrayItem::Adjustment(value) => {
                    if *value <= POSITIONAL_JUMP_THRESHOLD {
                        if !current.text.is_empty() {
                            self.emitter
                                .push_decoded_segment_with_adjustment(&current, pending_adjustment);
                            current = DecodedText {
                                text: String::new(),
                                raw: Vec::new(),
                            };
                            pending_adjustment = 0.0;
                        }
                        self.emitter.apply_tj_adjustment(*value);
                        pending_space = false;
                    } else {
                        pending_space = pending_space || *value <= -120.0;
                        pending_adjustment += *value;
                    }
                }
            }
        }

        if !current.text.is_empty() {
            self.emitter
                .push_decoded_segment_with_adjustment(&current, pending_adjustment);
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
            "g" => self.apply_fill_gray(),
            "rg" => self.apply_fill_rgb(),
            "k" => self.apply_fill_cmyk(),
            "sc" | "scn" => self.apply_fill_color_components(),
            "Td" => self.move_text_position(false),
            "TD" => self.move_text_position(true),
            "Tm" => self.set_text_matrix(),
            "T*" => self.emitter.next_line(),
            _ => {}
        }
    }

    fn begin_marked_content(&mut self) {
        let Some(tag) = self.operands.latest_name() else {
            return;
        };
        let role = self
            .operands
            .latest_mcid()
            .and_then(|mcid| self.emitter.struct_role(mcid))
            .unwrap_or(tag);
        self.emitter
            .begin_marked_content(role, self.operands.latest_actual_text());
    }

    fn apply_font(&mut self) {
        if let Some(size) = self.operands.latest_number() {
            self.emitter.set_font_size(size);
        }
        if let Some(name) = self.operands.latest_name() {
            self.emitter.set_font_name(name);
        }
    }

    fn apply_leading(&mut self) {
        if let Some(leading) = self.operands.latest_number() {
            self.emitter.set_leading(leading);
        }
    }

    fn apply_character_spacing(&mut self) {
        if let Some(spacing) = self.operands.latest_number() {
            self.emitter.set_character_spacing(spacing);
        }
    }

    fn apply_word_spacing(&mut self) {
        if let Some(spacing) = self.operands.latest_number() {
            self.emitter.set_word_spacing(spacing);
        }
    }

    fn apply_horizontal_scaling(&mut self) {
        if let Some(scaling) = self.operands.latest_number() {
            self.emitter.set_horizontal_scaling(scaling);
        }
    }

    fn apply_text_rise(&mut self) {
        if let Some(rise) = self.operands.latest_number() {
            self.emitter.set_text_rise(rise);
        }
    }

    fn apply_rendering_mode(&mut self) {
        if let Some(mode) = self.operands.latest_number() {
            self.emitter.set_rendering_mode(mode as i32);
        }
    }

    fn apply_fill_gray(&mut self) {
        if let Some(value) = self.operands.latest_number() {
            self.emitter.set_fill_gray(value);
        }
    }

    fn apply_fill_rgb(&mut self) {
        let values = self.operands.numbers();
        if values.len() >= 3 {
            self.emitter.set_fill_rgb(
                values[values.len() - 3],
                values[values.len() - 2],
                values[values.len() - 1],
            );
        }
    }

    fn apply_fill_color_components(&mut self) {
        let values = self.operands.numbers();
        if values.len() >= 3 {
            self.emitter.set_fill_rgb(
                values[values.len() - 3],
                values[values.len() - 2],
                values[values.len() - 1],
            );
        } else if let Some(value) = values.last() {
            self.emitter.set_fill_gray(*value);
        }
    }

    fn apply_fill_cmyk(&mut self) {
        let values = self.operands.numbers();
        if values.len() >= 4 {
            self.emitter.set_fill_cmyk(
                values[values.len() - 4],
                values[values.len() - 3],
                values[values.len() - 2],
                values[values.len() - 1],
            );
        }
    }

    fn apply_quote_spacing(&mut self) {
        let values = self.operands.numbers();
        if values.len() >= 2 {
            self.emitter.set_word_spacing(values[values.len() - 2]);
            self.emitter.set_character_spacing(values[values.len() - 1]);
        }
    }

    fn move_text_position(&mut self, update_leading: bool) {
        let Some((tx, ty)) = self.operands.latest_two_numbers() else {
            return;
        };
        self.emitter.move_position(tx, ty);
        if update_leading {
            self.emitter.set_leading(ty);
        }
    }

    fn set_text_matrix(&mut self) {
        if let Some(values) = self.operands.latest_six_numbers() {
            self.emitter.set_text_matrix(values);
        }
    }

    fn concat_matrix(&mut self) {
        if let Some(values) = self.operands.latest_six_numbers() {
            self.emitter.concat_matrix(values);
        }
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
