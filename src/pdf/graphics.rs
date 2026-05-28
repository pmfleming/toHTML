mod paths;
#[cfg(test)]
mod tests;
mod tokens;

use std::collections::HashMap;

use paths::{
    dash_array, push_filled_path_rectangles, push_rectangles, push_stroked_path_lines,
    push_vector_path, Path,
};
use tokens::{tokenize, Token};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RectShape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub fill: Option<String>,
    pub stroke: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VectorPath {
    pub commands: Vec<PathCommand>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub stroke_dasharray: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

pub(super) fn extract_rectangles(stream: &[u8]) -> Vec<RectShape> {
    extract_graphics(stream, &HashMap::new()).rectangles
}

pub(super) fn extract_paths(stream: &[u8]) -> Vec<VectorPath> {
    extract_graphics(stream, &HashMap::new()).paths
}

pub(super) fn extract_paths_with_shading_fills(
    stream: &[u8],
    shading_fills: &HashMap<String, String>,
) -> Vec<VectorPath> {
    extract_graphics(stream, shading_fills).paths
}

#[derive(Debug, Default)]
struct GraphicsExtraction {
    rectangles: Vec<RectShape>,
    paths: Vec<VectorPath>,
}

fn extract_graphics(stream: &[u8], shading_fills: &HashMap<String, String>) -> GraphicsExtraction {
    let tokens = tokenize(stream);
    let mut state = GraphicsState::default();
    let mut stack = Vec::new();
    let mut operands = Vec::new();
    let mut name_operands = Vec::new();
    let mut pending_rects = Vec::new();
    let mut path = Path::default();
    let mut shapes = Vec::new();
    let mut vector_paths = Vec::new();
    let mut wide_shading_fill = None;

    for token in tokens {
        match token {
            Token::Name(name) => name_operands.push(name),
            Token::Number(value) => operands.push(value),
            Token::NumberArray(values) => operands.extend(values),
            Token::Operator(operator) => {
                match operator.as_str() {
                    "q" => stack.push(state.clone()),
                    "Q" => state = stack.pop().unwrap_or_default(),
                    "cm" if operands.len() >= 6 => {
                        let values = last_operands::<6>(&operands);
                        state.ctm = state.ctm.multiply(Matrix {
                            a: values[0],
                            b: values[1],
                            c: values[2],
                            d: values[3],
                            e: values[4],
                            f: values[5],
                        });
                    }
                    "g" if !operands.is_empty() => state.fill = Some(gray(operands[0])),
                    "G" if !operands.is_empty() => state.stroke = Some(gray(operands[0])),
                    "rg" if operands.len() >= 3 => {
                        let values = last_operands::<3>(&operands);
                        state.fill = Some(rgb(values));
                    }
                    "RG" if operands.len() >= 3 => {
                        let values = last_operands::<3>(&operands);
                        state.stroke = Some(rgb(values));
                    }
                    "scn" | "sc" => {
                        if operands.len() >= 3 {
                            let values = last_operands::<3>(&operands);
                            state.fill = Some(rgb(values));
                        } else if !operands.is_empty() {
                            state.fill = Some(gray(operands[operands.len() - 1]));
                        }
                    }
                    "SCN" | "SC" => {
                        if operands.len() >= 3 {
                            let values = last_operands::<3>(&operands);
                            state.stroke = Some(rgb(values));
                        } else if !operands.is_empty() {
                            state.stroke = Some(gray(operands[operands.len() - 1]));
                        }
                    }
                    "w" if !operands.is_empty() => {
                        state.line_width = operands[operands.len() - 1].abs().max(0.25);
                    }
                    "d" if operands.len() >= 2 => {
                        let dash_count = operands.len() - 1;
                        state.dash_array = operands[..dash_count]
                            .iter()
                            .map(|value| value.abs())
                            .filter(|value| *value > 0.0)
                            .collect();
                    }
                    "re" if operands.len() >= 4 => {
                        let values = last_operands::<4>(&operands);
                        pending_rects.push(
                            state
                                .ctm
                                .transform_rect(values[0], values[1], values[2], values[3]),
                        );
                    }
                    "W" | "W*" => {
                        if path.has_drawing_commands() {
                            state.clip_path = Some(path.clone());
                            state.clip_fill = state.fill.clone();
                        } else if !pending_rects.is_empty() {
                            state.clip_rects = pending_rects.clone();
                            state.clip_fill = state.fill.clone();
                        }
                    }
                    "m" if operands.len() >= 2 => {
                        let values = last_operands::<2>(&operands);
                        path.move_to(state.ctm.transform_point(values[0], values[1]));
                    }
                    "l" if operands.len() >= 2 => {
                        let values = last_operands::<2>(&operands);
                        path.line_to(state.ctm.transform_point(values[0], values[1]));
                    }
                    "c" if operands.len() >= 6 => {
                        let values = last_operands::<6>(&operands);
                        path.curve_to(
                            state.ctm.transform_point(values[0], values[1]),
                            state.ctm.transform_point(values[2], values[3]),
                            state.ctm.transform_point(values[4], values[5]),
                        );
                    }
                    "v" if operands.len() >= 4 => {
                        let values = last_operands::<4>(&operands);
                        if let Some(current) = path.current_point() {
                            path.curve_to(
                                current,
                                state.ctm.transform_point(values[0], values[1]),
                                state.ctm.transform_point(values[2], values[3]),
                            );
                        }
                    }
                    "y" if operands.len() >= 4 => {
                        let values = last_operands::<4>(&operands);
                        let end = state.ctm.transform_point(values[2], values[3]);
                        path.curve_to(state.ctm.transform_point(values[0], values[1]), end, end);
                    }
                    "h" => path.close(),
                    "f" | "F" | "f*" => {
                        push_rectangles(&mut shapes, &pending_rects, state.fill.clone(), None);
                        push_filled_path_rectangles(&mut shapes, &path, state.fill.clone());
                        push_vector_path(
                            &mut vector_paths,
                            &path,
                            state.fill.clone(),
                            None,
                            state.transformed_line_width(),
                            None,
                        );
                        pending_rects.clear();
                        path.clear();
                    }
                    "S" | "s" => {
                        if operator == "s" {
                            path.close();
                        }
                        push_rectangles(&mut shapes, &pending_rects, None, state.stroke.clone());
                        if state.dash_array.is_empty() {
                            push_stroked_path_lines(
                                &mut shapes,
                                &path,
                                state.stroke.clone(),
                                state.transformed_line_width(),
                            );
                        }
                        push_vector_path(
                            &mut vector_paths,
                            &path,
                            None,
                            state.stroke.clone(),
                            state.transformed_line_width(),
                            dash_array(&state),
                        );
                        pending_rects.clear();
                        path.clear();
                    }
                    "B" | "B*" | "b" | "b*" => {
                        if operator == "b" || operator == "b*" {
                            path.close();
                        }
                        push_rectangles(
                            &mut shapes,
                            &pending_rects,
                            state.fill.clone(),
                            state.stroke.clone(),
                        );
                        push_filled_path_rectangles(&mut shapes, &path, state.fill.clone());
                        if state.dash_array.is_empty() {
                            push_stroked_path_lines(
                                &mut shapes,
                                &path,
                                state.stroke.clone(),
                                state.transformed_line_width(),
                            );
                        }
                        push_vector_path(
                            &mut vector_paths,
                            &path,
                            state.fill.clone(),
                            state.stroke.clone(),
                            state.transformed_line_width(),
                            dash_array(&state),
                        );
                        pending_rects.clear();
                        path.clear();
                    }
                    "sh" => {
                        let shading_fill = name_operands
                            .last()
                            .and_then(|name| shading_fills.get(name))
                            .cloned();
                        if let Some(clip_path) = &state.clip_path {
                            let fill = shading_fill.or_else(|| {
                                shading_fill_for_clip(
                                    clip_path,
                                    state.clip_fill.clone().or_else(|| state.fill.clone()),
                                    &mut wide_shading_fill,
                                )
                            });
                            push_vector_path(
                                &mut vector_paths,
                                clip_path,
                                fill,
                                None,
                                state.transformed_line_width(),
                                None,
                            );
                        } else if !state.clip_rects.is_empty() {
                            push_rectangles(
                                &mut shapes,
                                &state.clip_rects,
                                shading_fill
                                    .or_else(|| state.clip_fill.clone())
                                    .or_else(|| state.fill.clone()),
                                None,
                            );
                        }
                    }
                    "n" => {
                        pending_rects.clear();
                        path.clear();
                    }
                    _ => {}
                }
                operands.clear();
                name_operands.clear();
            }
        }
    }

    GraphicsExtraction {
        rectangles: shapes,
        paths: vector_paths,
    }
}

fn shading_fill_for_clip(
    path: &Path,
    fill: Option<String>,
    wide_shading_fill: &mut Option<String>,
) -> Option<String> {
    if !is_wide_shading_clip(path) {
        return fill;
    }
    if let Some(existing) = wide_shading_fill.clone() {
        return Some(existing);
    }
    if let Some(fill) = fill.clone() {
        *wide_shading_fill = Some(fill);
    }
    fill
}

fn is_wide_shading_clip(path: &Path) -> bool {
    let Some((width, height)) = path.bounds() else {
        return false;
    };
    width >= 100.0 && height >= 20.0 && width / height.max(1.0) >= 3.0
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: Matrix,
    fill: Option<String>,
    stroke: Option<String>,
    line_width: f32,
    dash_array: Vec<f32>,
    clip_path: Option<Path>,
    clip_rects: Vec<RectShape>,
    clip_fill: Option<String>,
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
    fn transformed_line_width(&self) -> f32 {
        self.line_width * self.ctm.scale_x().max(self.ctm.scale_y()).max(0.01)
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

    fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
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

    fn transform_rect(self, x: f32, y: f32, width: f32, height: f32) -> RectShape {
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

fn last_operands<const N: usize>(operands: &[f32]) -> [f32; N] {
    let start = operands.len().saturating_sub(N);
    let mut values = [0.0; N];
    values.copy_from_slice(&operands[start..start + N]);
    values
}

fn gray(value: f32) -> String {
    let channel = color_channel(value);
    format!("#{channel:02x}{channel:02x}{channel:02x}")
}

fn rgb(values: [f32; 3]) -> String {
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
