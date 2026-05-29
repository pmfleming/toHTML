use super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};
use super::{cmap, fonts, object};
use super::{graphics, streams, text};

mod content;

use content::{last_name, last_numbers, tokenize_content, ContentOperand, ContentToken};

#[derive(Debug, Clone, Copy)]
pub(super) struct FormMatrix {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl FormMatrix {
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

    fn x_scale(self) -> f32 {
        (self.a.mul_add(self.a, self.b * self.b)).sqrt().max(0.01)
    }

    fn y_scale(self) -> f32 {
        (self.c.mul_add(self.c, self.d * self.d)).sqrt().max(0.01)
    }
}

pub(super) fn form_xobject_text_segments(
    objects: &PdfObjects,
    resources: &std::collections::HashMap<String, object::PdfReference>,
    content_streams: &[Vec<u8>],
    font_cmaps: &std::collections::HashMap<String, cmap::CMap>,
    font_metrics: &std::collections::HashMap<String, fonts::FontMetrics>,
    struct_roles: &super::struct_tree::McidMap<String>,
    struct_actual_text: &super::struct_tree::McidMap<String>,
    page_reference: Option<object::PdfReference>,
) -> Vec<text::TextSegment> {
    let mut visited = std::collections::HashSet::new();
    let mut segments = Vec::new();
    for stream in content_streams {
        collect_content_form_text_segments(
            objects,
            stream,
            resources,
            FormMatrix::identity(),
            font_cmaps,
            font_metrics,
            struct_roles,
            struct_actual_text,
            page_reference,
            &mut visited,
            &mut segments,
        );
    }
    segments
}

fn collect_content_form_text_segments(
    objects: &PdfObjects,
    content_stream: &[u8],
    resources: &std::collections::HashMap<String, object::PdfReference>,
    initial_ctm: FormMatrix,
    font_cmaps: &std::collections::HashMap<String, cmap::CMap>,
    font_metrics: &std::collections::HashMap<String, fonts::FontMetrics>,
    struct_roles: &super::struct_tree::McidMap<String>,
    struct_actual_text: &super::struct_tree::McidMap<String>,
    page_reference: Option<object::PdfReference>,
    visited: &mut std::collections::HashSet<object::PdfReference>,
    segments: &mut Vec<text::TextSegment>,
) {
    walk_content_xobjects(content_stream, resources, initial_ctm, |reference, ctm| {
        collect_form_xobject_text_segments(
            objects,
            reference,
            ctm,
            font_cmaps,
            font_metrics,
            struct_roles,
            struct_actual_text,
            page_reference,
            visited,
            segments,
        );
    });
}

fn collect_form_xobject_text_segments(
    objects: &PdfObjects,
    reference: object::PdfReference,
    parent_matrix: FormMatrix,
    font_cmaps: &std::collections::HashMap<String, cmap::CMap>,
    font_metrics: &std::collections::HashMap<String, fonts::FontMetrics>,
    struct_roles: &super::struct_tree::McidMap<String>,
    struct_actual_text: &super::struct_tree::McidMap<String>,
    page_reference: Option<object::PdfReference>,
    visited: &mut std::collections::HashSet<object::PdfReference>,
    segments: &mut Vec<text::TextSegment>,
) {
    if !visited.insert(reference) {
        return;
    }
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        visited.remove(&reference);
        return;
    };
    let Some(dictionary) = object.dictionary() else {
        visited.remove(&reference);
        return;
    };
    if dictionary.name("Subtype") != Some("Form") {
        visited.remove(&reference);
        return;
    }

    let matrix = parent_matrix.multiply(form_matrix(dictionary));
    if let Some(stream) = object
        .stream
        .as_deref()
        .and_then(|stream| streams::decoded_stream_data(dictionary, stream).ok())
    {
        segments.extend(
            text::extract_segments_with_context(
                &stream,
                font_cmaps,
                font_metrics,
                struct_roles,
                struct_actual_text,
                page_reference,
            )
            .into_iter()
            .map(|segment| transform_form_segment(segment, matrix)),
        );
        collect_content_form_text_segments(
            objects,
            &stream,
            &xobject_resources(objects, dictionary),
            matrix,
            font_cmaps,
            font_metrics,
            struct_roles,
            struct_actual_text,
            page_reference,
            visited,
            segments,
        );
    }
    visited.remove(&reference);
}

fn transform_form_segment(mut segment: text::TextSegment, matrix: FormMatrix) -> text::TextSegment {
    let (x, y) = matrix.transform_point(segment.x, segment.y);
    segment.x = x;
    segment.y = y;
    segment.width *= matrix.x_scale();
    segment.font_size *= matrix.y_scale();
    segment
}

pub(super) fn form_xobject_graphics(
    objects: &PdfObjects,
    resources: &std::collections::HashMap<String, object::PdfReference>,
    content_streams: &[Vec<u8>],
) -> (Vec<graphics::RectShape>, Vec<graphics::VectorPath>) {
    let mut visited = std::collections::HashSet::new();
    let mut shapes = Vec::new();
    let mut paths = Vec::new();
    for stream in content_streams {
        collect_content_form_graphics(
            objects,
            stream,
            resources,
            FormMatrix::identity(),
            &mut visited,
            &mut shapes,
            &mut paths,
        );
    }
    (shapes, paths)
}

fn collect_content_form_graphics(
    objects: &PdfObjects,
    content_stream: &[u8],
    resources: &std::collections::HashMap<String, object::PdfReference>,
    initial_ctm: FormMatrix,
    visited: &mut std::collections::HashSet<object::PdfReference>,
    shapes: &mut Vec<graphics::RectShape>,
    paths: &mut Vec<graphics::VectorPath>,
) {
    walk_content_xobjects(content_stream, resources, initial_ctm, |reference, ctm| {
        collect_form_xobject_graphics(objects, reference, ctm, visited, shapes, paths);
    });
}

