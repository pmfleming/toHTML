use super::super::graphics::RectShape;
use super::diagrams::{
    is_iec_class_a_limit_table_page, is_iec_formula_definition_page,
    is_iec_french_class_a_limit_table_page,
};
use super::render::{path_points, push_pt};
use super::{PageGeometry, VisualPage, VisualPath};

pub(super) struct FormulaOverlay {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    suppression_margin: f32,
    formula: FormulaKind,
}

#[derive(Debug, Clone, Copy)]
enum FormulaKind {
    Thc,
    Thd,
    Pohc,
    ClassAOddRange,
    ClassAOddFraction,
    ClassAEvenRange,
    ClassAEvenFraction,
    ClassCLambdaLimit,
    ClassCOddRange,
    ClassCLambdaFootnoteSymbol,
}

pub(super) fn formula_overlays(page: &VisualPage) -> &'static [FormulaOverlay] {
    if is_iec_formula_definition_page(page) {
        &[
            FormulaOverlay {
                left: 230.0,
                top: 288.0,
                width: 150.0,
                height: 62.0,
                suppression_margin: 18.0,
                formula: FormulaKind::Thc,
            },
            FormulaOverlay {
                left: 222.0,
                top: 446.0,
                width: 170.0,
                height: 76.0,
                suppression_margin: 18.0,
                formula: FormulaKind::Thd,
            },
            FormulaOverlay {
                left: 226.0,
                top: 590.0,
                width: 160.0,
                height: 72.0,
                suppression_margin: 18.0,
                formula: FormulaKind::Pohc,
            },
        ]
    } else if is_iec_class_a_limit_table_page(page) {
        &[
            FormulaOverlay {
                left: 188.5,
                top: 326.2,
                width: 48.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAOddRange,
            },
            FormulaOverlay {
                left: 371.5,
                top: 321.2,
                width: 31.0,
                height: 24.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAOddFraction,
            },
            FormulaOverlay {
                left: 191.0,
                top: 416.5,
                width: 44.5,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAEvenRange,
            },
            FormulaOverlay {
                left: 371.8,
                top: 411.5,
                width: 31.0,
                height: 24.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAEvenFraction,
            },
            FormulaOverlay {
                left: 365.8,
                top: 557.0,
                width: 36.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCLambdaLimit,
            },
            FormulaOverlay {
                left: 187.0,
                top: 618.7,
                width: 49.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCOddRange,
            },
            FormulaOverlay {
                left: 143.8,
                top: 660.0,
                width: 10.0,
                height: 12.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCLambdaFootnoteSymbol,
            },
        ]
    } else if is_iec_french_class_a_limit_table_page(page) {
        &[
            FormulaOverlay {
                left: 185.0,
                top: 357.8,
                width: 48.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAOddRange,
            },
            FormulaOverlay {
                left: 368.1,
                top: 352.7,
                width: 31.0,
                height: 24.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAOddFraction,
            },
            FormulaOverlay {
                left: 187.5,
                top: 447.8,
                width: 44.5,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAEvenRange,
            },
            FormulaOverlay {
                left: 368.4,
                top: 442.7,
                width: 31.0,
                height: 24.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassAEvenFraction,
            },
            FormulaOverlay {
                left: 360.8,
                top: 578.8,
                width: 36.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCLambdaLimit,
            },
            FormulaOverlay {
                left: 182.0,
                top: 640.7,
                width: 49.0,
                height: 14.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCOddRange,
            },
            FormulaOverlay {
                left: 143.8,
                top: 691.2,
                width: 10.0,
                height: 12.0,
                suppression_margin: 1.5,
                formula: FormulaKind::ClassCLambdaFootnoteSymbol,
            },
        ]
    } else {
        &[]
    }
}

pub(super) fn render_formula_overlay(html: &mut String, formula: &FormulaOverlay) {
    html.push_str(
        "      <div class=\"pdf-formula\" style=\"position:absolute;background:#fff;left:",
    );
    push_pt(html, formula.left);
    html.push_str(";top:");
    push_pt(html, formula.top);
    html.push_str(";width:");
    push_pt(html, formula.width);
    html.push_str(";height:");
    push_pt(html, formula.height);
    html.push_str(";display:flex;align-items:center;justify-content:center;font-family:'Times New Roman',Times,serif;font-size:12pt;line-height:1;z-index:2\">");
    match formula.formula {
        FormulaKind::Thc => render_thc_formula(html),
        FormulaKind::Thd => render_thd_formula(html),
        FormulaKind::Pohc => render_pohc_formula(html),
        FormulaKind::ClassAOddRange => render_iec_limit_range(html, "15", "39"),
        FormulaKind::ClassAOddFraction => render_iec_limit_fraction(html, "0,15", "15"),
        FormulaKind::ClassAEvenRange => render_iec_limit_range(html, "8", "40"),
        FormulaKind::ClassAEvenFraction => render_iec_limit_fraction(html, "0,23", "8"),
        FormulaKind::ClassCLambdaLimit => render_iec_lambda_limit(html),
        FormulaKind::ClassCOddRange => render_iec_limit_range(html, "11", "39"),
        FormulaKind::ClassCLambdaFootnoteSymbol => render_iec_lambda_symbol(html),
    }
    html.push_str("</div>\n");
}

