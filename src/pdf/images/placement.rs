use std::collections::{HashMap, HashSet};

use crate::ConversionWarning;

use super::super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};
use super::super::streams;
use super::tokens::{last_name, last_numbers, tokenize, Operand, Token};

pub(super) struct ImagePlacement {
    pub(super) reference: PdfReference,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

pub(super) fn image_placements(
    source: &[u8],
    objects: &PdfObjects,
    content_streams: &[Vec<u8>],
    resources: &HashMap<String, PdfReference>,
    warnings: &mut Vec<ConversionWarning>,
) -> Vec<ImagePlacement> {
    let mut placements = Vec::new();
    let mut visited_forms = HashSet::new();
    for stream in content_streams {
        collect_image_placements(
            source,
            objects,
            stream,
            resources,
            Matrix::identity(),
            Clip::Unbounded,
            &mut visited_forms,
            warnings,
            &mut placements,
        );
    }
    placements
}

fn collect_image_placements(
    source: &[u8],
    objects: &PdfObjects,
    content_stream: &[u8],
    resources: &HashMap<String, PdfReference>,
    initial_ctm: Matrix,
    initial_clip: Clip,
    visited_forms: &mut HashSet<PdfReference>,
    warnings: &mut Vec<ConversionWarning>,
    placements: &mut Vec<ImagePlacement>,
) {
    let mut state = GraphicsState {
        ctm: initial_ctm,
        clip: initial_clip,
    };
    let mut stack = Vec::new();
    let mut operands = Vec::new();
    let mut pending_rects = Vec::new();

    for token in tokenize(content_stream) {
        match token {
            Token::Number(value) => operands.push(Operand::Number(value)),
            Token::Name(name) => operands.push(Operand::Name(name)),
            Token::Operator(operator) => {
                match operator.as_str() {
                    "q" => stack.push(state),
                    "Q" => state = stack.pop().unwrap_or_default(),
                    "cm" => {
                        if let Some(values) = last_numbers::<6>(&operands) {
                            state.ctm = state.ctm.multiply(Matrix {
                                a: values[0],
                                b: values[1],
                                c: values[2],
                                d: values[3],
                                e: values[4],
                                f: values[5],
                            });
                        }
                    }
                    "re" => {
                        if let Some(values) = last_numbers::<4>(&operands) {
                            pending_rects.push(
                                state
                                    .ctm
                                    .transform_rect(values[0], values[1], values[2], values[3]),
                            );
                        }
                    }
                    "W" | "W*" => {
                        if let Some(rect) = bounding_rect(&pending_rects) {
                            state.clip = state.clip.intersect(rect);
                        }
                    }
                    "n" => pending_rects.clear(),
                    "Do" => {
                        if let Some(name) = last_name(&operands) {
                            if let Some(reference) = resources.get(name) {
                                collect_xobject_image_placement(
                                    source,
                                    objects,
                                    *reference,
                                    state.ctm,
                                    state.clip,
                                    visited_forms,
                                    warnings,
                                    placements,
                                );
                            }
                        }
                    }
                    _ => {}
                }
                operands.clear();
            }
        }
    }
}

fn collect_xobject_image_placement(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
    ctm: Matrix,
    clip: Clip,
    visited_forms: &mut HashSet<PdfReference>,
    warnings: &mut Vec<ConversionWarning>,
    placements: &mut Vec<ImagePlacement>,
) {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return;
    };
    let Some(dictionary) = object.dictionary() else {
        return;
    };

