use super::PngColor;
use crate::pdf::object::{PdfDictionary, PdfObjects, PdfReference, PdfValue};
use crate::pdf::streams;

#[derive(Debug)]
pub(in crate::pdf::images::encoding) struct IndexedPalette {
    pub(in crate::pdf::images::encoding) color: PngColor,
    pub(in crate::pdf::images::encoding) lookup: Vec<u8>,
}

pub(in crate::pdf::images::encoding) fn indexed_palette(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
) -> Result<Option<IndexedPalette>, String> {
    let Some(PdfValue::Array(values)) = dictionary.get("ColorSpace") else {
        return Ok(None);
    };
    if values.len() != 4 {
        return Err(format!(
            "unsupported image color space array with {} entries",
            values.len()
        ));
    }
    if !matches!(values.first(), Some(PdfValue::Name(name)) if name == "Indexed" || name == "I") {
        return Ok(None);
    }
    let color = match values.get(1) {
        Some(PdfValue::Name(name)) if name == "DeviceRGB" || name == "RGB" => PngColor::Rgb,
        Some(PdfValue::Name(name)) if name == "DeviceGray" || name == "G" => PngColor::Grayscale,
        Some(PdfValue::Array(base)) => indexed_base_color(base)?,
        Some(PdfValue::Reference(reference)) => indexed_base_color_reference(objects, *reference)?,
        _ => return Err("unsupported indexed image base color space".to_string()),
    };
    let high_value = match values.get(2) {
        Some(PdfValue::Integer(value)) => usize::try_from(*value)
            .ok()
            .ok_or_else(|| format!("invalid indexed image high value {value}"))?,
        _ => return Err("indexed image color space is missing high value".to_string()),
    };
    let lookup = lookup_bytes(objects, values.get(3))?;
    let required_len = (high_value + 1)
        .checked_mul(color.channels())
        .ok_or_else(|| "indexed image palette is too large".to_string())?;
    if lookup.len() < required_len {
        return Err(format!(
            "indexed image palette is shorter than expected ({} < {required_len})",
            lookup.len()
        ));
    }
    Ok(Some(IndexedPalette {
        color,
        lookup: lookup[..required_len].to_vec(),
    }))
}

fn indexed_base_color(values: &[PdfValue]) -> Result<PngColor, String> {
    match values.first() {
        Some(PdfValue::Name(name)) if name == "DeviceRGB" || name == "RGB" => Ok(PngColor::Rgb),
        Some(PdfValue::Name(name)) if name == "DeviceGray" || name == "G" => {
            Ok(PngColor::Grayscale)
        }
        _ => Err("unsupported indexed image base color space".to_string()),
    }
}

fn indexed_base_color_reference(
    objects: &PdfObjects,
    reference: PdfReference,
) -> Result<PngColor, String> {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return Err("indexed image base color space reference was not found".to_string());
    };
    match &object.value {
        PdfValue::Name(name) if name == "DeviceRGB" || name == "RGB" => Ok(PngColor::Rgb),
        PdfValue::Name(name) if name == "DeviceGray" || name == "G" => Ok(PngColor::Grayscale),
        PdfValue::Array(values) => indexed_base_color(values),
        _ => Err("unsupported indexed image base color space".to_string()),
    }
}

fn lookup_bytes(objects: &PdfObjects, value: Option<&PdfValue>) -> Result<Vec<u8>, String> {
    match value {
        Some(PdfValue::String(bytes)) => Ok(bytes.clone()),
        Some(PdfValue::Reference(reference)) => {
            let Some(object) = objects
                .get(*reference)
                .or_else(|| objects.latest(reference.object))
            else {
                return Err("indexed image lookup reference was not found".to_string());
            };
            let dictionary = object
                .dictionary()
                .ok_or_else(|| "indexed image lookup object is not a stream".to_string())?;
            let stream = object
                .stream
                .as_deref()
                .ok_or_else(|| "indexed image lookup object has no stream data".to_string())?;
            streams::decoded_stream_data(dictionary, stream)
                .map_err(|error| format!("could not decode indexed image lookup stream ({error})"))
        }
        _ => Err("indexed image color space is missing lookup data".to_string()),
    }
}
