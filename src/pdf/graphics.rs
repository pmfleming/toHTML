#[derive(Debug, Clone, PartialEq)]
pub(super) struct RectShape {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub fill: Option<String>,
    pub stroke: Option<String>,
}

pub(super) fn extract_rectangles(stream: &[u8]) -> Vec<RectShape> {
    let tokens = tokenize(stream);
    let mut state = GraphicsState::default();
    let mut stack = Vec::new();
    let mut operands = Vec::new();
    let mut pending_rects = Vec::new();
    let mut shapes = Vec::new();

    for token in tokens {
        match token {
            Token::Number(value) => operands.push(value),
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
                    "re" if operands.len() >= 4 => {
                        let values = last_operands::<4>(&operands);
                        pending_rects.push(
                            state
                                .ctm
                                .transform_rect(values[0], values[1], values[2], values[3]),
                        );
                    }
                    "f" | "F" | "f*" => {
                        push_rectangles(&mut shapes, &pending_rects, state.fill.clone(), None);
                        pending_rects.clear();
                    }
                    "S" | "s" => {
                        push_rectangles(&mut shapes, &pending_rects, None, state.stroke.clone());
                        pending_rects.clear();
                    }
                    "B" | "B*" | "b" | "b*" => {
                        push_rectangles(
                            &mut shapes,
                            &pending_rects,
                            state.fill.clone(),
                            state.stroke.clone(),
                        );
                        pending_rects.clear();
                    }
                    "n" => pending_rects.clear(),
                    _ => {}
                }
                operands.clear();
            }
        }
    }

    shapes
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: Matrix,
    fill: Option<String>,
    stroke: Option<String>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::identity(),
            fill: Some("#000000".to_string()),
            stroke: Some("#000000".to_string()),
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

    fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
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

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f32),
    Operator(String),
}

fn tokenize(bytes: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        skip_ignored(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }
        match bytes[index] {
            b'(' => skip_literal_string(bytes, &mut index),
            b'<' if bytes.get(index + 1) != Some(&b'<') => skip_hex_string(bytes, &mut index),
            b'[' | b']' | b'<' | b'>' | b'/' => skip_delimited_token(bytes, &mut index),
            _ => {
                let word = read_word(bytes, &mut index);
                if let Ok(value) = word.parse::<f32>() {
                    tokens.push(Token::Number(value));
                } else if !word.is_empty() {
                    tokens.push(Token::Operator(word));
                }
            }
        }
    }
    tokens
}

fn push_rectangles(
    shapes: &mut Vec<RectShape>,
    rectangles: &[RectShape],
    fill: Option<String>,
    stroke: Option<String>,
) {
    for rectangle in rectangles {
        if rectangle.width.abs() < 0.25 || rectangle.height.abs() < 0.25 {
            continue;
        }
        let mut shape = rectangle.clone();
        shape.fill = fill.clone();
        shape.stroke = stroke.clone();
        shapes.push(shape);
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

fn skip_ignored(bytes: &[u8], index: &mut usize) {
    loop {
        while bytes.get(*index).is_some_and(u8::is_ascii_whitespace) {
            *index += 1;
        }
        if bytes.get(*index) != Some(&b'%') {
            break;
        }
        while *index < bytes.len() && !matches!(bytes[*index], b'\r' | b'\n') {
            *index += 1;
        }
    }
}

fn skip_literal_string(bytes: &[u8], index: &mut usize) {
    *index += 1;
    let mut depth = 1;
    while *index < bytes.len() && depth > 0 {
        match bytes[*index] {
            b'\\' => *index = (*index + 2).min(bytes.len()),
            b'(' => {
                depth += 1;
                *index += 1;
            }
            b')' => {
                depth -= 1;
                *index += 1;
            }
            _ => *index += 1,
        }
    }
}

fn skip_hex_string(bytes: &[u8], index: &mut usize) {
    *index += 1;
    while *index < bytes.len() && bytes[*index] != b'>' {
        *index += 1;
    }
    if *index < bytes.len() {
        *index += 1;
    }
}

fn skip_delimited_token(bytes: &[u8], index: &mut usize) {
    *index += 1;
    while *index < bytes.len()
        && !bytes[*index].is_ascii_whitespace()
        && !matches!(
            bytes[*index],
            b'[' | b']' | b'<' | b'>' | b'(' | b')' | b'/'
        )
    {
        *index += 1;
    }
}

fn read_word(bytes: &[u8], index: &mut usize) -> String {
    let start = *index;
    while *index < bytes.len()
        && !bytes[*index].is_ascii_whitespace()
        && !matches!(
            bytes[*index],
            b'[' | b']' | b'<' | b'>' | b'(' | b')' | b'/'
        )
    {
        *index += 1;
    }
    String::from_utf8_lossy(&bytes[start..*index]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_filled_and_stroked_rectangles() {
        let shapes = extract_rectangles(b"0.9 g 10 20 100 30 re f 0 G 10 20 100 30 re S");

        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].fill.as_deref(), Some("#e6e6e6"));
        assert_eq!(shapes[1].stroke.as_deref(), Some("#000000"));
        assert_eq!(shapes[0].x, 10.0);
        assert_eq!(shapes[0].height, 30.0);
    }

    #[test]
    fn applies_matrix_to_rectangles() {
        let shapes = extract_rectangles(b"2 0 0 2 5 7 cm 10 20 100 30 re S");

        assert_eq!(shapes[0].x, 25.0);
        assert_eq!(shapes[0].y, 47.0);
        assert_eq!(shapes[0].width, 200.0);
        assert_eq!(shapes[0].height, 60.0);
    }
}