    match dictionary.name("Subtype") {
        Some("Image") => {
            let rect = ctm.transform_unit_square();
            let Some(visible) = clip.visible_rect(rect) else {
                return;
            };
            if visible.width < 0.5 || visible.height < 0.5 {
                return;
            }
            placements.push(ImagePlacement {
                reference,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            });
        }
        Some("Form") if visited_forms.insert(reference) => {
            let Some(stream) = object.stream.as_deref() else {
                visited_forms.remove(&reference);
                return;
            };
            let decoded = match streams::decoded_stream_data(dictionary, stream) {
                Ok(stream) => stream,
                Err(error) => {
                    warnings.push(ConversionWarning {
                        message: format!(
                            "Skipped form XObject {} while extracting images: {error}",
                            reference.object
                        ),
                        source: None,
                    });
                    visited_forms.remove(&reference);
                    return;
                }
            };
            let form_resources = xobject_resources(objects, dictionary);
            let form_ctm = ctm.multiply(form_matrix(dictionary));
            collect_image_placements(
                source,
                objects,
                &decoded,
                &form_resources,
                form_ctm,
                // A form's current transformation matrix is already included in
                // both the form content and the active clip rectangle.
                clip,
                visited_forms,
                warnings,
                placements,
            );
            visited_forms.remove(&reference);
        }
        _ => {}
    }
}

fn form_matrix(dictionary: &PdfDictionary) -> Matrix {
    let Some(values) = dictionary.array("Matrix") else {
        return Matrix::identity();
    };
    let [a, b, c, d, e, f] = values else {
        return Matrix::identity();
    };
    let Some(a) = pdf_number(a) else {
        return Matrix::identity();
    };
    let Some(b) = pdf_number(b) else {
        return Matrix::identity();
    };
    let Some(c) = pdf_number(c) else {
        return Matrix::identity();
    };
    let Some(d) = pdf_number(d) else {
        return Matrix::identity();
    };
    let Some(e) = pdf_number(e) else {
        return Matrix::identity();
    };
    let Some(f) = pdf_number(f) else {
        return Matrix::identity();
    };
    Matrix { a, b, c, d, e, f }
}

fn xobject_resources(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
) -> HashMap<String, PdfReference> {
    let Some(resources) = dictionary_value(objects, dictionary.get("Resources")) else {
        return HashMap::new();
    };
    let Some(xobjects) = dictionary_value(objects, resources.get("XObject")) else {
        return HashMap::new();
    };
    xobjects
        .iter()
        .filter_map(|(name, value)| match value {
            PdfValue::Reference(reference) => Some((name.clone(), *reference)),
            _ => None,
        })
        .collect()
}

fn dictionary_value<'a>(
    objects: &'a PdfObjects,
    value: Option<&'a PdfValue>,
) -> Option<&'a PdfDictionary> {
    match value? {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary()),
        _ => None,
    }
}

fn pdf_number(value: &PdfValue) -> Option<f32> {
    match value {
        PdfValue::Integer(value) => Some(*value as f32),
        PdfValue::Real(value) => Some(*value),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
enum Clip {
    Unbounded,
    Rect(ImageRect),
    Empty,
}

impl Clip {
    fn intersect(self, rect: ImageRect) -> Self {
        match self {
            Self::Unbounded => Self::Rect(rect),
            Self::Rect(active) => active
                .intersection(rect)
                .map(Self::Rect)
                .unwrap_or(Self::Empty),
            Self::Empty => Self::Empty,
        }
    }

    fn visible_rect(self, rect: ImageRect) -> Option<ImageRect> {
        match self {
            Self::Unbounded => Some(rect),
            Self::Rect(active) => active.intersection(rect),
            Self::Empty => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GraphicsState {
    ctm: Matrix,
    clip: Clip,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::identity(),
            clip: Clip::Unbounded,
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

    fn transform_unit_square(self) -> ImageRect {
        self.transform_rect(0.0, 0.0, 1.0, 1.0)
    }

    fn transform_rect(self, x: f32, y: f32, width: f32, height: f32) -> ImageRect {
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
        ImageRect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct ImageRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ImageRect {
    fn intersection(self, other: Self) -> Option<Self> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        (x2 > x1 && y2 > y1).then_some(Self {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        })
    }
}

fn bounding_rect(rects: &[ImageRect]) -> Option<ImageRect> {
    let first = *rects.first()?;
    let (mut min_x, mut min_y) = (first.x, first.y);
    let (mut max_x, mut max_y) = (first.x + first.width, first.y + first.height);
    for rect in &rects[1..] {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.width);
        max_y = max_y.max(rect.y + rect.height);
    }
    Some(ImageRect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}
