use super::*;

mod layout_lines;
mod path_order;
mod positioning;
mod prose_repair;
mod render_order;

fn page(page_number: u32, width: f32, height: f32, segments: Vec<TextSegment>) -> VisualPage {
    VisualPage {
        page_number,
        width: Some(width),
        height: Some(height),
        segments,
        shapes: Vec::new(),
        images: Vec::new(),
        paths: Vec::new(),
        links: Vec::new(),
    }
}

fn segment(text: impl Into<String>, x: f32, y: f32, size: f32, width: f32) -> TextSegment {
    TextSegment::new(text.into(), x, y, size, width)
}
