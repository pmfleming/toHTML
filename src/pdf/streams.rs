use std::collections::HashMap;

use crate::ConvertError;

use super::object::{PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

mod filters;
mod legacy;
#[cfg(test)]
mod tests;

pub use legacy::{
    named_object_refs, number_after, object_body, object_ids, object_ref_after,
    stream_data_for_object,
};

pub struct PdfPageExtraction {
    pub pages: Vec<PageContent>,
    pub warnings: Vec<String>,
    pub page_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PageContent {
    pub page_number: u32,
    pub streams: Vec<Vec<u8>>,
    pub warnings: Vec<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub image_resources: HashMap<String, PdfReference>,
    pub font_resources: HashMap<String, PdfReference>,
    pub ink_annotations: Vec<InkAnnotation>,
    pub link_annotations: Vec<LinkAnnotation>,
}

#[derive(Debug, Clone, Default)]
pub struct InkAnnotation {
    pub paths: Vec<Vec<(f32, f32)>>,
    pub color: Option<String>,
    pub width: f32,
}

#[derive(Debug, Clone, Default)]
pub struct LinkAnnotation {
    pub uri: String,
    pub rect: (f32, f32, f32, f32),
}

pub fn document_pages(bytes: &[u8]) -> Result<PdfPageExtraction, ConvertError> {
    let objects = PdfObjects::parse(bytes);
    let page_refs = page_references(&objects);
    let pages = page_refs
        .iter()
        .enumerate()
        .map(|(index, reference)| page_content(bytes, &objects, *reference, index as u32 + 1))
        .collect::<Result<Vec<_>, _>>()?;
    let warnings = document_warnings(bytes, &objects, &pages);
    Ok(PdfPageExtraction {
        page_count: page_refs.len(),
        pages,
        warnings,
    })
}

fn page_references(objects: &PdfObjects) -> Vec<PdfReference> {
    catalog_pages(objects)
        .and_then(|root| walk_page_tree(objects, root))
        .filter(|pages| !pages.is_empty())
        .unwrap_or_else(|| flat_page_references(objects))
}

fn catalog_pages(objects: &PdfObjects) -> Option<PdfReference> {
    objects.values().find_map(|object| {
        (object.type_name() == Some("Catalog"))
            .then(|| object.dictionary()?.get_ref("Pages"))
            .flatten()
    })
}

fn walk_page_tree(objects: &PdfObjects, root: PdfReference) -> Option<Vec<PdfReference>> {
    let mut pages = Vec::new();
    collect_pages(objects, root, &mut pages);
    Some(pages)
}

fn collect_pages(objects: &PdfObjects, reference: PdfReference, pages: &mut Vec<PdfReference>) {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return;
    };
    match object.type_name() {
        Some("Page") => pages.push(reference),
        Some("Pages") => {
            if let Some(kids) = object
                .dictionary()
                .and_then(|dictionary| dictionary.array("Kids"))
            {
                for kid in kids.iter().filter_map(value_ref) {
                    collect_pages(objects, kid, pages);
                }
            }
        }
        _ => {}
    }
}

fn flat_page_references(objects: &PdfObjects) -> Vec<PdfReference> {
    objects
        .values()
        .filter(|object| object.type_name() == Some("Page"))
        .map(|object| object.reference)
        .collect()
}

fn page_content(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
    page_number: u32,
) -> Result<PageContent, ConvertError> {
    let Some(page) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return Ok(PageContent {
            page_number,
            streams: Vec::new(),
            warnings: Vec::new(),
            width: None,
            height: None,
            image_resources: HashMap::new(),
            font_resources: HashMap::new(),
            ink_annotations: Vec::new(),
            link_annotations: Vec::new(),
        });
    };
    let page_dictionary = page.dictionary();
    let (width, height) = page
        .dictionary()
        .and_then(media_box_size)
        .unwrap_or((None, None));
    let image_resources = page
        .dictionary()
        .map(|dictionary| page_image_resources(objects, dictionary))
        .unwrap_or_default();
    let font_resources = page
        .dictionary()
        .map(|dictionary| page_font_resources(objects, dictionary))
        .unwrap_or_default();
    let ink_annotations = page_dictionary
        .map(|dictionary| page_ink_annotations(objects, dictionary))
        .unwrap_or_default();
    let link_annotations = page_dictionary
        .map(|dictionary| page_link_annotations(objects, dictionary))
        .unwrap_or_default();
    let content_refs = page
        .dictionary()
        .and_then(|dictionary| dictionary.get("Contents"))
        .map(content_references)
        .unwrap_or_default();
    let mut streams = Vec::new();
    let mut warnings = Vec::new();
    for reference in content_refs {
        match stream_data_for_reference(source, objects, reference) {
            Ok(Some(stream)) => streams.push(stream),
            Ok(None) => {}
            Err(ConvertError::Pdf(message))
                if message.starts_with("unsupported PDF stream filter") =>
            {
                warnings.push(format!("Page {page_number}: {message}"));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(PageContent {
        page_number,
        streams,
        warnings,
        width,
        height,
        image_resources,
        font_resources,
        ink_annotations,
        link_annotations,
    })
}

fn page_link_annotations(
    objects: &PdfObjects,
    page: &super::object::PdfDictionary,
) -> Vec<LinkAnnotation> {
    page.array("Annots")
        .into_iter()
        .flatten()
        .filter_map(|value| annotation_dictionary(objects, value))
        .filter(|dictionary| dictionary.name("Subtype") == Some("Link"))
        .filter_map(link_annotation)
        .collect()
}

fn page_ink_annotations(
    objects: &PdfObjects,
    page: &super::object::PdfDictionary,
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
) -> Option<&'a super::object::PdfDictionary> {
    match value {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary()),
        _ => None,
    }
}

fn ink_annotation(dictionary: &super::object::PdfDictionary) -> Option<InkAnnotation> {
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

fn link_annotation(dictionary: &super::object::PdfDictionary) -> Option<LinkAnnotation> {
    Some(LinkAnnotation {
        uri: link_action_uri(dictionary)?,
        rect: annotation_rect(dictionary)?,
    })
}

fn link_action_uri(dictionary: &super::object::PdfDictionary) -> Option<String> {
    let action = match dictionary.get("A")? {
        PdfValue::Dictionary(values) => values,
        _ => return None,
    };
    let uri = action.string_bytes("URI")?;
    let uri = String::from_utf8_lossy(uri).to_string();
    is_plausible_link_uri(&uri).then_some(uri)
}

fn annotation_rect(dictionary: &super::object::PdfDictionary) -> Option<(f32, f32, f32, f32)> {
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

fn annotation_color(dictionary: &super::object::PdfDictionary) -> Option<String> {
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

fn annotation_stroke_width(dictionary: &super::object::PdfDictionary) -> Option<f32> {
    let border_style = match dictionary.get("BS")? {
        PdfValue::Dictionary(values) => values,
        _ => return None,
    };
    positive_number(border_style.get("W")?)
}

fn positive_number(value: &PdfValue) -> Option<f32> {
    pdf_number(value).filter(|value| value.is_finite() && *value > 0.0)
}

fn rgb_color(red: f32, green: f32, blue: f32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        color_channel(red),
        color_channel(green),
        color_channel(blue)
    )
}

fn color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn page_image_resources(
    objects: &PdfObjects,
    page: &super::object::PdfDictionary,
) -> HashMap<String, PdfReference> {
    let Some(resources) = inherited_resource_dictionary(objects, page) else {
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

fn page_font_resources(
    objects: &PdfObjects,
    page: &super::object::PdfDictionary,
) -> HashMap<String, PdfReference> {
    let Some(resources) = inherited_resource_dictionary(objects, page) else {
        return HashMap::new();
    };
    let Some(fonts) = dictionary_value(objects, resources.get("Font")) else {
        return HashMap::new();
    };

    fonts
        .iter()
        .filter_map(|(name, value)| match value {
            PdfValue::Reference(reference) => Some((name.clone(), *reference)),
            _ => None,
        })
        .collect()
}

fn inherited_resource_dictionary<'a>(
    objects: &'a PdfObjects,
    page: &'a super::object::PdfDictionary,
) -> Option<&'a super::object::PdfDictionary> {
    if let Some(resources) = dictionary_value(objects, page.get("Resources")) {
        return Some(resources);
    }

    let mut parent = page.get_ref("Parent");
    while let Some(reference) = parent {
        let dictionary = objects
            .get(reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary())?;
        if let Some(resources) = dictionary_value(objects, dictionary.get("Resources")) {
            return Some(resources);
        }
        parent = dictionary.get_ref("Parent");
    }
    None
}

fn dictionary_value<'a>(
    objects: &'a PdfObjects,
    value: Option<&'a PdfValue>,
) -> Option<&'a super::object::PdfDictionary> {
    match value? {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary()),
        _ => None,
    }
}