fn walk_content_xobjects(
    content_stream: &[u8],
    resources: &std::collections::HashMap<String, object::PdfReference>,
    initial_ctm: FormMatrix,
    mut visit: impl FnMut(object::PdfReference, FormMatrix),
) {
    let mut ctm = initial_ctm;
    let mut stack = Vec::new();
    let mut operands = Vec::new();

    for token in tokenize_content(content_stream) {
        match token {
            ContentToken::Number(value) => operands.push(ContentOperand::Number(value)),
            ContentToken::Name(name) => operands.push(ContentOperand::Name(name)),
            ContentToken::Operator(operator) => {
                match operator.as_str() {
                    "q" => stack.push(ctm),
                    "Q" => ctm = stack.pop().unwrap_or(initial_ctm),
                    "cm" => {
                        if let Some(values) = last_numbers::<6>(&operands) {
                            ctm = ctm.multiply(FormMatrix {
                                a: values[0],
                                b: values[1],
                                c: values[2],
                                d: values[3],
                                e: values[4],
                                f: values[5],
                            });
                        }
                    }
                    "Do" => {
                        if let Some(name) = last_name(&operands) {
                            if let Some(reference) = resources.get(name) {
                                visit(*reference, ctm);
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

fn collect_form_xobject_graphics(
    objects: &PdfObjects,
    reference: object::PdfReference,
    parent_matrix: FormMatrix,
    visited: &mut std::collections::HashSet<object::PdfReference>,
    shapes: &mut Vec<graphics::RectShape>,
    paths: &mut Vec<graphics::VectorPath>,
) {
    if !visited.insert(reference) {
        return;
    }
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        visited.remove(&reference);
        return;
    };
    let Some(dictionary) = object.dictionary() else {
        visited.remove(&reference);
        return;
    };
    if dictionary.name("Subtype") != Some("Form") {
        visited.remove(&reference);
        return;
    }

    let matrix = parent_matrix.multiply(form_matrix(dictionary));
    if let Some(stream) = object
        .stream
        .as_deref()
        .and_then(|stream| streams::decoded_stream_data(dictionary, stream).ok())
    {
        shapes.extend(
            graphics::extract_rectangles(&stream)
                .into_iter()
                .map(|shape| transform_form_shape(shape, matrix)),
        );
        paths.extend(
            graphics::extract_paths(&stream)
                .into_iter()
                .map(|path| transform_form_path(path, matrix)),
        );
        collect_content_form_graphics(
            objects,
            &stream,
            &xobject_resources(objects, dictionary),
            matrix,
            visited,
            shapes,
            paths,
        );
    }
    visited.remove(&reference);
}

fn transform_form_shape(mut shape: graphics::RectShape, matrix: FormMatrix) -> graphics::RectShape {
    let points = [
        matrix.transform_point(shape.x, shape.y),
        matrix.transform_point(shape.x + shape.width, shape.y),
        matrix.transform_point(shape.x, shape.y + shape.height),
        matrix.transform_point(shape.x + shape.width, shape.y + shape.height),
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
    shape.x = min_x;
    shape.y = min_y;
    shape.width = max_x - min_x;
    shape.height = max_y - min_y;
    shape
}

fn transform_form_path(mut path: graphics::VectorPath, matrix: FormMatrix) -> graphics::VectorPath {
    path.commands = path
        .commands
        .into_iter()
        .map(|command| transform_form_path_command(command, matrix))
        .collect();
    path.stroke_width *= matrix.x_scale().max(matrix.y_scale());
    path
}

fn transform_form_path_command(
    command: graphics::PathCommand,
    matrix: FormMatrix,
) -> graphics::PathCommand {
    match command {
        graphics::PathCommand::MoveTo(x, y) => {
            let (x, y) = matrix.transform_point(x, y);
            graphics::PathCommand::MoveTo(x, y)
        }
        graphics::PathCommand::LineTo(x, y) => {
            let (x, y) = matrix.transform_point(x, y);
            graphics::PathCommand::LineTo(x, y)
        }
        graphics::PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
            let (x1, y1) = matrix.transform_point(x1, y1);
            let (x2, y2) = matrix.transform_point(x2, y2);
            let (x, y) = matrix.transform_point(x, y);
            graphics::PathCommand::CubicTo(x1, y1, x2, y2, x, y)
        }
        graphics::PathCommand::Close => graphics::PathCommand::Close,
    }
}

fn form_matrix(dictionary: &PdfDictionary) -> FormMatrix {
    let Some(values) = dictionary.array("Matrix") else {
        return FormMatrix::identity();
    };
    let [a, b, c, d, e, f] = values else {
        return FormMatrix::identity();
    };
    let Some(a) = pdf_number(a) else {
        return FormMatrix::identity();
    };
    let Some(b) = pdf_number(b) else {
        return FormMatrix::identity();
    };
    let Some(c) = pdf_number(c) else {
        return FormMatrix::identity();
    };
    let Some(d) = pdf_number(d) else {
        return FormMatrix::identity();
    };
    let Some(e) = pdf_number(e) else {
        return FormMatrix::identity();
    };
    let Some(f) = pdf_number(f) else {
        return FormMatrix::identity();
    };
    FormMatrix { a, b, c, d, e, f }
}

fn xobject_resources(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
) -> std::collections::HashMap<String, object::PdfReference> {
    let Some(resources) = dictionary_value(objects, dictionary.get("Resources")) else {
        return std::collections::HashMap::new();
    };
    let Some(xobjects) = dictionary_value(objects, resources.get("XObject")) else {
        return std::collections::HashMap::new();
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

#[cfg(test)]
mod tests;
