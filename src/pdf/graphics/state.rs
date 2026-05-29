use super::paths::Path;
use super::RectShape;

#[derive(Debug, Clone)]
pub(super) struct GraphicsState {
    pub(super) ctm: Matrix,
    pub(super) fill: Option<String>,
    pub(super) stroke: Option<String>,
    pub(super) line_width: f32,
    pub(super) dash_array: Vec<f32>,
    pub(super) clip_path: Option<Path>,
    pub(super) clip_rects: Vec<RectShape>,
    pub(super) clip_fill: Option<String>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::identity(),
            fill: Some("#000000".to_string()),
            stroke: Some("#000000".to_string()),
            line_width: 1.0,
            dash_array: Vec::new(),
            clip_path: None,
            clip_rects: Vec::new(),
            clip_fill: None,
        }
    }
}

impl GraphicsState {
    pub(super) fn transformed_line_width(&self) -> f32 {
        self.line_width * self.ctm.scale_x().max(self.ctm.scale_y()).max(0.01)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Matrix {
    pub(super) a: f32,
    pub(super) b: f32,
    pub(super) c: f32,
    pub(super) d: f32,
    pub(super) e: f32,
    pub(super) f: f32,
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

    pub(super) fn multiply(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    pub(super) fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    fn scale_x(self) -> f32 {
        (self.a.powi(2) + self.b.powi(2)).sqrt().abs()
    }

    fn scale_y(self) -> f32 {
        (self.c.powi(2) + self.d.powi(2)).sqrt().abs()
    }

    pub(super) fn transform_rect(self, x: f32, y: f32, width: f32, height: f32) -> RectShape {
        let points = [
            self.transform_point(x, y),
            self.transform_point(x + width, y),
            self.transform_point(x, y + height),
            self.transform_point(x + width, y + height),
        ];
        let min_x = points
            .iter()
            .map(|point| point.0)
            .fold(f32::INFINITY, f32::min);
        let max_x = points
            .iter()
            .map(|point| point.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = points
            .iter()
            .map(|point| point.1)
            .fold(f32::INFINITY, f32::min);
        let max_y = points
            .iter()
            .map(|point| point.1)
            .fold(f32::NEG_INFINITY, f32::max);
        RectShape {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
            fill: None,
            stroke: None,
        }
    }
}

pub(super) fn last_operands<const N: usize>(operands: &[f32]) -> [f32; N] {
    let start = operands.len().saturating_sub(N);
    let mut values = [0.0; N];
    values.copy_from_slice(&operands[start..start + N]);
    values
}

pub(super) fn gray(value: f32) -> String {
    let channel = color_channel(value);
    format!("#{channel:02x}{channel:02x}{channel:02x}")
}

pub(super) fn rgb(values: [f32; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        color_channel(values[0]),
        color_channel(values[1]),
        color_channel(values[2])
    )
}

fn color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
