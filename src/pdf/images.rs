use std::collections::{HashMap, HashSet};

use crate::ConversionWarning;

use super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference};
use super::streams;
use super::visual::VisualImage;

mod encoding;
mod placement;
#[cfg(test)]
mod tests;
mod tokens;

pub(super) fn extract_page_images(
    source: &[u8],
    objects: &PdfObjects,
    content_streams: &[Vec<u8>],
    resources: &HashMap<String, PdfReference>,
    page_number: u32,
    warnings: &mut Vec<ConversionWarning>,
) -> Vec<VisualImage> {
    let mut cache = HashMap::new();
    let mut warned = HashSet::new();
    let mut images = Vec::new();

    for placement in
        placement::image_placements(source, objects, content_streams, resources, warnings)
    {
        let Some(data) = image_data(
            source,
            objects,
            placement.reference,
            page_number,
            warnings,
            &mut warned,
            &mut cache,
        ) else {
            continue;
        };
        images.push(VisualImage {
            src: data.src,
            mask_src: data.mask_src,
            alt: format!("PDF image on page {page_number}"),
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
        });
    }

    images
}

#[derive(Debug, Clone)]
struct ImageData {
    src: String,
    mask_src: Option<String>,
}

fn image_data(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
    page_number: u32,
    warnings: &mut Vec<ConversionWarning>,
    warned: &mut HashSet<PdfReference>,
    cache: &mut HashMap<PdfReference, Option<ImageData>>,
) -> Option<ImageData> {
    if let Some(cached) = cache.get(&reference) {
        return cached.clone();
    }

    let data = extract_image_data(source, objects, reference).unwrap_or_else(|message| {
        if warned.insert(reference) {
            warnings.push(ConversionWarning {
                message: format!("Page {page_number}: skipped PDF image: {message}"),
                source: None,
            });
        }
        None
    });
    cache.insert(reference, data.clone());
    data
}

fn extract_image_data(
    _source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
) -> Result<Option<ImageData>, String> {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return Ok(None);
    };
    let Some(dictionary) = object.dictionary() else {
        return Ok(None);
    };
    if dictionary.name("Subtype") != Some("Image") {
        return Ok(None);
    }
    let filters = streams::stream_filters(dictionary);
    let stream = object
        .stream
        .as_deref()
        .ok_or_else(|| "image object has no stream data".to_string())?;
    let data = streams::decoded_stream_data(dictionary, stream)
        .map_err(|error| format!("could not decode image stream ({error})"))?;
    let (media_type, image_data) = if let Some(media_type) = native_image_media_type(&filters) {
        (media_type, data)
    } else {
        (
            "image/png",
            encoding::png_from_raw_image(objects, dictionary, &data)?,
        )
    };
    let mask_src = soft_mask_src(objects, dictionary);

    Ok(Some(ImageData {
        src: encoding::data_uri(media_type, &image_data),
        mask_src,
    }))
}

fn soft_mask_src(objects: &PdfObjects, dictionary: &PdfDictionary) -> Option<String> {
    let reference = dictionary.get_ref("SMask")?;
    let object = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))?;
    let mask_dictionary = object.dictionary()?;
    let stream = object.stream.as_deref()?;
    let data = streams::decoded_stream_data(mask_dictionary, stream).ok()?;
    let png = encoding::png_alpha_from_gray_mask(mask_dictionary, &data).ok()?;
    Some(encoding::data_uri("image/png", &png))
}

fn native_image_media_type(filters: &[String]) -> Option<&'static str> {
    if filters
        .iter()
        .any(|filter| matches!(filter.as_str(), "DCTDecode" | "DCT"))
    {
        return Some("image/jpeg");
    }
    if filters
        .iter()
        .any(|filter| matches!(filter.as_str(), "JPXDecode" | "JPX"))
    {
        return Some("image/jp2");
    }
    None
}
