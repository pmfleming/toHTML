use super::TextParser;

impl<'a> TextParser<'a> {
    pub(super) fn apply_state_operator(&mut self, operator: &str) {
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
        let actual_text = self.operands.latest_actual_text().or_else(|| {
            self.operands
                .latest_mcid()
                .and_then(|mcid| self.emitter.struct_actual_text(mcid))
        });
        self.emitter.begin_marked_content(role, actual_text);
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
}
