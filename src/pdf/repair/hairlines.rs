use super::super::graphics::RectShape;

pub(in crate::pdf) fn remove_redundant_header_hairlines(
    page_height: f32,
    shapes: &mut Vec<RectShape>,
) {
    let original = std::mem::take(shapes);
    *shapes = original
        .iter()
        .enumerate()
        .filter_map(|(index, shape)| {
            (!is_redundant_header_hairline(index, shape, &original, page_height))
                .then(|| shape.clone())
        })
        .collect();
}

fn is_redundant_header_hairline(
    index: usize,
    candidate: &RectShape,
    shapes: &[RectShape],
    page_height: f32,
) -> bool {
    if !is_dark_horizontal_hairline(candidate) {
        return false;
    }

    let candidate_top = shape_top(candidate, page_height);
    if candidate_top > page_height * 0.16 || candidate.width > 180.0 {
        return false;
    }

    shapes.iter().enumerate().any(|(other_index, other)| {
        other_index != index
            && is_dark_horizontal_hairline(other)
            && is_long_header_rule_above(candidate, candidate_top, other, page_height)
    })
}

fn is_long_header_rule_above(
    candidate: &RectShape,
    candidate_top: f32,
    rule: &RectShape,
    page_height: f32,
) -> bool {
    let rule_top = shape_top(rule, page_height);
    let gap = candidate_top - rule_top;
    if !(6.0..=24.0).contains(&gap) {
        return false;
    }
    if rule.width < candidate.width * 3.5 || rule.width < 320.0 {
        return false;
    }

    let rule_right = rule.x + rule.width;
    let candidate_right = candidate.x + candidate.width;
    candidate.x >= rule.x + rule.width * 0.75
        && candidate_right <= rule_right + 80.0
        && candidate.x <= rule_right + 8.0
}

fn is_dark_horizontal_hairline(shape: &RectShape) -> bool {
    shape.stroke.is_none()
        && shape.fill.as_deref().is_some_and(is_dark_color)
        && shape.width >= 40.0
        && shape.height <= 1.5
}

fn is_dark_color(color: &str) -> bool {
    matches!(color, "#000" | "#000000") || hex_luminance(color).is_some_and(|value| value < 48)
}

fn hex_luminance(color: &str) -> Option<u32> {
    let color = color.strip_prefix('#')?;
    if color.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&color[0..2], 16).ok()? as u32;
    let green = u8::from_str_radix(&color[2..4], 16).ok()? as u32;
    let blue = u8::from_str_radix(&color[4..6], 16).ok()? as u32;
    Some((red * 299 + green * 587 + blue * 114) / 1000)
}

fn shape_top(shape: &RectShape, page_height: f32) -> f32 {
    page_height - shape.y - shape.height
}
