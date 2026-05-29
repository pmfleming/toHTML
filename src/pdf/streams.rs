use std::collections::HashMap;

use crate::ConvertError;

use super::object::{PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

mod annotations;
mod filters;
mod legacy;
mod resources;
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
    pub reference: PdfReference,
    pub page_number: u32,
    pub streams: Vec<Vec<u8>>,
    pub warnings: Vec<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub image_resources: HashMap<String, PdfReference>,
    pub font_resources: HashMap<String, PdfReference>,
    pub shading_resources: HashMap<String, String>,
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
            reference,
            page_number,
            ..PageContent::default()
        });
    };
    let page_dictionary = page.dictionary();
    let (width, height) = page
        .dictionary()
        .and_then(resources::media_box_size)
        .unwrap_or((None, None));
    let image_resources = page
        .dictionary()
        .map(|dictionary| resources::page_image_resources(objects, dictionary))
        .unwrap_or_default();
    let font_resources = page
        .dictionary()
        .map(|dictionary| resources::page_font_resources(objects, dictionary))
        .unwrap_or_default();
    let shading_resources = page
        .dictionary()
        .map(|dictionary| resources::page_shading_resources(objects, dictionary))
        .unwrap_or_default();
    let ink_annotations = page_dictionary
        .map(|dictionary| annotations::page_ink_annotations(objects, dictionary))
        .unwrap_or_default();
    let link_annotations = page_dictionary
        .map(|dictionary| annotations::page_link_annotations(objects, dictionary))
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
        reference,
        streams,
        warnings,
        width,
        height,
        image_resources,
        font_resources,
        shading_resources,
        ink_annotations,
        link_annotations,
    })
}

pub(super) fn positive_number(value: &PdfValue) -> Option<f32> {
    pdf_number(value).filter(|value| value.is_finite() && *value > 0.0)
}

pub(super) fn rgb_color(red: f32, green: f32, blue: f32) -> String {
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

pub(super) fn pdf_number(value: &PdfValue) -> Option<f32> {
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

pub(super) fn stream_data_for_reference(
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
    let Some(dictionary) = object.dictionary() else {
        return Ok(Some(data.clone()));
    };
    Ok(Some(filters::decode_stream(dictionary, data)?))
}

pub(super) fn decoded_stream_data(
    dictionary: &super::object::PdfDictionary,
    data: &[u8],
) -> Result<Vec<u8>, ConvertError> {
    filters::decode_stream(dictionary, data)
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
