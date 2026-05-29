mod shapes;

use std::collections::HashMap;

use ttf_parser::{Face, GlyphId};

use super::{encoding, CMap};
use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};
use crate::pdf::streams;

pub(super) fn embedded_font_cmap(
    source: &[u8],
    objects: &PdfObjects,
    _font_reference: PdfReference,
    font_dictionary: &PdfDictionary,
) -> Result<Option<CMap>, crate::ConvertError> {
    let Some(font_data) = embedded_font_data(source, objects, font_dictionary)? else {
        return Ok(None);
    };
    let Ok(face) = Face::parse(&font_data, 0) else {
        return Ok(None);
    };

    let mappings = if font_dictionary.name("Subtype") == Some("Type0") {
        type0_embedded_mappings(source, objects, font_dictionary, &face)?
    } else {
        simple_embedded_mappings(font_dictionary, &face)
    };

    if mappings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(CMap::from_code_mappings(mappings)))
    }
}

fn embedded_font_data(
    source: &[u8],
    objects: &PdfObjects,
    font_dictionary: &PdfDictionary,
) -> Result<Option<Vec<u8>>, crate::ConvertError> {
    let Some(descriptor) = font_descriptor(objects, font_dictionary) else {
        return Ok(None);
    };
    for key in ["FontFile2", "FontFile3", "FontFile"] {
        let Some(reference) = descriptor.get_ref(key) else {
            continue;
        };
        if let Some(data) = streams::stream_data_for_reference(source, objects, reference)? {
            return Ok(Some(data));
        }
    }
    Ok(None)
}

fn font_descriptor<'a>(
    objects: &'a PdfObjects,
    font_dictionary: &'a PdfDictionary,
) -> Option<&'a PdfDictionary> {
    if let Some(descriptor) =
        dictionary_ref_or_inline(objects, font_dictionary.get("FontDescriptor"))
    {
        return Some(descriptor);
    }
    let descendant = descendant_font_dictionary(objects, font_dictionary)?;
    dictionary_ref_or_inline(objects, descendant.get("FontDescriptor"))
}

fn descendant_font_dictionary<'a>(
    objects: &'a PdfObjects,
    dictionary: &'a PdfDictionary,
) -> Option<&'a PdfDictionary> {
    let descendants = dictionary.array("DescendantFonts")?;
    match descendants.first()? {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))?
            .dictionary(),
        _ => None,
    }
}

fn dictionary_ref_or_inline<'a>(
    objects: &'a PdfObjects,
    value: Option<&'a PdfValue>,
) -> Option<&'a PdfDictionary> {
    match value? {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects
            .get(*reference)
            .or_else(|| objects.latest(reference.object))?
            .dictionary(),
        _ => None,
    }
}

fn simple_embedded_mappings(
    font_dictionary: &PdfDictionary,
    face: &Face<'_>,
) -> Vec<(Vec<u8>, String)> {
    if font_dictionary.get("Encoding").is_some() {
        return Vec::new();
    }

    (0u8..=255)
        .filter_map(|code| {
            let ch = char::from_u32(u32::from(code))?;
            if !is_semantic_char(ch) || face.glyph_index(ch).is_none() {
                return None;
            }
            Some((vec![code], ch.to_string()))
        })
        .collect()
}

fn type0_embedded_mappings(
    source: &[u8],
    objects: &PdfObjects,
    font_dictionary: &PdfDictionary,
    face: &Face<'_>,
) -> Result<Vec<(Vec<u8>, String)>, crate::ConvertError> {
    if !matches!(
        font_dictionary.name("Encoding"),
        Some("Identity-H" | "Identity-V")
    ) {
        return Ok(Vec::new());
    }
    let Some(descendant) = descendant_font_dictionary(objects, font_dictionary) else {
        return Ok(Vec::new());
    };
    let gid_to_unicode = glyph_unicode_map(face);
    if gid_to_unicode.is_empty() {
        return Ok(Vec::new());
    }

    if let Some(cid_to_gid) = cid_to_gid_map(source, objects, descendant)? {
        Ok(cid_to_gid
            .into_iter()
            .filter_map(|(cid, gid)| {
                let text = gid_to_unicode.get(&gid)?.clone();
                Some((cid.to_be_bytes().to_vec(), text))
            })
            .collect())
    } else {
        Ok(gid_to_unicode
            .into_iter()
            .map(|(gid, text)| (gid.to_be_bytes().to_vec(), text))
            .collect())
    }
}

fn cid_to_gid_map(
    source: &[u8],
    objects: &PdfObjects,
    descendant: &PdfDictionary,
) -> Result<Option<Vec<(u16, u16)>>, crate::ConvertError> {
    match descendant.get("CIDToGIDMap") {
        Some(PdfValue::Name(name)) if name == "Identity" => Ok(None),
        Some(PdfValue::Reference(reference)) => {
            let Some(data) = streams::stream_data_for_reference(source, objects, *reference)?
            else {
                return Ok(None);
            };
            let mappings = data
                .chunks_exact(2)
                .enumerate()
                .filter_map(|(cid, bytes)| {
                    let cid = u16::try_from(cid).ok()?;
                    let gid = u16::from_be_bytes([bytes[0], bytes[1]]);
                    Some((cid, gid))
                })
                .collect::<Vec<_>>();
            Ok(Some(mappings))
        }
        _ => Ok(None),
    }
}

fn glyph_unicode_map(face: &Face<'_>) -> HashMap<u16, String> {
    let mut mappings = unicode_cmap_mappings(face);
    for (gid, text) in glyph_name_mappings(face) {
        mappings.entry(gid).or_insert(text);
    }
    for (gid, text) in shapes::shape_inferred_mappings(face, &mappings) {
        mappings.entry(gid).or_insert(text);
    }
    mappings
}

fn unicode_cmap_mappings(face: &Face<'_>) -> HashMap<u16, String> {
    let mut mappings = HashMap::new();
    if let Some(cmap) = face.tables().cmap {
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|codepoint| {
                let Some(ch) = char::from_u32(codepoint) else {
                    return;
                };
                if !is_semantic_char(ch) {
                    return;
                }
                if let Some(gid) = subtable.glyph_index(codepoint) {
                    mappings.entry(gid.0).or_insert_with(|| ch.to_string());
                }
            });
        }
    }
    mappings
}

fn glyph_name_mappings(face: &Face<'_>) -> HashMap<u16, String> {
    (0..face.number_of_glyphs())
        .filter_map(|gid| {
            let name = face.glyph_name(GlyphId(gid))?;
            let text = encoding::glyph_name_unicode(name)?;
            is_semantic_text(&text).then_some((gid, text))
        })
        .collect()
}

fn is_semantic_text(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_semantic_char)
}

fn is_semantic_char(ch: char) -> bool {
    !ch.is_control() && ch != '\u{fffd}' && ch != '\0'
}
