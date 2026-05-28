//! A tiny egui-native rich text editor surface.
//!
//! This lives outside `html-editor` so the custom widget can evolve separately
//! from the application shell and command palette.

use crate::doc::{chars_to_string, Block, Caret, StyledChar};
use crate::{palette, App};

mod render;

use eframe::egui::{self, Context, Event, Key, Modifiers, RichText, Sense, Ui};
use render::{render_inline_tagged_runs, render_runs_with_selection};

pub fn handle_input(app: &mut App, ctx: &Context) {
    if !app.editor_has_focus {
        return;
    }

    let mut inserted = String::new();
    for event in ctx.input(|i| i.events.clone()) {
        if let Event::Text(text) = event {
            if text == "<" {
                palette::open_tags(app);
                return;
            }
            if !text.chars().all(|c| c.is_control()) {
                inserted.push_str(&text);
            }
        }
    }

    if !inserted.is_empty() {
        app.push_history();
        app.delete_selection();
        let style = app.current_style.clone();
        app.doc.insert_text(&mut app.caret, &inserted, &style);
        app.dirty = true;
        return;
    }

    let backspace = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Backspace));
    let delete = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete));
    let enter = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    let shift_left = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::ArrowLeft));
    let shift_right = ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::ArrowRight));
    let left = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft));
    let right = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight));
    let up = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
    let down = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));

    if backspace {
        app.push_history();
        if !app.delete_selection() {
            app.doc.backspace(&mut app.caret);
            app.clear_selection();
        }
        app.dirty = true;
    } else if delete {
        app.push_history();
        if !app.delete_selection() {
            app.doc.delete_forward(&mut app.caret);
            app.clear_selection();
        }
        app.dirty = true;
    } else if enter {
        app.push_history();
        app.delete_selection();
        app.doc.split_block(&mut app.caret);
        app.clear_selection();
        app.dirty = true;
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
        move_block(app, -1);
    } else if down {
        app.clear_selection();
        move_block(app, 1);
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let font_size = app.settings.font_size;
    let line_height = app.settings.line_height;
    let mut clicked: Option<ClickTarget> = None;

    for block_idx in 0..app.doc.blocks.len() {
        let response = match app.doc.blocks[block_idx].clone() {
            Block::Heading(level, runs) => show_runs(
                app,
                ui,
                block_idx,
                None,
                &runs,
                font_size + (7.0 - level as f32).max(1.0) * 2.2,
                line_height,
                "",
            ),
            Block::Paragraph(runs) => {
                show_runs(app, ui, block_idx, None, &runs, font_size, line_height, "")
            }
            Block::Blockquote(runs) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("|").color(ui.visuals().weak_text_color()));
                    show_runs(app, ui, block_idx, None, &runs, font_size, line_height, "")
                })
                .inner
            }
            Block::Bullet(runs) => show_runs(
                app,
                ui,
                block_idx,
                None,
                &runs,
                font_size,
                line_height,
                "* ",
            ),
            Block::Numbered(runs) => show_runs(
                app,
                ui,
                block_idx,
                None,
                &runs,
                font_size,
                line_height,
                "1. ",
            ),
            Block::Pre(runs) => show_runs(app, ui, block_idx, None, &runs, font_size, 1.25, "    "),
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
                                    line_height,
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

fn show_runs(
    app: &App,
    ui: &mut Ui,
    block_idx: usize,
    table_cell: Option<(usize, usize)>,
    runs: &[StyledChar],
    font_size: f32,
    _line_height: f32,
    prefix: &str,
) -> egui::Response {
    let caret_pos = caret_position_for(app, block_idx, table_cell, runs.len());
    let selection = selection_range_for(app, block_idx, table_cell, runs.len());
    let response = if let Some(pos) = caret_pos {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if !prefix.is_empty() {
                ui.label(RichText::new(prefix).size(font_size));
            }
            render_inline_tagged_runs(
                app,
                &app.doc.blocks[block_idx],
                ui,
                runs,
                font_size,
                selection,
                pos,
                table_cell,
            );
        })
        .response
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            if !prefix.is_empty() {
                ui.label(RichText::new(prefix).size(font_size));
            }
            match caret_pos {
                Some(pos) => {
                    render_runs_with_selection(app, ui, runs, font_size, selection, Some(pos));
                }
                None => {
                    if runs.is_empty() {
                        ui.label(RichText::new(" ").size(font_size));
                    } else {
                        render_runs_with_selection(app, ui, runs, font_size, selection, None);
                    }
                }
            }
        })
        .response
    };
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
        MoveDirection::Left => app.doc.move_left(&mut app.caret),
        MoveDirection::Right => app.doc.move_right(&mut app.caret),
    }
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
