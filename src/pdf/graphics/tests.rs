use super::*;
use std::collections::HashMap;

#[test]
fn extracts_filled_and_stroked_rectangles() {
    let shapes = extract_rectangles(b"0.9 g 10 20 100 30 re f 0 G 10 20 100 30 re S");

    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].fill.as_deref(), Some("#e6e6e6"));
    assert_eq!(shapes[1].stroke.as_deref(), Some("#000000"));
    assert_eq!(shapes[0].x, 10.0);
    assert_eq!(shapes[0].height, 30.0);
}

#[test]
fn extracts_generic_color_space_fill_components() {
    let shapes = extract_rectangles(b"/CS0 cs 0.024 0.451 0.647 scn 10 20 100 30 re f");

    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].fill.as_deref(), Some("#0673a5"));
}

#[test]
fn applies_matrix_to_rectangles() {
    let shapes = extract_rectangles(b"2 0 0 2 5 7 cm 10 20 100 30 re S");

    assert_eq!(shapes[0].x, 25.0);
    assert_eq!(shapes[0].y, 47.0);
    assert_eq!(shapes[0].width, 200.0);
    assert_eq!(shapes[0].height, 60.0);
}

#[test]
fn extracts_axis_aligned_stroked_path_lines() {
    let shapes = extract_rectangles(b"0 G 2 w 10 20 m 110 20 l 110 50 l 10 50 l h S");

    assert_eq!(shapes.len(), 4);
    assert_eq!(shapes[0].fill.as_deref(), Some("#000000"));
    assert_eq!(shapes[0].x, 10.0);
    assert_eq!(shapes[0].y, 19.0);
    assert_eq!(shapes[0].width, 100.0);
    assert_eq!(shapes[0].height, 2.0);
}

#[test]
fn extracts_near_horizontal_hairline_as_shape() {
    let shapes = extract_rectangles(b"0.612 G 0.25 w 109.7 739.89 m 537.75 739.84 l S");

    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].fill.as_deref(), Some("#9c9c9c"));
    assert!((shapes[0].x - 109.7).abs() < 0.01);
    assert!((shapes[0].y - 739.765).abs() < 0.01);
    assert!((shapes[0].width - 428.05).abs() < 0.01);
    assert_eq!(shapes[0].height, 0.25);
}

#[test]
fn extracts_filled_path_rectangle() {
    let shapes = extract_rectangles(b"1 1 0 rg 10 20 m 110 20 l 110 50 l 10 50 l h f");

    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes[0].fill.as_deref(), Some("#ffff00"));
    assert_eq!(shapes[0].x, 10.0);
    assert_eq!(shapes[0].y, 20.0);
    assert_eq!(shapes[0].width, 100.0);
    assert_eq!(shapes[0].height, 30.0);
}

#[test]
fn extracts_transformed_even_odd_filled_path_rectangles() {
    let shapes = extract_rectangles(
        b"0.75 0 0 -0.75 0 595.32 cm
q
1 1 0 rg
700 236.64 m
831.36 236.64 l
831.36 256.16 l
700 256.16 l
h
397.92 295.36 m
672.16 295.36 l
672.16 314.88 l
397.92 314.88 l
h
f*
Q",
    );

    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].fill.as_deref(), Some("#ffff00"));
    assert_eq!(shapes[0].x, 525.0);
    assert!((shapes[0].y - 403.2).abs() < 0.01);
    assert!((shapes[0].width - 98.52).abs() < 0.01);
    assert!((shapes[0].height - 14.64).abs() < 0.01);
}

#[test]
fn extracts_non_axis_aligned_stroked_paths() {
    let paths = extract_paths(b"1 0 0 RG 3 w 10 20 m 30 40 l 50 20 l S");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].fill, None);
    assert_eq!(paths[0].stroke.as_deref(), Some("#ff0000"));
    assert_eq!(paths[0].stroke_width, 3.0);
    assert!(paths[0].stroke_dasharray.is_none());
    assert_eq!(
        paths[0].commands,
        vec![
            PathCommand::MoveTo(10.0, 20.0),
            PathCommand::LineTo(30.0, 40.0),
            PathCommand::LineTo(50.0, 20.0),
        ]
    );
}

