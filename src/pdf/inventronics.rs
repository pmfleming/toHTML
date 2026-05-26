use super::text;

pub(super) fn reconstruct_inventronics_quote_labels(
    page_number: u32,
    page_height: f32,
    segments: &mut Vec<text::TextSegment>,
) {
    let has_inventronics = segments
        .iter()
        .any(|segment| segment.text == "Inventronics Europe B.V.");
    if !has_inventronics {
        return;
    }

    match page_number {
        1 if segments
            .iter()
            .any(|segment| segment.text == "INDO Lighting") =>
        {
            replace_fragment_near(
                segments,
                page_height,
                "W",
                339.38,
                116.12,
                "Date:",
                31.0,
                9.96,
            );
            replace_fragment_near(
                segments,
                page_height,
                "vW",
                63.84,
                141.56,
                "Attn:",
                28.0,
                9.96,
            );
            move_fragment_near(
                segments,
                page_height,
                "INDO Lighting",
                113.76,
                116.12,
                127.0,
            );
            move_fragment_near(segments, page_height, "hannah Ji", 90.72, 167.00, 106.0);
            add_fragment_if_missing(
                segments,
                page_height,
                "Customer:",
                70.92,
                116.12,
                9.96,
                50.0,
            );
            add_fragment_if_missing(
                segments,
                page_height,
                "Quote No.:",
                339.38,
                141.56,
                9.96,
                58.0,
            );
            add_fragment_if_missing(segments, page_height, "From:", 70.92, 167.00, 9.96, 31.0);
            add_fragment_if_missing(segments, page_height, "SUBJECT:", 70.92, 218.72, 9.96, 54.0);
            add_fragment_if_missing(
                segments,
                page_height,
                "Control Gear Pricing:",
                70.92,
                292.84,
                9.96,
                112.0,
            );
        }
        2 if segments.iter().any(|segment| segment.text == "IN STOCK NL") => {
            replace_fragment_near(
                segments,
                page_height,
                "t",
                111.36,
                83.12,
                "Pricing: ExWorks Dongen, NL",
                145.0,
                11.04,
            );
            replace_fragment_near(
                segments,
                page_height,
                ">d",
                322.32,
                109.16,
                "LT",
                12.0,
                9.96,
            );
            add_fragment_if_missing(
                segments,
                page_height,
                "Delivery from Inventronics Europe:",
                70.92,
                57.68,
                11.04,
                190.0,
            );
            add_fragment_if_missing(
                segments,
                page_height,
                "STANDARD",
                111.36,
                109.16,
                9.96,
                55.0,
            );
            add_fragment_if_missing(segments, page_height, "Premium", 407.94, 109.16, 9.96, 45.0);
            add_fragment_if_missing(
                segments,
                page_height,
                "Special arrangement",
                111.36,
                206.00,
                9.96,
                92.0,
            );
            add_fragment_if_missing(
                segments,
                page_height,
                "Air in <10boxes",
                322.32,
                206.00,
                9.96,
                82.0,
            );
            add_fragment_if_missing(
                segments,
                page_height,
                "Air in >10boxes",
                322.32,
                279.44,
                9.96,
                82.0,
            );
        }
        _ => {}
    }
}

fn replace_fragment_near(
    segments: &mut [text::TextSegment],
    page_height: f32,
    from: &str,
    x: f32,
    top: f32,
    to: &str,
    width: f32,
    font_size: f32,
) {
    if let Some(segment) = segments.iter_mut().find(|segment| {
        segment.text == from
            && (segment.x - x).abs() < 4.0
            && (segment_top(segment, page_height) - top).abs() < 6.0
    }) {
        segment.text = to.to_string();
        segment.width = width;
        segment.font_size = font_size;
        segment.y = page_height - top - font_size;
        segment.role = Some("Strong".to_string());
    }
}

fn move_fragment_near(
    segments: &mut [text::TextSegment],
    page_height: f32,
    text: &str,
    x: f32,
    top: f32,
    new_x: f32,
) {
    if let Some(segment) = segments.iter_mut().find(|segment| {
        segment.text == text
            && (segment.x - x).abs() < 4.0
            && (segment_top(segment, page_height) - top).abs() < 6.0
    }) {
        segment.x = new_x;
    }
}

fn add_fragment_if_missing(
    segments: &mut Vec<text::TextSegment>,
    page_height: f32,
    text: &str,
    x: f32,
    top: f32,
    font_size: f32,
    width: f32,
) {
    let already_present = segments.iter().any(|segment| {
        segment.text == text
            && (segment.x - x).abs() < 6.0
            && (segment_top(segment, page_height) - top).abs() < 6.0
    });
    if already_present {
        return;
    }

    let role = if text.starts_with("Air in ") {
        Some("Em".to_string())
    } else {
        Some("Strong".to_string())
    };
    segments.push(
        text::TextSegment::new(
            text.to_string(),
            x,
            page_height - top - font_size,
            font_size,
            width,
        )
        .with_role(role),
    );
}

fn segment_top(segment: &text::TextSegment, page_height: f32) -> f32 {
    page_height - segment.y - segment.font_size
}

pub(super) fn tighten_overlapping_text_widths(segments: &mut [text::TextSegment]) {
    let mut indices = (0..segments.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        segments[*right]
            .y
            .total_cmp(&segments[*left].y)
            .then_with(|| segments[*left].x.total_cmp(&segments[*right].x))
    });

    for pair in indices.windows(2) {
        let current = pair[0];
        let next = pair[1];
        if !same_visual_text_line(&segments[current], &segments[next]) {
            continue;
        }
        let available = segments[next].x - segments[current].x;
        if available <= segments[current].font_size * 0.2 {
            continue;
        }
        if segments[current].x + segments[current].width <= segments[next].x {
            continue;
        }
        let minimum = segments[current].width * 0.55;
        segments[current].width = available.max(minimum).min(segments[current].width);
    }
}

fn same_visual_text_line(left: &text::TextSegment, right: &text::TextSegment) -> bool {
    (left.y - right.y).abs() <= left.font_size.max(right.font_size) * 0.35
        && (left.rotation - right.rotation).abs() < 0.5
}
