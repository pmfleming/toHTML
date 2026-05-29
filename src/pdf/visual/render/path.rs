use super::{push_number, push_pt};
use crate::html::escape::push_attr_escaped;
use crate::pdf::graphics::PathCommand;
use crate::pdf::visual::{PageGeometry, VisualPath};

pub(in crate::pdf::visual) fn render_path(
    html: &mut String,
    path: &VisualPath,
    geometry: PageGeometry,
    outline_filled_boxes: bool,
) {
    if path.commands.len() < 2 {
        return;
    }
    let outline_filled_path = outline_filled_boxes && is_filled_outline_path(path);

    html.push_str("      <svg class=\"pdf-ink\" style=\"left:0;top:0;width:");
    push_pt(html, geometry.width);
    html.push_str(";height:");
    push_pt(html, geometry.height);
    html.push_str("\" viewBox=\"0 0 ");
    push_number(html, geometry.width);
    html.push(' ');
    push_number(html, geometry.height);
    html.push_str("\" aria-hidden=\"true\"><path d=\"");
    render_path_data(html, path, geometry);
    html.push_str("\" fill=\"");
    if outline_filled_path {
        html.push_str("none");
    } else if let Some(fill) = &path.fill {
        push_attr_escaped(html, fill);
    } else {
        html.push_str("none");
    }
    html.push('"');
    let stroke = if outline_filled_path {
        path.fill.as_ref()
    } else {
        path.stroke.as_ref()
    };
    if let Some(stroke) = stroke {
        html.push_str(" stroke=\"");
        push_attr_escaped(html, stroke);
        html.push_str("\" stroke-width=\"");
        push_number(html, path.stroke_width.max(0.75));
        html.push_str("\" stroke-linecap=\"round\" stroke-linejoin=\"round\"");
        if !outline_filled_path {
            push_dasharray(html, path.stroke_dasharray.as_deref());
        }
    }
    html.push_str("/></svg>\n");
}

fn push_dasharray(html: &mut String, dasharray: Option<&[f32]>) {
    let Some(dasharray) = dasharray.filter(|values| !values.is_empty()) else {
        return;
    };
    html.push_str(" stroke-dasharray=\"");
    for (index, value) in dasharray.iter().enumerate() {
        if index > 0 {
            html.push(' ');
        }
        push_number(html, *value);
    }
    html.push('"');
}

fn is_filled_outline_path(path: &VisualPath) -> bool {
    if !matches!(path.fill.as_deref(), Some("#000000" | "#000")) || path.stroke.is_some() {
        return false;
    }
    if path
        .commands
        .iter()
        .any(|command| matches!(command, PathCommand::CubicTo(..)))
    {
        return false;
    }
    let points = path_points(&path.commands).collect::<Vec<_>>();
    if points.len() < 4 || points.len() > 8 {
        return false;
    }
    let Some((width, height)) = path_bounds(&points) else {
        return false;
    };
    if width < 6.0 || height < 8.0 {
        return false;
    }
    let area = polygon_area(&points);
    let bounds_area = width * height;
    bounds_area > 0.0 && area / bounds_area >= 0.75
}

fn render_path_data(html: &mut String, path: &VisualPath, geometry: PageGeometry) {
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(x, y) => {
                html.push('M');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::LineTo(x, y) => {
                html.push('L');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
                html.push('C');
                push_path_point(html, x1, y1, geometry);
                html.push(' ');
                push_path_point(html, x2, y2, geometry);
                html.push(' ');
                push_path_point(html, x, y, geometry);
            }
            PathCommand::Close => html.push('Z'),
        }
    }
}

fn push_path_point(html: &mut String, x: f32, y: f32, geometry: PageGeometry) {
    push_number(html, (x - geometry.min_x).max(0.0));
    html.push(' ');
    push_number(html, (geometry.height - y).max(0.0));
}

pub(in crate::pdf::visual) fn path_points(
    commands: &[PathCommand],
) -> impl Iterator<Item = (f32, f32)> + '_ {
    commands.iter().flat_map(|command| match *command {
        PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => vec![(x, y)],
        PathCommand::CubicTo(x1, y1, x2, y2, x, y) => vec![(x1, y1), (x2, y2), (x, y)],
        PathCommand::Close => Vec::new(),
    })
}

fn path_bounds(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    let min_x = points.iter().map(|point| point.0).reduce(f32::min)?;
    let max_x = points.iter().map(|point| point.0).reduce(f32::max)?;
    let min_y = points.iter().map(|point| point.1).reduce(f32::min)?;
    let max_y = points.iter().map(|point| point.1).reduce(f32::max)?;
    Some((max_x - min_x, max_y - min_y))
}

fn polygon_area(points: &[(f32, f32)]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for index in 0..points.len() {
        let (x1, y1) = points[index];
        let (x2, y2) = points[(index + 1) % points.len()];
        area += x1 * y2 - x2 * y1;
    }
    area.abs() / 2.0
}
