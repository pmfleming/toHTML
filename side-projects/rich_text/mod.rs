//! A tiny egui-native rich text editor surface.
//!
//! This lives outside `html-editor` so the custom widget can evolve separately
//! from the application shell and command palette.

use crate::doc::{
    chars_to_string, Block, Caret, PdfBox, PdfElement, PdfPage, PdfTextFragment, StyledChar,
};
use crate::{palette, App};

mod render;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, Context, Event, FontFamily, FontId, Key, Modifiers, Rect,
    RichText, Sense, Stroke, StrokeKind, Ui,
};
use render::{render_inline_tagged_runs, render_runs_with_selection, TaggedRuns};

pub fn handle_input(app: &mut App, ctx: &Context) {
    if !app.editor_has_focus {
        return;
    }

    let mut inserted = String::new();
    for event in ctx.input(|i| i.events.clone()) {
        match event {
            Event::Paste(text) => push_paste_text(&mut inserted, &text),
            Event::Text(text) => {
                if text == "<" {
                    palette::open_tags(app);
                    return;
                }
                if !text.chars().all(|c| c.is_control()) {
                    inserted.push_str(&text);
                }
            }
            _ => {}
        }
    }

    if !inserted.is_empty() {
        app.commit_edit(|app| {
            let deleted = app.delete_selection();
            let style = app.current_style.clone();
            let inserted = app.doc.insert_text(&mut app.caret, &inserted, &style);
            deleted || inserted
        });
        return;
    }

    let backspace = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Backspace));
    let delete = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete));
    let enter = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    let shift_tab = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Tab));
    let tab = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab));
    let shift_left = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::ArrowLeft));
    let shift_right = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::ArrowRight));
    let left = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft));
    let right = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight));
    let up = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
    let down = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));

    if backspace {
        app.commit_edit(|app| {
            if app.delete_selection() {
                true
            } else {
                let changed = app.doc.backspace(&mut app.caret);
                app.clear_selection();
                changed
            }
        });
    } else if delete {
        app.commit_edit(|app| {
            if app.delete_selection() {
                true
            } else {
                let changed = app.doc.delete_forward(&mut app.caret);
                app.clear_selection();
                changed
            }
        });
    } else if enter {
        app.commit_edit(|app| {
            let deleted = app.delete_selection();
            let split = app.doc.split_block(&mut app.caret);
            app.clear_selection();
            deleted || split
        });
    } else if shift_tab {
        app.clear_selection();
        app.doc.move_table_cell(&mut app.caret, -1);
    } else if tab {
        app.clear_selection();
        app.doc.move_table_cell(&mut app.caret, 1);
    } else if shift_left {
        extend_selection(app, MoveDirection::Left);
    } else if shift_right {
        extend_selection(app, MoveDirection::Right);
    } else if left {
        collapse_or_move(app, MoveDirection::Left);
    } else if right {
        collapse_or_move(app, MoveDirection::Right);
    } else if up {
        app.clear_selection();
        if !app.doc.move_table_row(&mut app.caret, -1) {
            move_block(app, -1);
        }
    } else if down {
        app.clear_selection();
        if !app.doc.move_table_row(&mut app.caret, 1) {
            move_block(app, 1);
        }
    }
}

