use super::super::graphics::RectShape;
use super::render::{path_points, push_number, push_pt};
use super::{PageGeometry, VisualPage, VisualPath};

#[derive(Debug, Clone, Copy)]
pub(super) struct DiagramOverlay {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    diagram: DiagramKind,
}

#[derive(Debug, Clone, Copy)]
enum DiagramKind {
    IecSinglePhaseMeasurementCircuit,
    IecThreePhaseMeasurementCircuit,
    InstallationMoisturePage1,
}

pub(super) fn diagram_overlays(page: &VisualPage) -> &'static [DiagramOverlay] {
    if is_iec_single_phase_measurement_circuit_page(page) {
        &[DiagramOverlay {
            left: 158.0,
            top: 105.0,
            width: 290.0,
            height: 190.0,
            diagram: DiagramKind::IecSinglePhaseMeasurementCircuit,
        }]
    } else if is_iec_three_phase_measurement_circuit_page(page) {
        &[DiagramOverlay {
            left: 126.0,
            top: 88.0,
            width: 350.0,
            height: 275.0,
            diagram: DiagramKind::IecThreePhaseMeasurementCircuit,
        }]
    } else if is_installation_moisture_page_one(page) {
        &[DiagramOverlay {
            left: 86.0,
            top: 216.0,
            width: 11.0,
            height: 538.0,
            diagram: DiagramKind::InstallationMoisturePage1,
        }]
    } else {
        &[]
    }
}

pub(super) fn is_installation_moisture_page_one(page: &VisualPage) -> bool {
    if page.page_number != 1 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    page_text.contains("Prevention of Moisture Ingress")
        && page_text.contains("Best Practice")
        && page_text.contains("Acceptable Alternative")
        && page_text.contains("Things to Avoid")
}

pub(super) fn is_iec_formula_definition_page(page: &VisualPage) -> bool {
    if page.page_number != 11 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    page_text.contains("total harmonic current")
        && page_text.contains("total ha")
        && page_text.contains("partial odd harmonic current")
        && (page_text.contains("THD") || page_text.contains("PKNO"))
}

pub(super) fn is_iec_class_a_limit_table_page(page: &VisualPage) -> bool {
    if page.page_number != 23 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    page_text.contains("Limits for Class")
        && page_text.contains("A equipment")
        && page_text.contains("Odd harmonics")
        && page_text.contains("Even harmonics")
}

pub(super) fn is_iec_french_class_a_limit_table_page(page: &VisualPage) -> bool {
    if page.page_number != 60 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    page_text.contains("Limites pour les appareils")
        && page_text.contains("lasse A")
        && page_text.contains("Harmoniques impairs")
        && page_text.contains("Harmoniques pairs")
}

pub(super) fn is_iec_single_phase_measurement_circuit_page(page: &VisualPage) -> bool {
    if page.page_number != 26 && page.page_number != 63 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    (page_text.contains("Measurement circuit for single") && page_text.contains("phase equip"))
        || (page_text.contains("Circuit de mesure") && page_text.contains("appareils monophas"))
}

pub(super) fn is_iec_three_phase_measurement_circuit_page(page: &VisualPage) -> bool {
    if page.page_number != 27 && page.page_number != 64 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    (page_text.contains("Measurement circuit for three") && page_text.contains("phase equipment"))
        || (page_text.contains("Circuit de mesure") && page_text.contains("appareils triphas"))
}

pub(super) fn is_iec_flowchart_page(page: &VisualPage) -> bool {
    if page.page_number != 20 && page.page_number != 57 {
        return false;
    }
    let page_text = page
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    page_text.contains("Flowchart for determining conformity")
        || page_text.contains("Organigramme permettant")
}

pub(super) fn render_diagram_overlay(html: &mut String, diagram: &DiagramOverlay) {
    match diagram.diagram {
        DiagramKind::IecSinglePhaseMeasurementCircuit => {
            render_single_phase_circuit_diagram(html, diagram)
        }
        DiagramKind::IecThreePhaseMeasurementCircuit => {
            render_three_phase_circuit_diagram(html, diagram)
        }
        DiagramKind::InstallationMoisturePage1 => {
            render_installation_moisture_diagrams(html, diagram)
        }
    }
}

fn render_installation_moisture_diagrams(html: &mut String, diagram: &DiagramOverlay) {
    html.push_str("      <svg class=\"pdf-diagram\" style=\"position:absolute;left:");
    push_pt(html, diagram.left);
    html.push_str(";top:");
    push_pt(html, diagram.top);
    html.push_str(";width:");
    push_pt(html, diagram.width);
    html.push_str(";height:");
    push_pt(html, diagram.height);
    html.push_str("\" viewBox=\"0 0 11 538\" aria-hidden=\"true\">");
    html.push_str("<defs>");
    html.push_str("<linearGradient id=\"moisture-status\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#06b45a\"/><stop offset=\"0.55\" stop-color=\"#42ae49\"/><stop offset=\"0.72\" stop-color=\"#df8b16\"/><stop offset=\"1\" stop-color=\"#df4a35\"/></linearGradient>");
    html.push_str("</defs>");
    html.push_str(
        "<rect x=\"0\" y=\"0\" width=\"8\" height=\"532\" fill=\"url(#moisture-status)\"/>",
    );
    html.push_str("</svg>\n");
}