fn media_box_size(dictionary: &super::object::PdfDictionary) -> Option<(Option<f32>, Option<f32>)> {
    let box_values = dictionary.array("MediaBox")?;
    let [left, bottom, right, top] = box_values else {
        return None;
    };
    let width = pdf_number(right)? - pdf_number(left)?;
    let height = pdf_number(top)? - pdf_number(bottom)?;
    Some((positive_dimension(width), positive_dimension(height)))
}

fn positive_dimension(value: f32) -> Option<f32> {
    value
        .is_finite()
        .then_some(value.abs())
        .filter(|value| *value > 0.0)
}

fn pdf_number(value: &PdfValue) -> Option<f32> {
    match value {
        PdfValue::Integer(value) => Some(*value as f32),
        PdfValue::Real(value) => Some(*value),
        _ => None,
    }
}

fn content_references(value: &PdfValue) -> Vec<PdfReference> {
    match value {
        PdfValue::Reference(reference) => vec![*reference],
        PdfValue::Array(values) => values.iter().filter_map(value_ref).collect(),
        _ => Vec::new(),
    }
}

fn stream_data_for_reference(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
) -> Result<Option<Vec<u8>>, ConvertError> {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return stream_data_for_object(source, reference.object);
    };
    let Some(data) = &object.stream else {
        return Ok(None);
    };
    let filters = object
        .dictionary()
        .map(filters::stream_filters)
        .unwrap_or_default();
    Ok(Some(filters::decode_filters(&filters, data)?))
}