fn push_paste_text(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '\r' => {}
            '\n' | '\t' => out.push(ch),
            c if !c.is_control() => out.push(c),
            _ => {}
        }
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let font_size = app.settings.font_size;
    let line_height = app.settings.line_height;
    let mut clicked: Option<ClickTarget> = None;

    for block_idx in 0..app.doc.blocks.len() {
        let response = match &app.doc.blocks[block_idx] {
            Block::Heading(level, runs) => show_runs(
                app,
                ui,
                block_idx,
                None,
                runs,
                font_size + (7.0 - *level as f32).max(1.0) * 2.2,
                "",
            ),
            Block::Paragraph(runs) => show_runs(app, ui, block_idx, None, runs, font_size, ""),
            Block::Blockquote(runs) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("|").color(ui.visuals().weak_text_color()));
                    show_runs(app, ui, block_idx, None, runs, font_size, "")
                })
                .inner
            }
            Block::Bullet(runs) => show_runs(app, ui, block_idx, None, runs, font_size, "* "),
            Block::Numbered(runs) => show_runs(app, ui, block_idx, None, runs, font_size, "1. "),
            Block::Pre(runs) => show_runs(app, ui, block_idx, None, runs, font_size, "    "),
            Block::Table(table) => {
                let mut table_clicked = None;
                egui::Grid::new(format!("table-{block_idx}"))
                    .striped(true)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        for (row_idx, row) in table.rows.iter().enumerate() {
                            for (col_idx, cell) in row.cells.iter().enumerate() {
                                let target = Some((row_idx, col_idx));
                                let resp = show_runs(
                                    app,
                                    ui,
                                    block_idx,
                                    target,
                                    &cell.content,
                                    font_size,
                                    "",
                                );
                                if resp.clicked() {
                                    table_clicked =
                                        Some(ClickTarget::TableCell(block_idx, row_idx, col_idx));
                                }
                            }
                            ui.end_row();
                        }
                    });
                if let Some(target) = table_clicked {
                    clicked = Some(target);
                }
                ui.allocate_response(egui::vec2(ui.available_width(), 2.0), Sense::click())
            }
            Block::Image(image) => {
                ui.label(RichText::new(format!("[image: {}]", image.alt)).italics())
            }
            Block::PdfPage(page) => show_pdf_page(ui, page),
            Block::PageBreak(page) => {
                let label = page
                    .map(|p| format!("--- page {p} ---"))
                    .unwrap_or_else(|| "--- page break ---".into());
                ui.label(
                    RichText::new(label)
                        .small()
                        .color(ui.visuals().weak_text_color()),
                )
            }
            Block::PagePlaceholder { page, reason } => {
                let page = page.map(|p| p.to_string()).unwrap_or_else(|| "?".into());
                ui.label(
                    RichText::new(format!("[page {page}: {reason}]"))
                        .small()
                        .color(ui.visuals().weak_text_color()),
                )
            }
            Block::RawHtml(html) => ui.label(RichText::new(html).monospace().small()),
            Block::Hr => {
                ui.separator();
                ui.allocate_response(egui::vec2(ui.available_width(), 4.0), Sense::click())
            }
        };

        if response.clicked() {
            clicked = Some(ClickTarget::Block(block_idx));
        }
        ui.add_space((font_size * (line_height - 1.0)).max(2.0));
    }

    if let Some(target) = clicked {
        match target {
            ClickTarget::Block(block) => app.doc.move_to_block_end(&mut app.caret, block),
            ClickTarget::TableCell(block, row, col) => {
                app.doc.move_to_table_cell(&mut app.caret, block, row, col)
            }
        }
        app.clear_selection();
        app.editor_has_focus = true;
    }
}

