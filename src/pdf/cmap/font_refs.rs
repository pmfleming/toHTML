use std::collections::HashMap;

use super::{embedded, encoding, predefined, CMap};
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

pub fn font_decoding_warnings(bytes: &[u8]) -> Result<Vec<String>, crate::ConvertError> {
    let objects = PdfObjects::parse(bytes);
    let mut warnings = Vec::new();
    for (font_name, font_object) in font_references(bytes) {
        let reference = PdfReference {
            object: font_object,
            generation: 0,
        };
        match cmap_for_font(bytes, &objects, reference)? {
            Some(cmap) => {
                warnings.extend(
                    cmap.warnings()
                        .iter()
                        .map(|warning| format!("Font /{font_name}: {warning}")),
                );
                if cmap.writing_mode() == super::WritingMode::Vertical {
                    warnings.push(format!(
                        "Font /{font_name}: vertical writing mode is decoded but layout ordering may be incomplete"
                    ));
                }
            }
            None => {
                if let Some(dictionary) = font_dictionary(&objects, reference) {
                    warnings.push(unsupported_font_warning(&font_name, dictionary));
                }
            }
        }
    }
    Ok(warnings)
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
    let mut decoder =
        embedded::embedded_font_cmap(bytes, objects, font_reference, font_dictionary)?;
    if let Some(fallback) = fallback_cmap_for_font(objects, font_dictionary) {
        match &mut decoder {
            Some(decoder) => decoder.merge(fallback),
            None => decoder = Some(fallback),
        }
    }

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
        return type0_cmap(objects, dictionary);
    }

    let mappings = encoding::simple_font_mappings(objects, dictionary);
    (!mappings.is_empty()).then(|| CMap::from_byte_mappings(mappings))
}

fn unsupported_font_warning(font_name: &str, dictionary: &PdfDictionary) -> String {
    let subtype = dictionary.name("Subtype").unwrap_or("unknown subtype");
    let base = dictionary.name("BaseFont").unwrap_or("unknown base font");
    format!("Font /{font_name} ({base}, {subtype}) has no supported Unicode mapping")
}

fn type0_cmap(objects: &PdfObjects, dictionary: &PdfDictionary) -> Option<CMap> {
    let encoding_name = dictionary.name("Encoding")?;
    if matches!(encoding_name, "Identity-H" | "Identity-V") {
        return type0_identity_cmap(objects, dictionary, encoding_name == "Identity-V");
    }
    if let Some(predefined) = predefined::predefined_cmap(encoding_name) {
        return Some(CMap::with_predefined_fallback(predefined));
    }
    None
}

fn type0_identity_cmap(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    vertical: bool,
) -> Option<CMap> {
    let descendant = descendant_font_dictionary(objects, dictionary)?;
    let cid_info = match descendant.get("CIDSystemInfo")? {
        PdfValue::Dictionary(dictionary) => dictionary,
        PdfValue::Reference(reference) => objects.get(*reference)?.dictionary()?,
        _ => return None,
    };
    let registry = string_or_name(cid_info.get("Registry"))?;
    let ordering = string_or_name(cid_info.get("Ordering"))?;
    predefined::identity_cmap(&registry, &ordering, vertical).map(CMap::with_predefined_fallback)
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
    fn decodes_agl_names_from_differences() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type1 /Encoding << /BaseEncoding /StandardEncoding /Differences [39 /quoteright /quoteleft /fi /fl /bullet] >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[39, 40, 41, 42, 43]), "’‘ﬁﬂ•");
    }

    #[test]
    fn decodes_symbolic_base_encodings() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R /F2 3 0 R /F3 4 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type1 /Encoding /StandardEncoding >> endobj
3 0 obj << /Type /Font /Subtype /Type1 /Encoding /SymbolEncoding >> endobj
4 0 obj << /Type /Font /Subtype /Type1 /Encoding /ZapfDingbatsEncoding >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0x27]), "’");
        assert_eq!(maps["F2"].decode(&[0x61]), "α");
        assert_eq!(maps["F3"].decode(&[0x21]), "✁");
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
    fn builds_identity_decoder_for_adobe_ucs_type0_fonts() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type0 /Encoding /Identity-H /DescendantFonts [3 0 R] >> endobj
3 0 obj << /Type /Font /Subtype /CIDFontType2 /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0, 0x48, 0, 0x69]), "Hi");
    }

    #[test]
    fn does_not_decode_adobe_identity_cids_as_unicode_without_font_evidence() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type0 /Encoding /Identity-H /DescendantFonts [3 0 R] >> endobj