pub(super) fn decoded_stream_data(
    dictionary: &super::object::PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    filters::decode_filters(&filters::stream_filters(dictionary), data)
}

pub(super) fn stream_filters(dictionary: &super::object::PdfDictionary) -> Vec<String> {
    filters::stream_filters(dictionary)
}

fn value_ref(value: &PdfValue) -> Option<PdfReference> {
    match value {
        PdfValue::Reference(reference) => Some(*reference),
        _ => None,
    }
}

fn document_warnings(bytes: &[u8], objects: &PdfObjects, pages: &[PageContent]) -> Vec<String> {
    let mut warnings = Vec::new();
    if legacy::contains_token(bytes, b"/Encrypt") {
        warnings
            .push("PDF declares encryption; extraction may be blocked or incomplete".to_string());
    }
    if objects
        .values()
        .any(|object| object.type_name() == Some("XRef"))
    {
        warnings
            .push("PDF uses cross-reference streams; object stream support is limited".to_string());
    }
    if objects
        .values()
        .any(|object| object.type_name() == Some("ObjStm"))
    {
        warnings.push(
            "PDF uses object streams; compressed object extraction may be incomplete".to_string(),
        );
    }
    for page in pages.iter().filter(|page| page.streams.is_empty()) {
        warnings.push(format!(
            "Page {} has no supported extractable content stream",
            page.page_number
        ));
    }
    for page in pages {
        warnings.extend(page.warnings.iter().cloned());
    }
    warnings
}