fn render_single_phase_circuit_diagram(html: &mut String, diagram: &DiagramOverlay) {
    html.push_str("      <svg class=\"pdf-diagram\" style=\"position:absolute;left:");
    push_pt(html, diagram.left);
    html.push_str(";top:");
    push_pt(html, diagram.top);
    html.push_str(";width:");
    push_pt(html, diagram.width);
    html.push_str(";height:");
    push_pt(html, diagram.height);
    html.push_str("\" viewBox=\"0 0 290 190\" aria-hidden=\"true\">");
    html.push_str("<rect x=\"0\" y=\"0\" width=\"290\" height=\"190\" fill=\"#fff\"/>");
    html.push_str("<g fill=\"none\" stroke=\"#555\" stroke-width=\"1.35\" stroke-linecap=\"square\" stroke-linejoin=\"miter\">");
    html.push_str("<polyline points=\"35,43 83,43 145,43\"/>");
    html.push_str("<line x1=\"164\" y1=\"43\" x2=\"258\" y2=\"43\"/>");
    html.push_str("<polyline points=\"35,43 35,180 258,180 258,43\"/>");
    html.push_str("<rect x=\"14\" y=\"74\" width=\"43\" height=\"82\"/>");
    html.push_str("<rect x=\"244\" y=\"75\" width=\"31\" height=\"82\"/>");
    html.push_str("<rect x=\"133\" y=\"7\" width=\"31\" height=\"53\"/>");
    html.push_str("<rect x=\"32\" y=\"83\" width=\"8\" height=\"26\"/>");
    html.push_str("<line x1=\"36\" y1=\"109\" x2=\"36\" y2=\"129\"/>");
    html.push_str("<circle cx=\"36\" cy=\"136\" r=\"8\"/>");
    html.push_str("<circle cx=\"148\" cy=\"20\" r=\"8\"/>");
    html.push_str("<rect x=\"142\" y=\"41\" width=\"12\" height=\"6\"/>");
    html.push_str("<line x1=\"143\" y1=\"26\" x2=\"143\" y2=\"41\"/>");
    html.push_str("<line x1=\"154\" y1=\"26\" x2=\"154\" y2=\"41\"/>");
    html.push_str("<circle cx=\"83\" cy=\"43\" r=\"2.4\" fill=\"#fff\"/>");
    html.push_str("<circle cx=\"203\" cy=\"43\" r=\"2.4\" fill=\"#fff\"/>");
    html.push_str("<circle cx=\"83\" cy=\"180\" r=\"2.4\" fill=\"#fff\"/>");
    html.push_str("<circle cx=\"203\" cy=\"180\" r=\"2.4\" fill=\"#fff\"/>");
    html.push_str("<line x1=\"171\" y1=\"52\" x2=\"190\" y2=\"52\"/>");
    html.push_str("<line x1=\"203\" y1=\"69\" x2=\"203\" y2=\"102\"/>");
    html.push_str("<line x1=\"203\" y1=\"123\" x2=\"203\" y2=\"164\"/>");
    html.push_str("</g><g fill=\"#000\">");
    html.push_str("<polygon points=\"205,167 199,151 203,151 207,151\"/>");
    html.push_str("<polygon points=\"206,52 191,48 191,52 191,56\"/>");
    html.push_str("</g></svg>\n");
}

