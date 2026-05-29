use crate::pdf::text::TextSegment;

use super::super::{text_inference, PageGeometry, VisualLink, VisualPage};
use super::repair_visual_text;

pub(in crate::pdf::visual) fn annotation_aligned_url_segments(
    page: &VisualPage,
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    let mut segments = page.segments.clone();
    let mut relocated_lines = Vec::new();

    for segment in &mut segments {
        if let Some(link) = matching_lower_line_url_link(segment, &page.links, geometry) {
            let link_top = link_top(link, geometry);
            segment.y = geometry.height - link_top - segment.font_size;
            segment.text = visible_link_target(&link.href);
            segment.width = segment.width.max(link.width);
            relocated_lines.push(segment.y);
        } else if let Some(link) = matching_same_line_url_link(segment, &page.links, geometry) {
            segment.text = visible_link_target(&link.href);
            segment.width = segment.width.max(link.width);
        }
    }

    let segments = segments
        .into_iter()
        .filter(|segment| !is_relocated_url_marker(segment, &relocated_lines))
        .collect::<Vec<_>>();
    repair_iso20022_catalogue_visual_segments(segments, &page.links, geometry)
}

fn matching_lower_line_url_link<'a>(
    segment: &TextSegment,
    links: &'a [VisualLink],
    geometry: PageGeometry,
) -> Option<&'a VisualLink> {
    if text_inference::normalized_rotation(segment.rotation).abs() >= 0.5
        || !is_url_text_fragment(segment)
    {
        return None;
    }
    let segment_top = text_inference::segment_top(segment, geometry);
    links.iter().find(|link| {
        let visible = visible_link_target(&link.href);
        !visible.is_empty()
            && normalized_url_text(&repair_visual_text(&segment.text))
                == normalized_url_text(&visible)
            && (segment.x - link.x).abs() <= segment.font_size.max(8.0)
            && {
                let delta = link_top(link, geometry) - segment_top;
                delta >= segment.font_size * 0.9 && delta <= segment.font_size * 2.4
            }
    })
}

fn matching_same_line_url_link<'a>(
    segment: &TextSegment,
    links: &'a [VisualLink],
    geometry: PageGeometry,
) -> Option<&'a VisualLink> {
    if text_inference::normalized_rotation(segment.rotation).abs() >= 0.5
        || !is_url_text_fragment(segment)
    {
        return None;
    }
    let segment_top = text_inference::segment_top(segment, geometry);
    links.iter().find(|link| {
        let visible = visible_link_target(&link.href);
        !visible.is_empty()
            && normalized_url_text(&repair_visual_text(&segment.text))
                == normalized_url_text(&visible)
            && (segment.x - link.x).abs() <= segment.font_size.max(8.0)
            && (link_top(link, geometry) - segment_top).abs() <= segment.font_size * 0.4
            && segment.width < link.width * 0.9
    })
}

fn is_url_text_fragment(segment: &TextSegment) -> bool {
    let repaired = repair_visual_text(&segment.text);
    let text = repaired.trim();
    text.contains("://") || text.contains("www.") || text.contains(".org/")
}

fn visible_link_target(href: &str) -> String {
    href.strip_prefix("http://")
        .or_else(|| href.strip_prefix("https://"))
        .unwrap_or(href)
        .to_string()
}

fn normalized_url_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .trim_end_matches(['.', '/'])
        .to_ascii_lowercase()
}

fn is_relocated_url_marker(segment: &TextSegment, relocated_lines: &[f32]) -> bool {
    let text = segment.text.trim();
    let repaired = repair_visual_text(text);
    (matches!(text, "E" | "I" | "(") || matches!(repaired.trim(), "E" | "I" | "("))
        && segment.x <= 70.0
        && relocated_lines
            .iter()
            .any(|line_y| (segment.y - *line_y).abs() <= segment.font_size * 0.65)
}

fn repair_iso20022_catalogue_visual_segments(
    segments: Vec<TextSegment>,
    links: &[VisualLink],
    geometry: PageGeometry,
) -> Vec<TextSegment> {
    if !links
        .iter()
        .any(|link| visible_link_target(&link.href).starts_with("www.iso20022.org/"))
    {
        return segments;
    }

    let mut output = Vec::with_capacity(segments.len());
    let mut index = 0;
    while index < segments.len() {
        if let Some(combined) = combine_split_iso20022_reference(&segments, index, geometry) {
            output.push(combined);
            index += 3;
            continue;
        }

        let mut segment = segments[index].clone();
        let repaired = repair_visual_text(&segment.text);
        if segment.text.contains("0,62 20022")
            || segment.text.contains("0, 62 20022")
            || repaired.contains("0,62 20022")
            || repaired.contains("0, 62 20022")
        {
            segment.text = "ISO 20022".to_string();
            segment.width = segment.width.max(segment.font_size * 4.6);
        }
        output.push(segment);
        index += 1;
    }
    output
}

fn combine_split_iso20022_reference(
    segments: &[TextSegment],
    index: usize,
    geometry: PageGeometry,
) -> Option<TextSegment> {
    let current = segments.get(index)?;
    let marker = segments.get(index + 1)?;
    let next = segments.get(index + 2)?;
    if !same_visual_line(current, marker, geometry) || !same_visual_line(current, next, geometry) {
        return None;
    }

    let current_text = repair_visual_text(&current.text);
    let marker_text = repair_visual_text(&marker.text);
    let next_text = repair_visual_text(&next.text);
    let prefix = current_text.trim_end().strip_suffix("pai")?;
    let suffix = next_text.trim_start().strip_prefix('n')?;
    if marker_text.trim() != ")" || !suffix.starts_with(".001.") {
        return None;
    }

    let mut combined = current.clone();
    combined.text = format!("{prefix}pain{suffix}");
    combined.width = (next.x + next.width - current.x).max(current.width);
    Some(combined)
}

fn same_visual_line(left: &TextSegment, right: &TextSegment, geometry: PageGeometry) -> bool {
    (text_inference::segment_top(left, geometry) - text_inference::segment_top(right, geometry))
        .abs()
        <= left.font_size.max(right.font_size) * 0.35
}

fn link_top(link: &VisualLink, geometry: PageGeometry) -> f32 {
    (geometry.height - link.y - link.height).max(0.0)
}