fn show_pdf_page(ui: &mut Ui, page: &PdfPage) -> egui::Response {
    let page_width = page.width_pt.unwrap_or_else(|| inferred_page_width(page));
    let page_height = page.height_pt.unwrap_or_else(|| inferred_page_height(page));
    let available = ui.available_width().max(320.0);
    let scale = (available / page_width.max(1.0)).min(1.0);
    let size = vec2(page_width * scale, page_height * scale);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, Color32::WHITE);
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, Color32::from_gray(180)),
        StrokeKind::Inside,
    );

    for element in &page.elements {
        match element {
            PdfElement::Shape(shape) => {
                let element_rect = scaled_rect(rect, scale, &shape.bounds);
                if let Some(background) = parse_hex_color(shape.background.as_deref()) {
                    painter.rect_filled(element_rect, 0.0, background);
                } else if shape.background.is_none() && shape.border.is_none() {
                    painter.rect_filled(element_rect, 0.0, Color32::LIGHT_GRAY);
                }
                if shape.border.is_some() {
                    let width = shape.border_width_pt.unwrap_or(1.0).max(0.5) * scale.max(0.5);
                    let color = parse_hex_color(shape.border_color.as_deref())
                        .unwrap_or(Color32::from_gray(95));
                    painter.rect_stroke(
                        element_rect,
                        0.0,
                        Stroke::new(width, color),
                        StrokeKind::Inside,
                    );
                }
            }
            PdfElement::Image(image) => {
                let element_rect = scaled_rect(rect, scale, &image.bounds);
                painter.rect_filled(element_rect, 0.0, Color32::from_gray(235));
                painter.rect_stroke(
                    element_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_gray(165)),
                    StrokeKind::Inside,
                );
                let label = if image.alt.is_empty() {
                    "image"
                } else {
                    image.alt.as_str()
                };
                painter.text(
                    element_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::monospace((10.0 * scale).clamp(7.0, 12.0)),
                    Color32::from_gray(90),
                );
            }
            PdfElement::Text(text) => {
                let pos = pos2(
                    rect.left() + text.bounds.left_pt.unwrap_or(0.0) * scale,
                    rect.top() + text.bounds.top_pt.unwrap_or(0.0) * scale,
                );
                if text.rotation_deg.is_some() {
                    painter.rect_stroke(
                        scaled_rect(rect, scale, &text.bounds),
                        0.0,
                        Stroke::new(0.75, Color32::from_rgb(70, 115, 180)),
                        StrokeKind::Inside,
                    );
                }
                painter.text(
                    pos,
                    Align2::LEFT_TOP,
                    &text.text,
                    pdf_text_font_id(text, scale),
                    parse_hex_color(text.color.as_deref()).unwrap_or(Color32::BLACK),
                );
            }
            PdfElement::Ink(ink) => {
                let element_rect = scaled_rect(rect, scale, &ink.bounds);
                painter.rect_stroke(
                    element_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_gray(120)),
                    StrokeKind::Inside,
                );
            }
            PdfElement::Link(link) => {
                let element_rect = scaled_rect(rect, scale, &link.bounds);
                painter.rect_stroke(
                    element_rect,
                    0.0,
                    Stroke::new(1.0, ui.visuals().hyperlink_color),
                    StrokeKind::Inside,
                );
            }
        }
    }

    response
}

fn pdf_text_font_id(text: &PdfTextFragment, scale: f32) -> FontId {
    let family = text
        .font_family
        .as_deref()
        .map(pdf_font_family)
        .unwrap_or(FontFamily::Proportional);
    FontId::new(text.font_size_pt.unwrap_or(10.0) * scale, family)
}

fn pdf_font_family(font_family: &str) -> FontFamily {
    let font_family = font_family.to_ascii_lowercase();
    if font_family.contains("courier") || font_family.contains("monospace") {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    }
}

fn scaled_rect(page_rect: Rect, scale: f32, bounds: &PdfBox) -> Rect {
    Rect::from_min_size(
        pos2(
            page_rect.left() + bounds.left_pt.unwrap_or(0.0) * scale,
            page_rect.top() + bounds.top_pt.unwrap_or(0.0) * scale,
        ),
        vec2(
            bounds.width_pt.unwrap_or(1.0).max(1.0) * scale,
            bounds.height_pt.unwrap_or(1.0).max(1.0) * scale,
        ),
    )
}

fn inferred_page_width(page: &PdfPage) -> f32 {
    page.elements
        .iter()
        .filter_map(element_bounds)
        .filter_map(|bounds| Some(bounds.left_pt? + bounds.width_pt?))
        .fold(612.0, f32::max)
}

fn inferred_page_height(page: &PdfPage) -> f32 {
    page.elements
        .iter()
        .filter_map(element_bounds)
        .filter_map(|bounds| Some(bounds.top_pt? + bounds.height_pt?))
        .fold(792.0, f32::max)
}

fn element_bounds(element: &PdfElement) -> Option<&PdfBox> {
    match element {
        PdfElement::Text(text) => Some(&text.bounds),
        PdfElement::Image(image) => Some(&image.bounds),
        PdfElement::Shape(shape) => Some(&shape.bounds),
        PdfElement::Ink(ink) => Some(&ink.bounds),
        PdfElement::Link(link) => Some(&link.bounds),
    }
}

fn parse_hex_color(value: Option<&str>) -> Option<Color32> {
    let value = value?.trim().trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }
    Some(Color32::from_rgb(
        u8::from_str_radix(&value[0..2], 16).ok()?,
        u8::from_str_radix(&value[2..4], 16).ok()?,
        u8::from_str_radix(&value[4..6], 16).ok()?,
    ))
}

