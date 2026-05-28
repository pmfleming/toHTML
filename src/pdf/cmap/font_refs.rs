use std::collections::HashMap;

use super::{encoding, CMap};
use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};
use crate::pdf::streams;

pub fn font_cmaps(bytes: &[u8]) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    let mut maps = HashMap::new();
    let objects = PdfObjects::parse(bytes);
    for (font_name, font_object) in font_references(bytes) {
        if let Some(cmap) = cmap_for_font(
            bytes,
            &objects,
            PdfReference {
                object: font_object,
                generation: 0,
            },
        )? {
            maps.insert(font_name, cmap);
        }
    }
    Ok(maps)
}

pub fn font_cmaps_for_resources(
    bytes: &[u8],
    objects: &PdfObjects,
    resources: &HashMap<String, PdfReference>,
) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    let mut maps = HashMap::new();
    for (font_name, reference) in resources {
        if let Some(cmap) = cmap_for_font(bytes, objects, *reference)? {
            maps.insert(font_name.clone(), cmap);
        }
    }
    Ok(maps)
}

fn cmap_for_font(
    bytes: &[u8],
    objects: &PdfObjects,
    font_reference: PdfReference,
) -> Result<Option<CMap>, crate::ConvertError> {
    let Some(font_dictionary) = font_dictionary(objects, font_reference) else {
        return Ok(None);
    };
    let mut decoder = fallback_cmap_for_font(objects, font_dictionary);

    let Some(cmap_reference) = parsed_to_unicode_ref(objects, font_reference).or_else(|| {
        legacy_to_unicode_ref(bytes, font_reference.object).map(|object| PdfReference {
            object,
            generation: 0,
        })
    }) else {
        return Ok(decoder);
    };
    let Some(cmap_stream) = streams::stream_data_for_object(bytes, cmap_reference.object)? else {
        return Ok(decoder);
    };
    let to_unicode = CMap::parse(&cmap_stream);
    match &mut decoder {
        Some(fallback) => fallback.merge(to_unicode),
        None => decoder = Some(to_unicode),
    }
    Ok(decoder)
}

fn parsed_to_unicode_ref(
    objects: &PdfObjects,
    font_reference: PdfReference,
) -> Option<PdfReference> {
    objects
        .get(font_reference)
        .or_else(|| objects.latest(font_reference.object))
        .and_then(|object| object.dictionary())
        .and_then(|dictionary| dictionary.get_ref("ToUnicode"))
}

fn font_dictionary(objects: &PdfObjects, reference: PdfReference) -> Option<&PdfDictionary> {
    objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
        .and_then(|object| object.dictionary())
}

fn fallback_cmap_for_font(objects: &PdfObjects, dictionary: &PdfDictionary) -> Option<CMap> {
    if dictionary.name("Subtype") == Some("Type0") {
        return type0_identity_cmap(objects, dictionary);
    }

    let mappings = encoding::simple_font_mappings(objects, dictionary);
    (!mappings.is_empty()).then(|| CMap::from_byte_mappings(mappings))
}

fn type0_identity_cmap(objects: &PdfObjects, dictionary: &PdfDictionary) -> Option<CMap> {
    if !matches!(
        dictionary.name("Encoding"),
        Some("Identity-H" | "Identity-V")
    ) {
        return None;
    }
    let descendant = descendant_font_dictionary(objects, dictionary)?;
    let cid_info = match descendant.get("CIDSystemInfo")? {
        PdfValue::Dictionary(dictionary) => dictionary,
        PdfValue::Reference(reference) => objects.get(*reference)?.dictionary()?,
        _ => return None,
    };
    let registry = string_or_name(cid_info.get("Registry"))?;
    let ordering = string_or_name(cid_info.get("Ordering"))?;
    matches!(
        (registry.as_str(), ordering.as_str()),
        ("Adobe", "Identity" | "UCS")
    )
    .then(CMap::identity_two_byte)
}

fn descendant_font_dictionary<'a>(
    objects: &'a PdfObjects,
    dictionary: &'a PdfDictionary,
) -> Option<&'a PdfDictionary> {
    let descendants = dictionary.array("DescendantFonts")?;
    match descendants.first()? {
        PdfValue::Dictionary(dictionary) => Some(dictionary),
        PdfValue::Reference(reference) => objects.get(*reference)?.dictionary(),
        _ => None,
    }
}

fn string_or_name(value: Option<&PdfValue>) -> Option<String> {
    match value? {
        PdfValue::String(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        PdfValue::Name(name) => Some(name.clone()),
        _ => None,
    }
}

fn legacy_to_unicode_ref(bytes: &[u8], font_object: u32) -> Option<u32> {
    let body = streams::object_body(bytes, font_object)?;
    to_unicode_ref(body)
}

fn font_references(bytes: &[u8]) -> HashMap<String, u32> {
    streams::named_object_refs(bytes, "/F")
}

fn to_unicode_ref(body: &[u8]) -> Option<u32> {
    streams::object_ref_after(body, b"/ToUnicode")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_packed_font_resource_references() {
        let pdf = br#"<</Font<</F1 5 0 R/F3 9 0 R/F4 108 0 R>>>>"#;

        let refs = font_references(pdf);

        assert_eq!(refs["F1"], 5);
        assert_eq!(refs["F3"], 9);
        assert_eq!(refs["F4"], 108);
    }

    #[test]
    fn finds_packed_to_unicode_reference() {
        let body = br#"<</Type/Font/Subtype/Type0/DescendantFonts 10 0 R/ToUnicode 14981 0 R>>"#;

        assert_eq!(to_unicode_ref(body), Some(14981));
    }

    #[test]
    fn builds_decoder_from_simple_font_encoding_differences() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type1 /Encoding << /BaseEncoding /WinAnsiEncoding /Differences [65 /T 66 /h 67 /e 128 /Euro 129 /uniFB01] >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[65, 66, 67, 32, 128, 129]), "The €ﬁ");
    }

    #[test]
    fn lets_to_unicode_override_simple_font_encoding_fallback() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type1 /Encoding /WinAnsiEncoding /ToUnicode 3 0 R >> endobj
3 0 obj << /Length 68 >>
stream
beginbfchar
<41> <0058>
endbfchar
endstream
endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(b"ABC"), "XBC");
    }

    #[test]
    fn builds_identity_decoder_for_adobe_identity_type0_fonts() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type0 /Encoding /Identity-H /DescendantFonts [3 0 R] >> endobj
3 0 obj << /Type /Font /Subtype /CIDFontType2 /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0, 0x48, 0, 0x69]), "Hi");
    }
}
