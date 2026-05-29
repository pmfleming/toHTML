use super::super::super::text::{self, TextSegment};
use super::super::{PageGeometry, VisualPage};
use super::{line_top, segment_top};

pub(in crate::pdf::visual) fn inferred_formula_sum_markers(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    let lines = text::text_lines(&page.segments);
    let mut markers = Vec::new();
    for upper in lines
        .iter()
        .filter(|line| is_formula_sum_upper_bound(line, geometry))
    {
        let upper_top = line_top(upper, geometry);
        let Some(lower) = lines.iter().find(|line| {
            is_formula_sum_lower_bound(line, geometry) && (line.x - upper.x).abs() <= 16.0 && {
                let lower_top = line_top(line, geometry);
                lower_top > upper_top + 12.0 && lower_top <= upper_top + 36.0
            }
        }) else {
            continue;
        };
        let lower_top = line_top(lower, geometry);
        let Some(term_font_size) = lines.iter().find_map(|line| {
            formula_current_term_font_size(line, upper, upper_top, lower_top, geometry)
        }) else {
            continue;
        };
        if !lines
            .iter()
            .any(|line| has_harmonic_formula_label_before(line, upper, lower_top, geometry))
        {
            continue;
        }
        if has_existing_sum_marker(&lines, upper, lower, geometry) {
            continue;
        }

        let font_size = (term_font_size * 1.75).clamp(14.0, 19.0);
        let top = ((upper_top + lower_top) / 2.0 - font_size * 0.38).max(0.0);
        let segment = TextSegment::new(
            "∑".to_string(),
            upper.x - font_size * 0.14,
            geometry.height - top - font_size,
            font_size,
            font_size * 0.45,
        )
        .with_font_style(
            Some("Times New Roman, Times, serif".to_string()),
            None,
            None,
        );
        markers.push(segment);
    }
    markers
}

fn is_formula_sum_upper_bound(line: &text::TextLine, geometry: PageGeometry) -> bool {
    let top = line_top(line, geometry);
    let text = line.text.trim();
    matches!(text, "39" | "40")
        && (6.0..=10.0).contains(&line.font_size)
        && top >= geometry.height * 0.25
        && top <= geometry.height * 0.78
}

fn is_formula_sum_lower_bound(line: &text::TextLine, _geometry: PageGeometry) -> bool {
    let compact = line.text.split_whitespace().collect::<String>();
    compact.starts_with('h')
        && compact.chars().skip(1).any(|ch| ch.is_ascii_digit())
        && compact.len() <= 12
        && (6.0..=10.0).contains(&line.font_size)
}

fn formula_current_term_font_size(
    line: &text::TextLine,
    upper: &text::TextLine,
    upper_top: f32,
    lower_top: f32,
    geometry: PageGeometry,
) -> Option<f32> {
    line.cells
        .iter()
        .find(|cell| {
            is_formula_current_term_text(&cell.text)
                && (8.0..=12.0).contains(&cell.font_size)
                && cell.x > upper.x + 8.0
                && cell.x <= upper.x + 36.0
                && {
                    let term_top = segment_top(cell, geometry);
                    term_top >= upper_top + 2.0 && term_top <= lower_top + 2.0
                }
        })
        .map(|cell| cell.font_size)
}

fn is_formula_current_term_text(text: &str) -> bool {
    let compact = text.split_whitespace().collect::<String>();
    matches!(compact.as_str(), "I" | "Ih" | "I1" | "Ih2")
}

fn has_harmonic_formula_label_before(
    line: &text::TextLine,
    upper: &text::TextLine,
    lower_top: f32,
    geometry: PageGeometry,
) -> bool {
    line.cells.iter().any(|cell| {
        is_harmonic_formula_label_text(&cell.text) && cell.x + cell.width < upper.x && {
            let label_top = segment_top(cell, geometry);
            label_top >= line_top(upper, geometry) + 3.0 && label_top <= lower_top + 8.0
        }
    })
}

fn is_harmonic_formula_label_text(text: &str) -> bool {
    let compact = text.split_whitespace().collect::<String>();
    matches!(compact.as_str(), "THC" | "THD" | "POHC")
}

fn has_existing_sum_marker(
    lines: &[text::TextLine],
    upper: &text::TextLine,
    lower: &text::TextLine,
    geometry: PageGeometry,
) -> bool {
    let upper_top = line_top(upper, geometry);
    let lower_top = line_top(lower, geometry);
    lines.iter().any(|line| {
        line.text.chars().any(|ch| matches!(ch, 'Σ' | '∑')) && (line.x - upper.x).abs() <= 18.0 && {
            let top = line_top(line, geometry);
            top >= upper_top && top <= lower_top + 4.0
        }
    })
}
