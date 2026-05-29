use std::collections::HashMap;

use super::super::fonts::FontMetrics;
use super::super::object::PdfReference;
use super::super::struct_tree::{McidMap, McidScope};
use super::lines::estimated_text_width;
use super::operands::DecodedText;
use super::state::TextState;
use super::strings::{is_probable_symbol_noise, is_readable_text, normalize_whitespace};
use super::types::TextSegment;

pub(super) struct SegmentEmitter<'a> {
    segments: Vec<TextSegment>,
    state: TextState,
    state_stack: Vec<TextState>,
    marked_roles: Vec<String>,
    actual_text_stack: Vec<Option<DecodedText>>,
    font_metrics: &'a HashMap<String, FontMetrics>,
    struct_roles: &'a McidMap<String>,
    struct_actual_text: &'a McidMap<String>,
    page_reference: Option<PdfReference>,
}

impl<'a> SegmentEmitter<'a> {
    pub(super) fn new(
        font_metrics: &'a HashMap<String, FontMetrics>,
        struct_roles: &'a McidMap<String>,
        struct_actual_text: &'a McidMap<String>,
        page_reference: Option<PdfReference>,
    ) -> Self {
        Self {
            segments: Vec::new(),
            state: TextState::default(),
            state_stack: Vec::new(),
            marked_roles: Vec::new(),
            actual_text_stack: Vec::new(),
            font_metrics,
            struct_roles,
            struct_actual_text,
            page_reference,
        }
    }

    pub(super) fn into_segments(self) -> Vec<TextSegment> {
        self.segments
    }

    pub(super) fn push_decoded_segment(&mut self, decoded: &DecodedText) {
        self.push_decoded_segment_with_adjustment(decoded, 0.0);
    }

    pub(super) fn push_decoded_segment_with_adjustment(
        &mut self,
        decoded: &DecodedText,
        tj_adjustment: f32,
    ) {
        let replacement = self.current_actual_text();
        let decoded = replacement.as_ref().unwrap_or(decoded);
        let text =
            super::strings::repair_shifted_subset_words(&normalize_whitespace(&decoded.text));
        if !is_readable_text(&text)
            || is_probable_symbol_noise(&text)
            || !self.state.is_visible_text()
        {
            return;
        }

        let font_family = self
            .current_metrics()
            .and_then(|metrics| metrics.css_family())
            .map(str::to_string);
        let font_weight = self
            .current_metrics()
            .and_then(|metrics| metrics.is_bold().then_some(700));
        let font_style = self
            .current_metrics()
            .and_then(|metrics| metrics.is_italic().then(|| "italic".to_string()));
        let width = self.current_metrics().map(|metrics| {
            metrics.text_width(&decoded.raw, self.state.font_size(), text.chars().count())
        });
        let advance = width.unwrap_or_else(|| estimated_text_width(&text, self.state.font_size()));
        let visual_width = (advance - tj_adjustment / 1000.0 * self.state.font_size())
            .max(self.state.font_size() * 0.1);
        let segment = self
            .state
            .segment(text, Some(visual_width))
            .with_role(self.current_role())
            .with_font_style(font_family, font_weight, font_style);
        self.state.advance_text(&segment.text, advance);
        if tj_adjustment.abs() > f32::EPSILON {
            self.state.apply_tj_adjustment(tj_adjustment);
        }
        self.segments.push(segment);
    }

    pub(super) fn begin_text_object(&mut self) {
        self.state.begin_text_object();
    }

    pub(super) fn next_line(&mut self) {
        self.state.next_line();
    }

    pub(super) fn apply_tj_adjustment(&mut self, value: f32) {
        self.state.apply_tj_adjustment(value);
    }

    pub(super) fn save_graphics_state(&mut self) {
        self.state_stack.push(self.state.clone());
    }

    pub(super) fn restore_graphics_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.state = state;
        }
    }

    pub(super) fn begin_marked_content(&mut self, role: String, actual_text: Option<DecodedText>) {
        self.marked_roles.push(role);
        self.actual_text_stack.push(actual_text);
    }

    pub(super) fn end_marked_content(&mut self) {
        self.marked_roles.pop();
        self.actual_text_stack.pop();
    }

    pub(super) fn set_font_size(&mut self, size: f32) {
        self.state.set_font_size(size);
    }

    pub(super) fn set_font_name(&mut self, name: String) {
        self.state.font_name = Some(name);
    }

    pub(super) fn set_leading(&mut self, leading: f32) {
        self.state.set_leading(leading);
    }

    pub(super) fn set_character_spacing(&mut self, spacing: f32) {
        self.state.set_character_spacing(spacing);
    }

    pub(super) fn set_word_spacing(&mut self, spacing: f32) {
        self.state.set_word_spacing(spacing);
    }

    pub(super) fn set_horizontal_scaling(&mut self, scaling: f32) {
        self.state.set_horizontal_scaling(scaling);
    }

    pub(super) fn set_text_rise(&mut self, rise: f32) {
        self.state.set_text_rise(rise);
    }

    pub(super) fn set_rendering_mode(&mut self, mode: i32) {
        self.state.set_rendering_mode(mode);
    }

    pub(super) fn set_fill_gray(&mut self, value: f32) {
        self.state.set_fill_gray(value);
    }

    pub(super) fn set_fill_rgb(&mut self, red: f32, green: f32, blue: f32) {
        self.state.set_fill_rgb(red, green, blue);
    }

    pub(super) fn set_fill_cmyk(&mut self, cyan: f32, magenta: f32, yellow: f32, black: f32) {
        self.state.set_fill_cmyk(cyan, magenta, yellow, black);
    }

    pub(super) fn move_position(&mut self, tx: f32, ty: f32) {
        self.state.move_position(tx, ty);
    }

    pub(super) fn set_text_matrix(&mut self, values: [f32; 6]) {
        self.state.set_text_matrix(values);
    }

    pub(super) fn concat_matrix(&mut self, values: [f32; 6]) {
        self.state.concat_matrix(values);
    }

    pub(super) fn font_name(&self) -> Option<&str> {
        self.state.font_name.as_deref()
    }

    pub(super) fn struct_role(&self, mcid: u32) -> Option<String> {
        scoped_lookup(self.struct_roles, self.page_reference, mcid).cloned()
    }

    pub(super) fn struct_actual_text(&self, mcid: u32) -> Option<DecodedText> {
        scoped_lookup(self.struct_actual_text, self.page_reference, mcid).map(|text| DecodedText {
            text: text.clone(),
            raw: Vec::new(),
        })
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
}

fn scoped_lookup<T>(
    map: &McidMap<T>,
    page_reference: Option<PdfReference>,
    mcid: u32,
) -> Option<&T> {
    page_reference
        .and_then(|page| map.get(&McidScope::new(Some(page), mcid)))
        .or_else(|| map.get(&McidScope::unscoped(mcid)))
}