fn show_runs(
    app: &App,
    ui: &mut Ui,
    block_idx: usize,
    table_cell: Option<(usize, usize)>,
    runs: &[StyledChar],
    font_size: f32,
    prefix: &str,
) -> egui::Response {
    let caret_pos = caret_position_for(app, block_idx, table_cell, runs.len());
    let selection = selection_range_for(app, block_idx, table_cell, runs.len());
    let response = ui
        .horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            if !prefix.is_empty() {
                ui.label(RichText::new(prefix).size(font_size));
            }

            if let Some(pos) = caret_pos {
                if app.settings.show_tags {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    render_inline_tagged_runs(TaggedRuns {
                        app,
                        ui,
                        block: &app.doc.blocks[block_idx],
                        runs,
                        font_size,
                        selection,
                        caret_pos: pos,
                        table_cell,
                    });
                } else {
                    render_runs_with_selection(app, ui, runs, font_size, selection, Some(pos));
                }
            } else if runs.is_empty() {
                ui.label(RichText::new(" ").size(font_size));
            } else {
                render_runs_with_selection(app, ui, runs, font_size, selection, None);
            }
        })
        .response;
    response.interact(Sense::click())
}

fn selection_range_for(
    app: &App,
    block_idx: usize,
    table_cell: Option<(usize, usize)>,
    run_len: usize,
) -> Option<(usize, usize)> {
    let (start, end) = app.selection_bounds()?;
    if start.block != block_idx {
        return None;
    }
    match table_cell {
        Some((row, col)) if start.table_row == row && start.table_col == col => {}
        None if start.table_row == 0 && start.table_col == 0 => {}
        _ => return None,
    }
    Some((start.char.min(run_len), end.char.min(run_len)))
}

fn caret_position_for(
    app: &App,
    block_idx: usize,
    table_cell: Option<(usize, usize)>,
    run_len: usize,
) -> Option<usize> {
    if app.caret.block != block_idx {
        return None;
    }
    match table_cell {
        Some((row, col)) if app.caret.table_row == row && app.caret.table_col == col => {
            Some(app.caret.char.min(run_len))
        }
        None if app.caret.table_row == 0 && app.caret.table_col == 0 => {
            Some(app.caret.char.min(run_len))
        }
        _ => None,
    }
}

fn move_block(app: &mut App, delta: isize) {
    let current = app.caret.block as isize;
    let next = (current + delta).clamp(0, app.doc.blocks.len().saturating_sub(1) as isize) as usize;
    app.caret.block = next;
    app.caret.table_row = 0;
    app.caret.table_col = 0;
    app.doc.clamp_caret(&mut app.caret);
}

fn extend_selection(app: &mut App, direction: MoveDirection) {
    let before = app.caret.clone();
    let anchor = app
        .selection_anchor
        .clone()
        .unwrap_or_else(|| before.clone());
    app.selection_anchor = Some(anchor.clone());
    move_caret(app, direction);
    if !same_edit_target(&anchor, &app.caret) {
        app.caret = before;
    }
    if app.selection_anchor.as_ref() == Some(&app.caret) {
        app.clear_selection();
    }
}

fn collapse_or_move(app: &mut App, direction: MoveDirection) {
    if let Some((start, end)) = app.selection_bounds() {
        app.caret = match direction {
            MoveDirection::Left => start,
            MoveDirection::Right => end,
        };
        app.clear_selection();
        return;
    }
    app.clear_selection();
    move_caret(app, direction);
}

fn move_caret(app: &mut App, direction: MoveDirection) {
    match direction {
        MoveDirection::Left => {
            app.doc.move_left(&mut app.caret);
        }
        MoveDirection::Right => {
            app.doc.move_right(&mut app.caret);
        }
    };
}

fn same_edit_target(a: &Caret, b: &Caret) -> bool {
    a.block == b.block && a.table_row == b.table_row && a.table_col == b.table_col
}

#[derive(Clone, Copy)]
enum MoveDirection {
    Left,
    Right,
}

enum ClickTarget {
    Block(usize),
    TableCell(usize, usize, usize),
}

#[allow(dead_code)]
fn plain_text(runs: &[StyledChar]) -> String {
    chars_to_string(runs)
}