3 0 obj << /Type /Font /Subtype /CIDFontType2 /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert!(!maps.contains_key("F1"));
    }

    #[test]
    fn decodes_known_type0_unicode_cmap_names() {
        let pdf = br#"
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type0 /Encoding /UniJIS-UCS2-H /DescendantFonts [3 0 R] >> endobj
3 0 obj << /Type /Font /Subtype /CIDFontType0 /CIDSystemInfo << /Registry (Adobe) /Ordering (Japan1) /Supplement 6 >> >> endobj
"#;

        let maps = font_cmaps(pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0x4e, 0x2d]), "中");
    }

    #[test]
    fn uses_embedded_truetype_cmap_when_encoding_is_missing() {
        let mut pdf = format!(
            "\
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /TrueType /FontDescriptor 3 0 R >> endobj
3 0 obj << /Type /FontDescriptor /FontName /Demo /FontFile2 4 0 R >> endobj
4 0 obj << /Length {} >>
stream
",
            DEMO_TTF.len()
        )
        .into_bytes();
        pdf.extend_from_slice(DEMO_TTF);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        let maps = font_cmaps(&pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0x41]), "A");
    }

    #[test]
    fn uses_embedded_truetype_cmap_and_cid_to_gid_map_for_type0_identity_fonts() {
        let cid_to_gid = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let mut pdf = format!(
            "\
1 0 obj << /Type /Page /Resources << /Font << /F1 2 0 R >> >> >> endobj
2 0 obj << /Type /Font /Subtype /Type0 /Encoding /Identity-H /DescendantFonts [3 0 R] >> endobj
3 0 obj << /Type /Font /Subtype /CIDFontType2 /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor 4 0 R /CIDToGIDMap 6 0 R >> endobj
4 0 obj << /Type /FontDescriptor /FontName /Demo /FontFile2 5 0 R >> endobj
5 0 obj << /Length {} >>
stream
",
            DEMO_TTF.len()
        )
        .into_bytes();
        pdf.extend_from_slice(DEMO_TTF);
        pdf.extend_from_slice(
            format!(
                "\nendstream\nendobj\n6 0 obj << /Length {} >>\nstream\n",
                cid_to_gid.len()
            )
            .as_bytes(),
        );
        pdf.extend_from_slice(&cid_to_gid);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        let maps = font_cmaps(&pdf).unwrap();

        assert_eq!(maps["F1"].decode(&[0, 5]), "A");
    }

    const DEMO_TTF: &[u8] = &[
        0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x00, 0x40, 0x00, 0x02, 0x00, 0x30, 0x63, 0x6d, 0x61,
        0x70, 0x00, 0x09, 0x00, 0x76, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x67, 0x6c,
        0x79, 0x66, 0xf1, 0xcb, 0x66, 0x98, 0x00, 0x00, 0x01, 0x34, 0x00, 0x00, 0x00, 0x5c, 0x68,
        0x65, 0x61, 0x64, 0xf2, 0x35, 0xdd, 0xf8, 0x00, 0x00, 0x00, 0x7c, 0x00, 0x00, 0x00, 0x36,
        0x68, 0x68, 0x65, 0x61, 0x06, 0x61, 0x00, 0xca, 0x00, 0x00, 0x00, 0xb4, 0x00, 0x00, 0x00,
        0x24, 0x68, 0x6d, 0x74, 0x78, 0x04, 0x74, 0x00, 0x6a, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00,
        0x00, 0x08, 0x6c, 0x6f, 0x63, 0x61, 0x00, 0x2e, 0x00, 0x14, 0x00, 0x00, 0x01, 0x2c, 0x00,
        0x00, 0x00, 0x06, 0x6d, 0x61, 0x78, 0x70, 0x00, 0x05, 0x00, 0x0b, 0x00, 0x00, 0x00, 0xd8,
        0x00, 0x00, 0x00, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xf5, 0x9c, 0x29,
        0x44, 0x5f, 0x0f, 0x3c, 0xf5, 0x00, 0x02, 0x03, 0xe8, 0x00, 0x00, 0x00, 0x00, 0xb4, 0x92,
        0xf4, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0x2f, 0xa6, 0x5c, 0x00, 0x06, 0x00, 0x00, 0x02,
        0x58, 0x02, 0xbc, 0x00, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x04, 0x00, 0xfe, 0x70, 0x00, 0x00, 0x02, 0x58, 0x00, 0x06, 0xff,
        0xff, 0x02, 0x58, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x0b, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x58, 0x00, 0x64, 0x02, 0x1c, 0x00,
        0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x04,
        0x00, 0x20, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x41, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x41, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x14, 0x00, 0x2e, 0x00, 0x00, 0x00, 0x02, 0x00, 0x64, 0x00, 0x00, 0x02,
        0x58, 0x02, 0xbc, 0x00, 0x03, 0x00, 0x07, 0x00, 0x00, 0x33, 0x11, 0x21, 0x11, 0x25, 0x21,
        0x11, 0x21, 0x64, 0x01, 0xf4, 0xfe, 0x34, 0x01, 0xa4, 0xfe, 0x5c, 0x02, 0xbc, 0xfd, 0x44,
        0x28, 0x02, 0x6c, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x02, 0x1d, 0x02, 0x90, 0x00, 0x02,
        0x00, 0x0a, 0x00, 0x00, 0x13, 0x33, 0x03, 0x01, 0x13, 0x33, 0x13, 0x23, 0x27, 0x23, 0x07,
        0xad, 0xc4, 0x63, 0xfe, 0xf8, 0xda, 0x60, 0xdd, 0x59, 0x3e, 0xef, 0x42, 0x01, 0x0b, 0x01,
        0x40, 0xfd, 0xb5, 0x02, 0x90, 0xfd, 0x70, 0xc8, 0xc8, 0x00,
    ];
}
