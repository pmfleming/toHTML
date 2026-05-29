use super::{pdf_number, positive_number, rgb_color, InkAnnotation, LinkAnnotation};
use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};

pub(super) fn page_link_annotations(
    objects: &PdfObjects,
    page: &PdfDictionary,
) -> Vec<LinkAnnotation> {
    page.array("Annots")
        .into_iter()
        .flatten()
        .filter_map(|value| annotation_dictionary(objects, value))
        .filter(|dictionary| dictionary.name("Subtype") == Some("Link"))
        .filter_map(link_annotation)
        .collect()
}

pub(super) fn page_ink_annotations(
    objects: &PdfObjects,
    page: &PdfDictionary,
) -> Vec<InkAnnotation> {
    page.array("Annots")
        .into_iter()
        .flatten()
        .filter_map(|value| annotation_dictionary(objects, value))
        .filter(|dictionary| dictionary.name("Subtype") == Some("Ink"))
        .filter_map(ink_annotation)
        .collect()
}

fn annotation_dictionary<'a>(
    objects: &'a PdfObjects,
    value: &'a PdfValue,
) -> Option<&'a PdfDictionary> {
    match value {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary()),
        _ => None,
    }
}

fn ink_annotation(dictionary: &PdfDictionary) -> Option<InkAnnotation> {
    let paths = dictionary
        .array("InkList")?
        .iter()
        .filter_map(ink_path)
        .collect::<Vec<_>>();
    (!paths.is_empty()).then(|| InkAnnotation {
        paths,
        color: annotation_color(dictionary),
        width: annotation_stroke_width(dictionary).unwrap_or(1.5),
    })
}

fn link_annotation(dictionary: &PdfDictionary) -> Option<LinkAnnotation> {
    Some(LinkAnnotation {
        uri: link_action_uri(dictionary)?,
        rect: annotation_rect(dictionary)?,
    })
}

fn link_action_uri(dictionary: &PdfDictionary) -> Option<String> {
    let action = match dictionary.get("A")? {
        PdfValue::Dictionary(values) => values,
        _ => return None,
    };
    let uri = action.string_bytes("URI")?;
    let uri = String::from_utf8_lossy(uri).to_string();
    is_plausible_link_uri(&uri).then_some(uri)
}

fn annotation_rect(dictionary: &PdfDictionary) -> Option<(f32, f32, f32, f32)> {
    let values = dictionary.array("Rect")?;
    let [x1, y1, x2, y2] = values else {
        return None;
    };
    let x1 = pdf_number(x1)?;
    let y1 = pdf_number(y1)?;
    let x2 = pdf_number(x2)?;
    let y2 = pdf_number(y2)?;
    Some((x1.min(x2), y1.min(y2), (x1 - x2).abs(), (y1 - y2).abs()))
}

fn is_plausible_link_uri(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        value if value.starts_with("http://")
            || value.starts_with("https://")
            || value.starts_with("mailto:")
            || value.starts_with("www.")
    ) && !uri.chars().any(|ch| ch.is_control())
}

fn ink_path(value: &PdfValue) -> Option<Vec<(f32, f32)>> {
    let PdfValue::Array(values) = value else {
        return None;
    };
    let numbers = values.iter().filter_map(pdf_number).collect::<Vec<_>>();
    let points = numbers
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect::<Vec<_>>();
    (points.len() >= 2).then_some(points)
}

fn annotation_color(dictionary: &PdfDictionary) -> Option<String> {
    let values = dictionary.array("C")?;
    let [red, green, blue] = values else {
        return None;
    };
    Some(rgb_color(
        pdf_number(red)?,
        pdf_number(green)?,
        pdf_number(blue)?,
    ))
}

fn annotation_stroke_width(dictionary: &PdfDictionary) -> Option<f32> {
    let border_style = match dictionary.get("BS")? {
        PdfValue::Dictionary(values) => values,
        _ => return None,
    };
    positive_number(border_style.get("W")?)
}
