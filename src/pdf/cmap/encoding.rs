use crate::pdf::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};

mod agl;
mod tables;

pub(super) fn simple_font_mappings(
    objects: &PdfObjects,
    font_dictionary: &PdfDictionary,
) -> Vec<(u8, String)> {
    let Some(encoding) = resolved_value(objects, font_dictionary.get("Encoding")) else {
        return default_font_mappings(font_dictionary);
    };

    match encoding {
        PdfValue::Name(name) => base_encoding_mappings(name),
        PdfValue::Dictionary(dictionary) => dictionary_encoding_mappings(objects, dictionary),
        _ => Vec::new(),
    }
}

fn default_font_mappings(font_dictionary: &PdfDictionary) -> Vec<(u8, String)> {
    let Some(base_font) = font_dictionary.name("BaseFont") else {
        return Vec::new();
    };
    let base_font = base_font.rsplit('+').next().unwrap_or(base_font);
    if base_font.contains("Symbol") {
        return base_encoding_mappings("SymbolEncoding");
    }
    if base_font.contains("ZapfDingbats") {
        return base_encoding_mappings("ZapfDingbatsEncoding");
    }
    if is_standard_latin_font(base_font) {
        return base_encoding_mappings("StandardEncoding");
    }
    Vec::new()
}

fn is_standard_latin_font(name: &str) -> bool {
    matches!(
        name,
        "Times-Roman"
            | "Times-Bold"
            | "Times-Italic"
            | "Times-BoldItalic"
            | "Helvetica"
            | "Helvetica-Bold"
            | "Helvetica-Oblique"
            | "Helvetica-BoldOblique"
            | "Courier"
            | "Courier-Bold"
            | "Courier-Oblique"
            | "Courier-BoldOblique"
    )
}

fn dictionary_encoding_mappings(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
) -> Vec<(u8, String)> {
    let mut mappings = dictionary
        .name("BaseEncoding")
        .map(base_encoding_mappings)
        .unwrap_or_default();

    let Some(differences) = resolved_value(objects, dictionary.get("Differences")) else {
        return mappings;
    };
    let PdfValue::Array(differences) = differences else {
        return mappings;
    };

    let mut code = 0u8;
    for difference in differences {
        match difference {
            PdfValue::Integer(value) => {
                if let Ok(next) = u8::try_from(*value) {
                    code = next;
                }
            }
            PdfValue::Name(name) => {
                mappings.retain(|(existing, _)| *existing != code);
                if let Some(unicode) = glyph_name_to_unicode(name) {
                    mappings.push((code, unicode));
                }
                code = code.saturating_add(1);
            }
            _ => {}
        }
    }

    mappings
}

fn resolved_value<'a>(
    objects: &'a PdfObjects,
    value: Option<&'a PdfValue>,
) -> Option<&'a PdfValue> {
    match value? {
        PdfValue::Reference(reference) => Some(&objects.get(*reference)?.value),
        value => Some(value),
    }
}

fn base_encoding_mappings(name: &str) -> Vec<(u8, String)> {
    (0u8..=255)
        .filter_map(|code| tables::base_encoding_char(name, code).map(|ch| (code, ch.to_string())))
        .collect()
}

fn glyph_name_to_unicode(name: &str) -> Option<String> {
    let name = name.split_once('.').map_or(name, |(base, _)| base);
    if name.contains('_') {
        let text = name
            .split('_')
            .map(glyph_name_to_unicode)
            .collect::<Option<Vec<_>>>()?
            .join("");
        return (!text.is_empty()).then_some(text);
    }
    if let Some(text) = unicode_name_sequence(name) {
        return Some(text);
    }

    tables::glyph_name_to_unicode(name)
}

pub(super) fn glyph_name_unicode(name: &str) -> Option<String> {
    glyph_name_to_unicode(name)
}

fn unicode_name_sequence(name: &str) -> Option<String> {
    if let Some(hex) = name.strip_prefix("uni") {
        if hex.len() >= 4 && hex.len() % 4 == 0 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return hex
                .as_bytes()
                .chunks_exact(4)
                .map(|chunk| {
                    let hex = std::str::from_utf8(chunk).ok()?;
                    char::from_u32(u32::from_str_radix(hex, 16).ok()?)
                })
                .collect();
        }
    }
    if let Some(hex) = name.strip_prefix('u') {
        if (4..=6).contains(&hex.len()) && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return char::from_u32(u32::from_str_radix(hex, 16).ok()?).map(|ch| ch.to_string());
        }
    }
    None
}
