use std::collections::HashMap;

use super::streams;

#[derive(Debug, Clone, Default)]
pub struct FontMetrics {
    first_char: u16,
    widths: Vec<f32>,
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

fn metrics_for_font(bytes: &[u8], object: u32) -> Option<FontMetrics> {
    let body = streams::object_body(bytes, object)?;
    let first_char = streams::number_after(body, b"/FirstChar")? as u16;
    let widths_ref = streams::object_ref_after(body, b"/Widths")?;
    let widths = widths_for_object(bytes, widths_ref)?;
    Some(FontMetrics { first_char, widths })
}

fn font_references(bytes: &[u8]) -> HashMap<String, u32> {
    streams::named_object_refs(bytes, "/F")
}

fn named_font_objects(bytes: &[u8]) -> HashMap<String, u32> {
    streams::object_ids(bytes)
        .into_iter()
        .filter_map(|id| {
            let body = streams::object_body(bytes, id)?;
            let name = streams::font_name_after(body, b"/Name")?;
            Some((name, id))
        })
        .collect()
}

fn widths_for_object(bytes: &[u8], object: u32) -> Option<Vec<f32>> {
    let body = streams::object_body(bytes, object)?;
    Some(numbers_in_array(body))
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
<</Type/Font/Name/F1/FirstChar 32/LastChar 33/Widths 2 0 R>>
endobj
2 0 obj
[250 500]
endobj
"#;

        let metrics = font_metrics(pdf);

        assert_eq!(metrics["F1"].text_width(b" !", 10.0, 2), 7.5);
    }
}
