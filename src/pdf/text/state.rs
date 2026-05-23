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
    ctm: Matrix,
    text_rotation: f32,
    text_scale_x: f32,
    text_scale_y: f32,
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
            ctm: Matrix::identity(),
            text_rotation: 0.0,
            text_scale_x: 1.0,
            text_scale_y: 1.0,
            font_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Matrix {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Matrix {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    fn transform_vector(self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y, self.b * x + self.d * y)
    }

    fn rotation_degrees(self) -> f32 {
        self.b.atan2(self.a).to_degrees()
    }

    fn scale_x(self) -> f32 {
        self.a.hypot(self.b).max(0.01)
    }

    fn scale_y(self) -> f32 {
        self.c.hypot(self.d).max(0.01)
    }
}

impl TextState {
    pub fn begin_text_object(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.line_x = 0.0;
        self.line_y = 0.0;
        self.text_scale_x = self.ctm.scale_x();
        self.text_scale_y = self.ctm.scale_y();
    }

    pub fn segment(&self, text: String, width: Option<f32>) -> TextSegment {
        let width = width.unwrap_or_else(|| estimated_text_width(&text, self.font_size));
        TextSegment::new(
            text,
            self.x,
            self.y + self.text_rise * self.text_scale_y,
            self.font_size * self.text_scale_y,
            width * self.text_scale_x,
        )
        .with_rotation(self.text_rotation)
    }

    pub fn text_advance(&self, text: &str, width: f32) -> f32 {
        let spacing = self.spacing_advance(text);
        (width + spacing) * (self.horizontal_scaling / 100.0).max(0.01) * self.text_scale_x
    }

    pub fn advance_text(&mut self, text: &str, width: f32) {
        self.x += self.text_advance(text, width);
    }

    pub fn apply_tj_adjustment(&mut self, adjustment: f32) {
        // TJ adjustments are expressed in 1/1000 em; negative values move x forward.
        let horizontal = self.horizontal_scaling / 100.0;
        self.x += -adjustment / 1000.0 * self.font_size * horizontal.max(0.01) * self.text_scale_x;
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
        let (tx, ty) = self.ctm.transform_vector(tx, ty);
        self.x += tx;
        self.y += ty;
        self.line_x = self.x;
        self.line_y = self.y;
    }

    pub fn set_text_matrix(&mut self, values: [f32; 6]) {
        let text_matrix = Matrix {
            a: values[0],
            b: values[1],
            c: values[2],
            d: values[3],
            e: values[4],
            f: values[5],
        };
        let combined = self.ctm.multiply(text_matrix);
        self.x = combined.e;
        self.y = combined.f;
        self.line_x = combined.e;
        self.line_y = combined.f;
        self.text_rotation = combined.rotation_degrees();
        self.text_scale_x = combined.scale_x();
        self.text_scale_y = combined.scale_y();
    }

    pub fn concat_matrix(&mut self, values: [f32; 6]) {
        let matrix = Matrix {
            a: values[0],
            b: values[1],
            c: values[2],
            d: values[3],
            e: values[4],
            f: values[5],
        };
        self.ctm = self.ctm.multiply(matrix);
        self.text_scale_x = self.ctm.scale_x();
        self.text_scale_y = self.ctm.scale_y();
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
