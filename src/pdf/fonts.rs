use std::collections::HashMap;

use super::object::PdfReference;
use super::streams;

#[derive(Debug, Clone, Default)]
pub struct FontMetrics {
    first_char: u16,
    widths: Vec<f32>,
    family: Option<String>,
    bold: bool,
    italic: bool,
}

impl FontMetrics {
    pub fn text_width(&self, bytes: &[u8], font_size: f32, fallback_chars: usize) -> f32 {
        let width_units: f32 = bytes
            .iter()
            .map(|byte| self.width_for_code(u16::from(*byte)))
            .sum();

        if width_units > 0.0 {
            width_units * font_size / 1000.0
        } else {
            fallback_chars as f32 * font_size * 0.45
        }
    }

    pub fn css_family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    pub fn is_bold(&self) -> bool {
        self.bold
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }

    fn width_for_code(&self, code: u16) -> f32 {
        let Some(index) = code.checked_sub(self.first_char).map(usize::from) else {
            return 0.0;
        };
        self.widths.get(index).copied().unwrap_or_default()
    }
}

pub fn font_metrics(bytes: &[u8]) -> HashMap<String, FontMetrics> {
    let mut references = named_font_objects(bytes);
    references.extend(font_references(bytes));
    references
        .into_iter()
        .filter_map(|(name, object)| metrics_for_font(bytes, object).map(|metrics| (name, metrics)))
        .collect()
}

pub fn font_metrics_for_resources(
    bytes: &[u8],
    resources: &HashMap<String, PdfReference>,
) -> HashMap<String, FontMetrics> {
    resources
        .iter()
        .filter_map(|(name, reference)| {
            metrics_for_font(bytes, reference.object).map(|metrics| (name.clone(), metrics))
        })
        .collect()
}

fn metrics_for_font(bytes: &[u8], object: u32) -> Option<FontMetrics> {
    let body = streams::object_body(bytes, object)?;
    let first_char = streams::number_after(body, b"/FirstChar")? as u16;
    let widths = streams::object_ref_after(body, b"/Widths")
        .and_then(|widths_ref| widths_for_object(bytes, widths_ref))
        .or_else(|| widths_in_inline_array_after(body, b"/Widths"))?;
    let base_font = pdf_name_after(body, b"/BaseFont");
    let family = base_font.as_deref().and_then(css_family_for_font);
    let lowered = base_font
        .as_deref()
        .map(|name| name.to_ascii_lowercase())
        .unwrap_or_default();
    Some(FontMetrics {
        first_char,
        widths,
        family: family.map(str::to_string),
        bold: lowered.contains("bold"),
        italic: lowered.contains("italic") || lowered.contains("oblique"),
    })
}

fn font_references(bytes: &[u8]) -> HashMap<String, u32> {
    let mut references = streams::named_object_refs(bytes, "/F");
    references.extend(streams::named_object_refs(bytes, "/TT"));
    references
}

fn named_font_objects(bytes: &[u8]) -> HashMap<String, u32> {
    streams::object_ids(bytes)
        .into_iter()
        .filter_map(|id| {
            let body = streams::object_body(bytes, id)?;
            let name = pdf_name_after(body, b"/Name")?;
            Some((name, id))
        })
        .collect()
}

fn widths_for_object(bytes: &[u8], object: u32) -> Option<Vec<f32>> {
    let body = streams::object_body(bytes, object)?;
    Some(numbers_in_array(body))
}

fn widths_in_inline_array_after(bytes: &[u8], marker: &[u8]) -> Option<Vec<f32>> {
    let marker_start = bytes
        .windows(marker.len())
        .position(|window| window == marker)?;
    let array_start = bytes[marker_start + marker.len()..]
        .iter()
        .position(|byte| *byte == b'[')?
        + marker_start
        + marker.len();
    let array_end = bytes[array_start..].iter().position(|byte| *byte == b']')? + array_start + 1;
    Some(numbers_in_array(&bytes[array_start..array_end]))
}

fn css_family_for_font(name: &str) -> Option<&'static str> {
    let normalized = name
        .split_once('+')
        .map(|(_, base)| base)
        .unwrap_or(name)
        .to_ascii_lowercase();
    if normalized.contains("arial") || normalized.contains("helvetica") {
        Some("Arial, Helvetica, sans-serif")
    } else if normalized.contains("times") {
        Some("Times New Roman, Times, serif")
    } else if normalized.contains("courier") {
        Some("Courier New, Courier, monospace")
    } else {
        None
    }
}

fn pdf_name_after(bytes: &[u8], marker: &[u8]) -> Option<String> {
    let start = bytes
        .windows(marker.len())
        .position(|window| window == marker)?
        + marker.len();
    let mut index = start;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    if bytes.get(index) != Some(&b'/') {
        return None;
    }
    index += 1;
    let name_start = index;
    while index < bytes.len()
        && !bytes[index].is_ascii_whitespace()
        && !matches!(bytes[index], b'/' | b'[' | b']' | b'<' | b'>' | b'(' | b')')
    {
        index += 1;
    }
    (index > name_start).then(|| String::from_utf8_lossy(&bytes[name_start..index]).to_string())
}

fn numbers_in_array(bytes: &[u8]) -> Vec<f32> {
    let text = String::from_utf8_lossy(bytes);
    text.trim_matches(|ch| matches!(ch, '[' | ']' | '\r' | '\n' | ' '))
        .split_whitespace()
        .filter_map(|token| token.parse::<f32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_font_width_reference() {
        let pdf = br#"
1 0 obj
<</Type/Font/Name/F1/BaseFont/ABCDEF+Arial-BoldItalicMT/FirstChar 32/LastChar 33/Widths 2 0 R>>
endobj
2 0 obj
[250 500]
endobj
3 0 obj
<</Type/Font/BaseFont/ArialMT/FirstChar 32/LastChar 33/Widths[278 556]>>
endobj
4 0 obj
<</Type/Pages/Resources<</Font<</TT0 3 0 R>>>>>>
endobj
"#;

        let metrics = font_metrics(pdf);

        assert_eq!(metrics["F1"].text_width(b" !", 10.0, 2), 7.5);
        assert_eq!(
            metrics["F1"].css_family(),
            Some("Arial, Helvetica, sans-serif")
        );
        assert_eq!(
            metrics["TT0"].css_family(),
            Some("Arial, Helvetica, sans-serif")
        );
        assert!((metrics["TT0"].text_width(b" !", 10.0, 2) - 8.34).abs() < 0.001);
        assert!(metrics["F1"].is_bold());
        assert!(metrics["F1"].is_italic());
    }
}