#[test]
fn scales_stroked_path_width_by_current_transform() {
    let paths = extract_paths(b"0.1 0 0 0.1 0 0 cm 0 G 16 w 10 20 m 30 40 l S");

    assert_eq!(paths.len(), 1);
    assert!((paths[0].stroke_width - 1.6).abs() < 0.01);
}

#[test]
fn scales_axis_aligned_line_shapes_by_current_transform() {
    let shapes = extract_rectangles(b"0.1 0 0 0.1 0 0 cm 0 G 16 w 10 20 m 110 20 l S");

    assert_eq!(shapes.len(), 1);
    assert!((shapes[0].height - 1.6).abs() < 0.01);
}

#[test]
fn extracts_dashed_axis_aligned_paths() {
    let stream = b"[3 2] 0 d 0 G 1 w 10 20 m 110 20 l S";
    let paths = extract_paths(stream);
    let shapes = extract_rectangles(stream);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].stroke.as_deref(), Some("#000000"));
    assert_eq!(paths[0].stroke_dasharray.as_deref(), Some(&[3.0, 2.0][..]));
    assert!(shapes.is_empty());
}

#[test]
fn extracts_stroked_bezier_paths() {
    let paths = extract_paths(b"1 0 0 RG 2 w 10 20 m 15 35 25 35 30 20 c S");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].stroke.as_deref(), Some("#ff0000"));
    assert!(matches!(paths[0].commands[1], PathCommand::CubicTo(..)));
}

#[test]
fn extracts_filled_bezier_paths() {
    let paths = extract_paths(b"0.9 0.07 0.14 rg 10 20 m 15 35 25 35 30 20 c h f");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].fill.as_deref(), Some("#e61224"));
    assert_eq!(paths[0].stroke, None);
    assert!(matches!(paths[0].commands[1], PathCommand::CubicTo(..)));
}

#[test]
fn extracts_shading_paint_from_active_clip_path() {
    let paths = extract_paths(
        b"1 0.4 0 rg
10 20 m
10 16 14 12 18 12 c
120 12 l
124 12 128 16 128 20 c
128 42 l
10 42 l
h
W n
0 g
/Sh0 sh",
    );

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].fill.as_deref(), Some("#ff6600"));
    assert_eq!(paths[0].stroke, None);
    assert!(matches!(paths[0].commands[1], PathCommand::CubicTo(..)));
}

#[test]
fn reuses_first_wide_shading_fill_when_current_fill_is_text_color() {
    let paths = extract_paths(
        b"1 0.4 0 rg
10 20 m 10 16 14 12 18 12 c 180 12 l 184 12 188 16 188 20 c 188 42 l 10 42 l h
W n /Sh0 sh
0 0.62 0.725 rg
210 20 m 210 16 214 12 218 12 c 380 12 l 384 12 388 16 388 20 c 388 42 l 210 42 l h
W n /Sh1 sh",
    );

    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].fill.as_deref(), Some("#ff6600"));
    assert_eq!(paths[1].fill.as_deref(), Some("#ff6600"));
}

#[test]
fn named_shading_resource_overrides_stale_fill_color() {
    let shading_fills = HashMap::from([("Sh1".to_string(), "#ff6600".to_string())]);
    let paths = extract_paths_with_shading_fills(
        b"0 0.62 0.588 rg
10 20 m 10 16 14 12 18 12 c 180 12 l 184 12 188 16 188 20 c 188 42 l 10 42 l h
W n q 0 g /Sh1 sh Q",
        &shading_fills,
    );

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].fill.as_deref(), Some("#ff6600"));
}
