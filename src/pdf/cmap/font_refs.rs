use std::collections::HashMap;

use super::CMap;
use crate::pdf::streams;

pub fn font_cmaps(bytes: &[u8]) -> Result<HashMap<String, CMap>, crate::ConvertError> {
    let mut maps = HashMap::new();
    for (font_name, font_object) in font_references(bytes) {
        if let Some(cmap) = cmap_for_font(bytes, font_object)? {
            maps.insert(font_name, cmap);
        }
    }
    Ok(maps)
}

fn cmap_for_font(bytes: &[u8], font_object: u32) -> Result<Option<CMap>, crate::ConvertError> {
    let Some(body) = streams::object_body(bytes, font_object) else {
        return Ok(None);
    };
    let Some(cmap_object) = to_unicode_ref(body) else {
        return Ok(None);
    };
    let Some(cmap_stream) = streams::stream_data_for_object(bytes, cmap_object)? else {
        return Ok(None);
    };
    Ok(Some(CMap::parse(&cmap_stream)))
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
}
