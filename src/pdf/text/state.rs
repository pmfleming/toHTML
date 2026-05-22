use super::lines::estimated_text_width;
use super::types::TextSegment;

#[derive(Debug, Clone)]
pub(super) struct TextState {
    x: f32,
    y: f32,
    line_x: f32,
    line_y: f32,
    font_size: f32,
    leading: f32,
    character_spacing: f32,
    word_spacing: f32,
    horizontal_scaling: f32,
    text_rise: f32,
    rendering_mode: i32,
    pub font_name: Option<String>,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            line_x: 0.0,
            line_y: 0.0,
            font_size: 12.0,
            leading: 12.0,
            character_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            text_rise: 0.0,
            rendering_mode: 0,
            font_name: None,
        }
    }
}

impl TextState {
    pub fn begin_text_object(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.line_x = 0.0;
        self.line_y = 0.0;
    }

    pub fn segment(&self, text: String, width: Option<f32>) -> TextSegment {
        let width = width.unwrap_or_else(|| estimated_text_width(&text, self.font_size));
        TextSegment::new(text, self.x, self.y + self.text_rise, self.font_size, width)
    }

    pub fn text_advance(&self, text: &str, width: f32) -> f32 {
        let spacing = self.spacing_advance(text);
        (width + spacing) * (self.horizontal_scaling / 100.0).max(0.01)
    }

    pub fn advance_text(&mut self, text: &str, width: f32) {
        self.x += self.text_advance(text, width);
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.max(1.0);
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_leading(&mut self, leading: f32) {
        self.leading = leading.abs().max(1.0);
    }

    pub fn set_character_spacing(&mut self, spacing: f32) {
        self.character_spacing = spacing;
    }

    pub fn set_word_spacing(&mut self, spacing: f32) {
        self.word_spacing = spacing;
    }

    pub fn set_horizontal_scaling(&mut self, scaling: f32) {
        self.horizontal_scaling = scaling.max(1.0);
    }

    pub fn set_text_rise(&mut self, rise: f32) {
        self.text_rise = rise;
    }

    pub fn set_rendering_mode(&mut self, mode: i32) {
        self.rendering_mode = mode;
    }

    pub fn is_visible_text(&self) -> bool {
        !matches!(self.rendering_mode, 3 | 7)
    }

    pub fn move_position(&mut self, tx: f32, ty: f32) {
        self.x += tx;
        self.y += ty;
        self.line_x = self.x;
        self.line_y = self.y;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.line_x = x;
        self.line_y = y;
    }

    pub fn next_line(&mut self) {
        self.line_y -= self.leading;
        self.x = self.line_x;
        self.y = self.line_y;
    }

    fn spacing_advance(&self, text: &str) -> f32 {
        let chars = text.chars().count().saturating_sub(1) as f32;
        let spaces = text.chars().filter(|ch| *ch == ' ').count() as f32;
        chars * self.character_spacing + spaces * self.word_spacing
    }
}