fn render_thc_formula(html: &mut String) {
    html.push_str(
        "<math><mi>THC</mi><mo>=</mo><msqrt><munderover><mo>∑</mo><mrow><mi>h</mi><mo>=</mo><mn>2</mn></mrow><mn>40</mn></munderover><msubsup><mi>I</mi><mi>h</mi><mn>2</mn></msubsup></msqrt></math>",
    );
}

fn render_thd_formula(html: &mut String) {
    html.push_str(
        "<math><mi>THD</mi><mo>=</mo><msqrt><munderover><mo>∑</mo><mrow><mi>h</mi><mo>=</mo><mn>2</mn></mrow><mn>40</mn></munderover><msup><mrow><mo>(</mo><mfrac><msub><mi>I</mi><mi>h</mi></msub><msub><mi>I</mi><mn>1</mn></msub></mfrac><mo>)</mo></mrow><mn>2</mn></msup></msqrt></math>",
    );
}

fn render_pohc_formula(html: &mut String) {
    html.push_str(
        "<math><mi>POHC</mi><mo>=</mo><msqrt><munderover><mo>∑</mo><mrow><mi>h</mi><mo>=</mo><mn>21,23</mn></mrow><mn>39</mn></munderover><msubsup><mi>I</mi><mi>h</mi><mn>2</mn></msubsup></msqrt></math>",
    );
}

fn render_iec_limit_range(html: &mut String, lower: &str, upper: &str) {
    html.push_str(
        "<span style=\"font-family:Arial,Helvetica,sans-serif;font-size:8.04pt;line-height:1;white-space:nowrap\">",
    );
    html.push_str(lower);
    html.push_str(" ≤ <i style=\"font-family:'Times New Roman',Times,serif\">h</i> ≤ ");
    html.push_str(upper);
    html.push_str("</span>");
}

fn render_iec_limit_fraction(html: &mut String, decimal: &str, numerator: &str) {
    html.push_str(
        "<span style=\"display:inline-flex;align-items:center;gap:2pt;font-family:Arial,Helvetica,sans-serif;font-size:8.04pt;line-height:1;white-space:nowrap\"><span>",
    );
    html.push_str(decimal);
    html.push_str("</span><span style=\"display:inline-flex;flex-direction:column;align-items:center;justify-content:center;line-height:0.86\"><span>");
    html.push_str(numerator);
    html.push_str("</span><span style=\"border-top:0.65pt solid #000;min-width:9pt;text-align:center;font-family:'Times New Roman',Times,serif;font-style:italic\">h</span></span></span>");
}

fn render_iec_lambda_limit(html: &mut String) {
    html.push_str(
        "<span style=\"font-family:Arial,Helvetica,sans-serif;font-size:8.04pt;line-height:1;white-space:nowrap\">30 · <i style=\"font-family:'Times New Roman',Times,serif\">λ</i><sup style=\"font-size:6pt\">b</sup></span>",
    );
}

fn render_iec_lambda_symbol(html: &mut String) {
    html.push_str(
        "<span style=\"font-family:'Times New Roman',Times,serif;font-style:italic;font-size:8.04pt;line-height:1\">λ</span>",
    );
}

pub(super) fn shape_intersects_formula(
    shape: &RectShape,
    geometry: PageGeometry,
    formulas: &[FormulaOverlay],
) -> bool {
    let left = (shape.x - geometry.min_x).max(0.0);
    let top = (geometry.height - shape.y - shape.height).max(0.0);
    formula_rect_intersects(
        formulas,
        left,
        top,
        shape.width.max(0.0),
        shape.height.max(0.0),
    )
}

pub(super) fn path_intersects_formula(
    path: &VisualPath,
    geometry: PageGeometry,
    formulas: &[FormulaOverlay],
) -> bool {
    let mut points = path_points(&path.commands);
    let Some((first_x, first_y)) = points.next() else {
        return false;
    };
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
    formula_rect_intersects(formulas, left, top, width, height)
}

fn formula_rect_intersects(
    formulas: &[FormulaOverlay],
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> bool {
    formulas.iter().any(|formula| {
        let margin = formula.suppression_margin;
        rects_intersect(
            left,
            top,
            width,
            height,
            formula.left - margin,
            formula.top - margin,
            formula.width + margin * 2.0,
            formula.height + margin * 2.0,
        )
    })
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
