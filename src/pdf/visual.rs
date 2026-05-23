use super::graphics::RectShape;
use super::text::TextSegment;

#[derive(Debug, Clone)]
pub(super) struct VisualPage {
    pub page_number: u32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub segments: Vec<TextSegment>,
    pub shapes: Vec<RectShape>,
}

pub(super) fn render_pages(pages: &[VisualPage]) -> Option<String> {
    let mut html = String::new();
    let mut emitted = false;

    for page in pages
        .iter()
        .filter(|page| !page.segments.is_empty() || !page.shapes.is_empty())
    {
        emitted = true;
        let geometry = PageGeometry::from_page(page);
        html.push_str("    <section class=\"pdf-recreated-page\" data-page=\"");
        html.push_str(&page.page_number.to_string());
        html.push_str("\" style=\"width:");
        push_pt(&mut html, geometry.width);
        html.push_str(";height:");
        push_pt(&mut html, geometry.height);
        html.push_str("\">\n");

        for shape in &page.shapes {
            render_shape(&mut html, shape, geometry);
        }
        for segment in &page.segments {
            render_fragment(&mut html, segment, geometry);
        }

        html.push_str("    </section>\n");
    }

    emitted.then_some(html)
}

#[derive(Debug, Clone, Copy)]
struct PageGeometry {
    width: f32,
    height: f32,
    min_x: f32,
}

impl PageGeometry {
    fn from_page(page: &VisualPage) -> Self {
        let max_x = page
            .segments
            .iter()
            .map(|segment| segment.x + segment.width.max(0.0))
            .chain(
                page.shapes
                    .iter()
                    .map(|shape| shape.x + shape.width.max(0.0)),
            )
            .fold(0.0_f32, f32::max);
        let max_y = page
            .segments
            .iter()
            .map(|segment| segment.y + segment.font_size.max(0.0))
            .chain(
                page.shapes
                    .iter()
                    .map(|shape| shape.y + shape.height.max(0.0)),
            )
            .fold(0.0_f32, f32::max);
        let min_x = page
            .segments
            .iter()
            .map(|segment| segment.x)
            .chain(page.shapes.iter().map(|shape| shape.x))
            .fold(f32::INFINITY, f32::min)
            .min(0.0);

        Self {
            width: page
                .width
                .unwrap_or_else(|| (max_x + 72.0).max(612.0))
                .ceil(),
            height: page
                .height
                .unwrap_or_else(|| (max_y + 72.0).max(792.0))
                .ceil(),
            min_x,
        }
    }
}

fn render_shape(html: &mut String, shape: &RectShape, geometry: PageGeometry) {
    let left = (shape.x - geometry.min_x).max(0.0);
    let top = (geometry.height - shape.y - shape.height).max(0.0);
    let width = shape.width.max(0.0);
    let height = shape.height.max(0.0);
    if left > geometry.width || top > geometry.height || width < 0.25 || height < 0.25 {
        return;
    }

    html.push_str("      <div class=\"pdf-shape\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";width:");
    push_pt(html, width.min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, height.min(geometry.height - top));
    if let Some(fill) = &shape.fill {
        html.push_str(";background:");
        push_css_color(html, fill);
    }
    if let Some(stroke) = &shape.stroke {
        html.push_str(";border:0.75pt solid ");
        push_css_color(html, stroke);
    }
    html.push_str("\"></div>\n");
}

fn render_fragment(html: &mut String, segment: &TextSegment, geometry: PageGeometry) {
    let font_size = segment.font_size.clamp(4.0, 48.0);
    let width = segment.width.max(font_size * 0.5);
    let left = (segment.x - geometry.min_x).max(0.0);
    let top = (geometry.height - segment.y - font_size).max(0.0);
    let rotation = normalized_rotation(segment.rotation);
    if left > geometry.width || top > geometry.height {
        return;
    }

    html.push_str("      <span class=\"pdf-text-fragment\" style=\"left:");
    push_pt(html, left);
    html.push_str(";top:");
    push_pt(html, top);
    html.push_str(";font-size:");
    push_pt(html, font_size);
    html.push_str(";width:");
    push_pt(html, width.min(geometry.width - left));
    html.push_str(";height:");
    push_pt(html, font_size * 1.12);
    if rotation.abs() >= 0.5 {
        html.push_str(";transform:rotate(");
        push_number(html, rotation);
        html.push_str("deg)");
    }
    html.push_str("\">");
    push_escaped(html, &segment.text);
    html.push_str("</span>\n");
}

fn normalized_rotation(rotation: f32) -> f32 {
    let mut rotation = rotation % 360.0;
    if rotation > 180.0 {
        rotation -= 360.0;
    } else if rotation < -180.0 {
        rotation += 360.0;
    }
    rotation
}

fn push_pt(html: &mut String, value: f32) {
    push_number(html, value.max(0.0));
    html.push_str("pt");
}

fn push_number(html: &mut String, value: f32) {
    html.push_str(&format!("{value:.2}"));
}

fn push_css_color(html: &mut String, value: &str) {
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    {
        html.push_str(value);
    }
}

fn push_escaped(html: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            _ => html.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_positioned_text_fragments_without_embedding_pdf() {
        let html = render_pages(&[VisualPage {
            page_number: 2,
            width: Some(200.0),
            height: Some(300.0),
            segments: vec![TextSegment::new(
                "Hello <PDF>".to_string(),
                20.0,
                240.0,
                12.0,
                60.0,
            )],
            shapes: Vec::new(),
        }])
        .unwrap();

        assert!(html.contains("class=\"pdf-recreated-page\" data-page=\"2\""));
        assert!(html.contains("left:20.00pt;top:48.00pt;font-size:12.00pt"));
        assert!(html.contains("Hello &lt;PDF&gt;"));
        assert!(!html.contains("application/pdf"));
    }

    #[test]
    fn preserves_fragment_rotation() {
        let html = render_pages(&[VisualPage {
            page_number: 1,
            width: Some(200.0),
            height: Some(300.0),
            segments: vec![
                TextSegment::new("Sideways".to_string(), 20.0, 240.0, 12.0, 60.0)
                    .with_rotation(90.0),
            ],
            shapes: Vec::new(),
        }])
        .unwrap();

        assert!(html.contains("transform:rotate(90.00deg)"));
    }

    #[test]
    fn renders_shapes_before_text() {
        let html = render_pages(&[VisualPage {
            page_number: 1,
            width: Some(200.0),
            height: Some(300.0),
            segments: vec![TextSegment::new(
                "Cell".to_string(),
                20.0,
                240.0,
                12.0,
                24.0,
            )],
            shapes: vec![RectShape {
                x: 10.0,
                y: 220.0,
                width: 100.0,
                height: 30.0,
                fill: Some("#eeeeee".to_string()),
                stroke: Some("#000000".to_string()),
            }],
        }])
        .unwrap();

        assert!(html.contains("class=\"pdf-shape\""));
        assert!(html.find("pdf-shape") < html.find("pdf-text-fragment"));
        assert!(html.contains("background:#eeeeee"));
        assert!(html.contains("border:0.75pt solid #000000"));
    }
}