fn render_three_phase_circuit_diagram(html: &mut String, diagram: &DiagramOverlay) {
    html.push_str("      <svg class=\"pdf-diagram\" style=\"position:absolute;left:");
    push_pt(html, diagram.left);
    html.push_str(";top:");
    push_pt(html, diagram.top);
    html.push_str(";width:");
    push_pt(html, diagram.width);
    html.push_str(";height:");
    push_pt(html, diagram.height);
    html.push_str("\" viewBox=\"0 0 350 275\" aria-hidden=\"true\">");
    html.push_str("<rect x=\"0\" y=\"0\" width=\"350\" height=\"275\" fill=\"#fff\"/>");
    html.push_str("<g fill=\"none\" stroke=\"#555\" stroke-width=\"1.35\" stroke-linecap=\"square\" stroke-linejoin=\"miter\">");
    html.push_str("<rect x=\"6\" y=\"29\" width=\"74\" height=\"239\"/>");
    html.push_str("<rect x=\"266\" y=\"29\" width=\"75\" height=\"239\"/>");
    html.push_str("<polyline points=\"14,61 14,239 30,239\"/>");
    html.push_str("<line x1=\"30\" y1=\"239\" x2=\"266\" y2=\"239\"/>");
    for y in [61.0_f32, 123.0, 184.0] {
        html.push_str("<line x1=\"14\" y1=\"");
        push_number(html, y);
        html.push_str("\" x2=\"266\" y2=\"");
        push_number(html, y);
        html.push_str("\"/>");
        html.push_str("<circle cx=\"33\" cy=\"");
        push_number(html, y);
        html.push_str("\" r=\"8\"/>");
        html.push_str("<rect x=\"58\" y=\"");
        push_number(html, y - 3.0);
        html.push_str("\" width=\"18\" height=\"6\"/>");
        html.push_str("<circle cx=\"93\" cy=\"");
        push_number(html, y);
        html.push_str("\" r=\"2.4\" fill=\"#fff\"/>");
        html.push_str("<circle cx=\"250\" cy=\"");
        push_number(html, y);
        html.push_str("\" r=\"2.4\" fill=\"#fff\"/>");
        html.push_str("<rect x=\"158\" y=\"");
        push_number(html, y - 3.0);
        html.push_str("\" width=\"18\" height=\"6\"/>");
    }
    html.push_str("<rect x=\"149\" y=\"13\" width=\"43\" height=\"80\"/>");
    html.push_str("<rect x=\"161\" y=\"25\" width=\"24\" height=\"57\"/>");
    html.push_str("<circle cx=\"173\" cy=\"37\" r=\"14\"/>");
    html.push_str("<rect x=\"164\" y=\"72\" width=\"18\" height=\"7\"/>");
    html.push_str("<line x1=\"139\" y1=\"3\" x2=\"195\" y2=\"3\"/>");
    html.push_str("<line x1=\"139\" y1=\"264\" x2=\"195\" y2=\"264\"/>");
    for x in [139.0_f32, 195.0] {
        let mut y = 3.0_f32;
        while y < 264.0 {
            html.push_str("<line x1=\"");
            push_number(html, x);
            html.push_str("\" y1=\"");
            push_number(html, y);
            html.push_str("\" x2=\"");
            push_number(html, x);
            html.push_str("\" y2=\"");
            push_number(html, (y + 22.0).min(264.0));
            html.push_str("\"/>");
            y += 36.0;
        }
    }
    html.push_str("<line x1=\"208\" y1=\"61\" x2=\"230\" y2=\"61\"/>");
    html.push_str("<line x1=\"208\" y1=\"123\" x2=\"230\" y2=\"123\"/>");
    html.push_str("<line x1=\"208\" y1=\"184\" x2=\"230\" y2=\"184\"/>");
    html.push_str("<line x1=\"247\" y1=\"69\" x2=\"247\" y2=\"94\"/>");
    html.push_str("</g><g fill=\"#000\">");
    for y in [61.0_f32, 123.0, 184.0] {
        html.push_str("<polygon points=\"235,");
        push_number(html, y);
        html.push_str(" 218,");
        push_number(html, y - 5.0);
        html.push_str(" 218,");
        push_number(html, y + 5.0);
        html.push_str("\"/>");
    }
    html.push_str("<polygon points=\"247,111 242,94 247,94 252,94\"/>");
    html.push_str("</g></svg>\n");
}
pub(super) fn shape_intersects_diagram(
    shape: &RectShape,
    geometry: PageGeometry,
    diagrams: &[DiagramOverlay],
) -> bool {
    let left = (shape.x - geometry.min_x).max(0.0);
    let top = (geometry.height - shape.y - shape.height).max(0.0);
    rect_intersects_diagrams(
        diagrams,
        left,
        top,
        shape.width.max(0.0),
        shape.height.max(0.0),
    )
}
pub(super) fn path_intersects_diagram(
    path: &VisualPath,
    geometry: PageGeometry,
    diagrams: &[DiagramOverlay],
) -> bool {
    let Some((left, top, width, height)) = path_bounds(path, geometry) else {
        return false;
    };
    rect_intersects_diagrams(diagrams, left, top, width, height)
}

fn rect_intersects_diagrams(
    diagrams: &[DiagramOverlay],
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> bool {
    diagrams.iter().any(|diagram| {
        rects_intersect(
            left,
            top,
            width,
            height,
            diagram.left,
            diagram.top,
            diagram.width,
            diagram.height,
        )
    })
}

fn path_bounds(path: &VisualPath, geometry: PageGeometry) -> Option<(f32, f32, f32, f32)> {
    let mut points = path_points(&path.commands);
    let (first_x, first_y) = points.next()?;
    let (mut min_x, mut max_x) = (first_x, first_x);
    let (mut min_y, mut max_y) = (first_y, first_y);
    for (x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let left = (min_x - geometry.min_x).max(0.0);
    let top = (geometry.height - max_y).max(0.0);
    let width = (max_x - min_x).max(0.0);
    let height = (max_y - min_y).max(0.0);
    Some((left, top, width, height))
}

fn rects_intersect(
    left_a: f32,
    top_a: f32,
    width_a: f32,
    height_a: f32,
    left_b: f32,
    top_b: f32,
    width_b: f32,
    height_b: f32,
) -> bool {
    left_a < left_b + width_b
        && left_b < left_a + width_a
        && top_a < top_b + height_b
        && top_b < top_a + height_a
}
