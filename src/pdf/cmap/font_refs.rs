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
    pdf_tokens(bytes)
        .windows(4)
        .filter_map(|window| font_reference(window))
        .collect()
}

fn font_reference(window: &[String]) -> Option<(String, u32)> {
    if !is_font_reference(window) {
        return None;
    }

    Some((
        window[0].trim_start_matches('/').to_string(),
        window[1].parse().ok()?,
    ))
}

fn is_font_reference(window: &[String]) -> bool {
    window[0].starts_with("/F") && window[2] == "0" && window[3] == "R"
}

fn to_unicode_ref(body: &[u8]) -> Option<u32> {
    pdf_tokens(body)
        .windows(4)
        .find_map(|window| to_unicode_window(window))
}

fn to_unicode_window(window: &[String]) -> Option<u32> {
    (window[0] == "/ToUnicode" && window[2] == "0" && window[3] == "R")
        .then(|| window[1].parse().ok())
        .flatten()
}

fn pdf_tokens(bytes: &[u8]) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() || is_token_delimiter(byte) => index += 1,
            b'/' => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !is_token_delimiter(bytes[index])
                    && bytes[index] != b'/'
                {
                    index += 1;
                }
                tokens.push(String::from_utf8_lossy(&bytes[start..index]).to_string());
            }
            _ => {
                let start = index;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !is_token_delimiter(bytes[index])
                    && bytes[index] != b'/'
                {
                    index += 1;
                }
                tokens.push(String::from_utf8_lossy(&bytes[start..index]).to_string());
            }
        }
    }
    tokens
}

fn is_token_delimiter(byte: u8) -> bool {
    matches!(byte, b'<' | b'>' | b'[' | b']' | b'(' | b')')
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
