use std::collections::HashMap;

use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

use super::{color_channel, pdf_number, rgb_color};

pub(super) fn page_image_resources(
    objects: &PdfObjects,
    page: &PdfDictionary,
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

pub(super) fn page_font_resources(
    objects: &PdfObjects,
    page: &PdfDictionary,
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

pub(super) fn page_shading_resources(
    objects: &PdfObjects,
    page: &PdfDictionary,
) -> HashMap<String, String> {
    let Some(resources) = inherited_resource_dictionary(objects, page) else {
        return HashMap::new();
    };
    let Some(shadings) = dictionary_value(objects, resources.get("Shading")) else {
        return HashMap::new();
    };

    shadings
        .iter()
        .filter_map(|(name, value)| Some((name.clone(), shading_color(objects, value)?)))
        .collect()
}

fn shading_color(objects: &PdfObjects, value: &PdfValue) -> Option<String> {
    let shading = dictionary_value(objects, Some(value))?;
    let channels = color_space_channels(shading.get("ColorSpace")?)?;
    function_color(objects, shading.get("Function")?, channels)
}

fn color_space_channels(value: &PdfValue) -> Option<usize> {
    match value {
        PdfValue::Name(name) if name == "DeviceRGB" => Some(3),
        PdfValue::Name(name) if name == "DeviceGray" => Some(1),
        PdfValue::Array(values) => values.first().and_then(color_space_channels),
        _ => None,
    }
}

fn function_color(objects: &PdfObjects, value: &PdfValue, channels: usize) -> Option<String> {
    match value {
        PdfValue::Dictionary(dictionary) => {
            function_dictionary_color(objects, dictionary, channels)
        }
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))
            .and_then(|object| object.dictionary())
            .and_then(|dictionary| function_dictionary_color(objects, dictionary, channels)),
        PdfValue::Array(values) => values
            .iter()
            .rev()
            .find_map(|value| function_color(objects, value, channels)),
        _ => None,
    }
}

fn function_dictionary_color(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    channels: usize,
) -> Option<String> {
    match dictionary.integer("FunctionType")? {
        2 => dictionary
            .array("C1")
            .or_else(|| dictionary.array("C0"))
            .and_then(|values| color_from_array(values, channels)),
        3 => dictionary
            .array("Functions")?
            .iter()
            .rev()
            .find_map(|value| function_color(objects, value, channels)),
        _ => None,
    }
}

fn color_from_array(values: &[PdfValue], channels: usize) -> Option<String> {
    match channels {
        1 => Some(gray_color(pdf_number(values.first()?)?)),
        3 => {
            let [red, green, blue, ..] = values else {
                return None;
            };
            Some(rgb_color(
                pdf_number(red)?,
                pdf_number(green)?,
                pdf_number(blue)?,
            ))
        }
        _ => None,
    }
}

fn gray_color(value: f32) -> String {
    let channel = color_channel(value);
    format!("#{channel:02x}{channel:02x}{channel:02x}")
}

fn inherited_resource_dictionary<'a>(
    objects: &'a PdfObjects,
    page: &'a PdfDictionary,
) -> Option<&'a PdfDictionary> {
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

pub(super) fn media_box_size(dictionary: &PdfDictionary) -> Option<(Option<f32>, Option<f32>)> {
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
