use super::{GraphicsState, PathCommand, RectShape, VectorPath};

#[derive(Debug, Default, Clone)]
pub(super) struct Path {
    current: Option<(f32, f32)>,
    start: Option<(f32, f32)>,
    current_contour: Vec<LineSegment>,
    contours: Vec<Vec<LineSegment>>,
    all_lines: Vec<LineSegment>,
    commands: Vec<PathCommand>,
}

impl Path {
    pub(super) fn move_to(&mut self, point: (f32, f32)) {
        self.finish_open_contour();
        self.current = Some(point);
        self.start = Some(point);
        self.commands.push(PathCommand::MoveTo(point.0, point.1));
    }

    pub(super) fn line_to(&mut self, point: (f32, f32)) {
        if let Some(current) = self.current {
            let line = LineSegment {
                start: current,
                end: point,
            };
            self.current_contour.push(line);
            self.all_lines.push(line);
        }
        self.current = Some(point);
        self.commands.push(PathCommand::LineTo(point.0, point.1));
    }

    pub(super) fn curve_to(
        &mut self,
        control_1: (f32, f32),
        control_2: (f32, f32),
        end: (f32, f32),
    ) {
        self.finish_open_contour();
        self.current = Some(end);
        self.commands.push(PathCommand::CubicTo(
            control_1.0,
            control_1.1,
            control_2.0,
            control_2.1,
            end.0,
            end.1,
        ));
    }

    pub(super) fn close(&mut self) {
        if let (Some(current), Some(start)) = (self.current, self.start) {
            if !same_point(current, start) {
                let line = LineSegment {
                    start: current,
                    end: start,
                };
                self.current_contour.push(line);
                self.all_lines.push(line);
            }
            self.finish_open_contour();
            self.current = Some(start);
            self.start = None;
            self.commands.push(PathCommand::Close);
        }
    }

    pub(super) fn current_point(&self) -> Option<(f32, f32)> {
        self.current
    }

    pub(super) fn clear(&mut self) {
        self.current = None;
        self.start = None;
        self.current_contour.clear();
        self.contours.clear();
        self.all_lines.clear();
        self.commands.clear();
    }

    pub(super) fn finish_open_contour(&mut self) {
        if !self.current_contour.is_empty() {
            self.contours
                .push(std::mem::take(&mut self.current_contour));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LineSegment {
    start: (f32, f32),
    end: (f32, f32),
}
pub(super) fn push_rectangles(
    shapes: &mut Vec<RectShape>,
    rectangles: &[RectShape],
    fill: Option<String>,
    stroke: Option<String>,
) {
    for rectangle in rectangles {
        if rectangle.width.abs() < 0.25 || rectangle.height.abs() < 0.25 {
            continue;
        }
        let mut shape = rectangle.clone();
        shape.fill = fill.clone();
        shape.stroke = stroke.clone();
        shapes.push(shape);
    }
}

pub(super) fn push_filled_path_rectangles(
    shapes: &mut Vec<RectShape>,
    path: &Path,
    fill: Option<String>,
) {
    for mut rectangle in path_rectangles(path) {
        if rectangle.width.abs() < 0.25 || rectangle.height.abs() < 0.25 {
            continue;
        }
        rectangle.fill = fill.clone();
        shapes.push(rectangle);
    }
}

pub(super) fn push_stroked_path_lines(
    shapes: &mut Vec<RectShape>,
    path: &Path,
    stroke: Option<String>,
    line_width: f32,
) {
    for line in &path.all_lines {
        let Some(mut shape) = line_shape(*line, line_width) else {
            continue;
        };
        shape.fill = stroke.clone();
        shapes.push(shape);
    }
}

pub(super) fn push_vector_path(
    paths: &mut Vec<VectorPath>,
    path: &Path,
    fill: Option<String>,
    stroke: Option<String>,
    stroke_width: f32,
    stroke_dasharray: Option<Vec<f32>>,
) {
    if fill.is_none() && stroke.is_none() {
        return;
    }
    let is_dashed_stroke = stroke.is_some()
        && stroke_dasharray
            .as_ref()
            .is_some_and(|dasharray| !dasharray.is_empty());
    if path.commands.len() < 2 || (path_is_axis_aligned_only(path) && !is_dashed_stroke) {
        return;
    }
    paths.push(VectorPath {
        commands: path.commands.clone(),
        fill,
        stroke,
        stroke_width,
        stroke_dasharray,
    });
}

pub(super) fn dash_array(state: &GraphicsState) -> Option<Vec<f32>> {
    (!state.dash_array.is_empty()).then(|| state.dash_array.clone())
}

fn path_is_axis_aligned_only(path: &Path) -> bool {
    !path
        .commands
        .iter()
        .any(|command| matches!(command, PathCommand::CubicTo(..)))
        && path.all_lines.iter().all(axis_aligned)
}

fn path_rectangles(path: &Path) -> Vec<RectShape> {
    path.contours
        .iter()
        .filter_map(|contour| contour_rectangle(contour))
        .collect()
}

fn contour_rectangle(lines: &[LineSegment]) -> Option<RectShape> {
    if lines.len() < 4 || !lines.iter().all(axis_aligned) {
        return None;
    }
    let first = lines.first()?.start;
    let last = lines.last()?.end;
    if !same_point(first, last) {
        return None;
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for line in lines {
        for (x, y) in [line.start, line.end] {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    Some(RectShape {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
        fill: None,
        stroke: None,
    })
}

fn line_shape(line: LineSegment, line_width: f32) -> Option<RectShape> {
    let thickness = line_width.max(0.25);
    if horizontal(line) {
        let x = line.start.0.min(line.end.0);
        let width = (line.start.0 - line.end.0).abs();
        if width < 0.25 {
            return None;
        }
        return Some(RectShape {
            x,
            y: line.start.1 - thickness / 2.0,
            width,
            height: thickness,
            fill: None,
            stroke: None,
        });
    }
    if vertical(line) {
        let y = line.start.1.min(line.end.1);
        let height = (line.start.1 - line.end.1).abs();
        if height < 0.25 {
            return None;
        }
        return Some(RectShape {
            x: line.start.0 - thickness / 2.0,
            y,
            width: thickness,
            height,
            fill: None,
            stroke: None,
        });
    }
    None
}

fn axis_aligned(line: &LineSegment) -> bool {
    horizontal(*line) || vertical(*line)
}

fn horizontal(line: LineSegment) -> bool {
    (line.start.1 - line.end.1).abs() < 0.1
}

fn vertical(line: LineSegment) -> bool {
    (line.start.0 - line.end.0).abs() < 0.1
}

fn same_point(left: (f32, f32), right: (f32, f32)) -> bool {
    (left.0 - right.0).abs() < 0.01 && (left.1 - right.1).abs() < 0.01
}
