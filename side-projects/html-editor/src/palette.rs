//! Command palette: overlay window with fuzzy-filtered items, keyboard nav, and a prompt mode.

use crate::App;
use eframe::egui;
use eframe::egui::{Context, Key, Modifiers};

#[derive(Default)]
pub struct Palette {
    pub open: bool,
    pub query: String,
    pub items: Vec<Item>,
    pub filtered: Vec<usize>,
    pub active: usize,
    pub mode: Mode,
    pub mode_label: String,
    pub placeholder: String,
    pub should_focus: bool,
    pub prompt: Option<PromptKind>,
    pub table_rows: usize,
    pub table_cols: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Commands,
    Tags,
    SpecialChars,
    Outline,
    Prompt,
    TableGrid,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub hint: String,
    pub group: String,
    pub search: String,
    pub kind: ItemKind,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Command(usize),
    Tag(usize),
    SpecialChar(char),
    OutlineBlock(usize),
}

#[derive(Clone, Debug)]
pub enum PromptKind {
    LinkUrl,
    ImageUrl,
    FontSize,
    LineHeight,
    MaxWidth,
    PageMargin,
    FontFamily,
    Accent,
    TableSize,
}

pub fn open_commands(app: &mut App) {
    let mut items = crate::commands::all_command_items();
    items.extend(crate::commands::all_tag_items());
    app.palette = Palette {
        open: true,
        items,
        mode: Mode::Commands,
        mode_label: "commands".into(),
        placeholder: "Type a command…".into(),
        should_focus: true,
        ..Default::default()
    };
    refilter(app);
}

pub fn open_tags(app: &mut App) {
    app.palette = Palette {
        open: true,
        items: crate::commands::all_tag_items(),
        mode: Mode::Tags,
        mode_label: "html tag".into(),
        placeholder: "Tag name…".into(),
        should_focus: true,
        ..Default::default()
    };
    refilter(app);
}

pub fn open_special_chars(app: &mut App) {
    app.palette = Palette {
        open: true,
        items: crate::commands::all_special_chars(),
        mode: Mode::SpecialChars,
        mode_label: "special character".into(),
        placeholder: "Character name…".into(),
        should_focus: true,
        ..Default::default()
    };
    refilter(app);
}

pub fn open_outline(app: &mut App) {
    let items = crate::commands::outline_items(&app.doc);
    if items.is_empty() {
        app.notify("No headings to outline");
        return;
    }
    app.palette = Palette {
        open: true,
        items,
        mode: Mode::Outline,
        mode_label: "outline".into(),
        placeholder: "Heading…".into(),
        should_focus: true,
        ..Default::default()
    };
    refilter(app);
}

pub fn open_prompt(app: &mut App, kind: PromptKind, label: &str, prefill: &str) {
    app.palette = Palette {
        open: true,
        items: Vec::new(),
        mode: Mode::Prompt,
        mode_label: label.into(),
        placeholder: label.into(),
        should_focus: true,
        prompt: Some(kind),
        query: prefill.into(),
        ..Default::default()
    };
}

pub fn open_table_grid(app: &mut App) {
    app.palette = Palette {
        open: true,
        mode: Mode::TableGrid,
        mode_label: "table".into(),
        placeholder: "Choose table size".into(),
        should_focus: true,
        table_rows: 1,
        table_cols: 1,
        ..Default::default()
    };
}

pub fn close(app: &mut App) {
    app.palette = Palette::default();
}

pub fn refilter(app: &mut App) {
    let q = app.palette.query.to_lowercase();
    let q = q.trim();
    let mut scored: Vec<(usize, i32)> = Vec::new();
    for (i, item) in app.palette.items.iter().enumerate() {
        let score = if q.is_empty() {
            1
        } else {
            fuzzy_score(&item.search.to_lowercase(), q)
        };
        if score > 0 {
            scored.push((i, score));
        }
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    app.palette.filtered = scored.into_iter().map(|(i, _)| i).collect();
    if app.palette.active >= app.palette.filtered.len() {
        app.palette.active = 0;
    }
}

fn fuzzy_score(text: &str, query: &str) -> i32 {
    if text == query {
        return 10_000;
    }
    if text.starts_with(query) {
        return 5_000 - text.len() as i32;
    }
    if text.contains(&format!(" {}", query)) {
        return 3_000 - text.len() as i32;
    }
    if text.contains(query) {
        return 1_000 - text.len() as i32;
    }
    let mut qi = 0;
    let qc: Vec<char> = query.chars().collect();
    for c in text.chars() {
        if qi < qc.len() && c == qc[qi] {
            qi += 1;
        }
    }
    if qi == qc.len() {
        100
    } else {
        0
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        close(app);
        return;
    }

    let is_prompt = app.palette.mode == Mode::Prompt;
    let is_table_grid = app.palette.mode == Mode::TableGrid;

    let accept_pressed = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
    let mut table_accept: Option<(usize, usize)> = None;
    let mut reveal_active = false;

    if is_table_grid {
        let left = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft));
        let right = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight));
        let up = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
        let down = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));
        if left {
            app.palette.table_cols = app.palette.table_cols.saturating_sub(1).max(1);
        }
        if right {
            app.palette.table_cols = (app.palette.table_cols + 1).min(10);
        }
        if up {
            app.palette.table_rows = app.palette.table_rows.saturating_sub(1).max(1);
        }
        if down {
            app.palette.table_rows = (app.palette.table_rows + 1).min(8);
        }
        if accept_pressed {
            table_accept = Some((app.palette.table_rows, app.palette.table_cols));
        }
    } else if !is_prompt {
        let up = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
        let down = ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));
        if up && !app.palette.filtered.is_empty() {
            if app.palette.active == 0 {
                app.palette.active = app.palette.filtered.len() - 1;
            } else {
                app.palette.active -= 1;
            }
            reveal_active = true;
        }
        if down && !app.palette.filtered.is_empty() {
            app.palette.active = (app.palette.active + 1) % app.palette.filtered.len();
            reveal_active = true;
        }
    }

    let mode_label = app.palette.mode_label.clone();
    let placeholder = app.palette.placeholder.clone();
    let mut query_changed = false;
    let mut mouse_accept_idx: Option<usize> = None;

    let screen = ctx.content_rect();
    egui::Window::new("palette_window")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, screen.height() * 0.14])
        .default_width(670.0)
        .show(ctx, |ui| {
            ui.set_min_width(650.0);
            ui.set_max_width(670.0);
            if is_table_grid {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(egui_phosphor::regular::TABLE).size(22.0));
                    ui.label(egui::RichText::new("Table").size(20.0));
                });
                ui.separator();
                let max_rows = 8;
                let max_cols = 10;
                for r in 1..=max_rows {
                    ui.horizontal(|ui| {
                        for c in 1..=max_cols {
                            let active = r <= app.palette.table_rows && c <= app.palette.table_cols;
                            let color = if active {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.bg_fill
                            };
                            let resp = ui.add(
                                egui::Button::new("")
                                    .min_size(egui::vec2(22.0, 22.0))
                                    .fill(color),
                            );
                            if resp.hovered() {
                                app.palette.table_rows = r;
                                app.palette.table_cols = c;
                            }
                            if resp.clicked() {
                                table_accept = Some((r, c));
                            }
                        }
                    });
                }
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} x {}",
                            app.palette.table_rows, app.palette.table_cols
                        ))
                        .size(18.0),
                    );
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(egui_phosphor::regular::MAGNIFYING_GLASS).size(22.0),
                    );
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut app.palette.query)
                            .hint_text(&placeholder)
                            .desired_width(600.0),
                    );
                    if app.palette.should_focus {
                        resp.request_focus();
                        app.palette.should_focus = false;
                    }
                    if resp.changed() {
                        query_changed = true;
                    }
                });
                ui.label(
                    egui::RichText::new(&mode_label)
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
            }
            if !is_prompt && !is_table_grid {
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        if app.palette.filtered.is_empty() {
                            ui.label(
                                egui::RichText::new("No matches")
                                    .color(ui.visuals().weak_text_color()),
                            );
                        } else {
                            let filtered = app.palette.filtered.clone();
                            let active = app.palette.active;
                            for (i, &idx) in filtered.iter().enumerate() {
                                let item = &app.palette.items[idx];
                                let selected = i == active;
                                let resp = palette_row(ui, item, selected);
                                if selected && reveal_active {
                                    resp.scroll_to_me(None);
                                }
                                if resp.clicked() {
                                    mouse_accept_idx = Some(i);
                                }
                            }
                        }
                    });
            } else if is_prompt {
                ui.label(
                    egui::RichText::new("press Enter to confirm")
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
            }
        });

    if query_changed && !is_prompt {
        refilter(app);
    }

    if let Some(i) = mouse_accept_idx {
        app.palette.active = i;
    }

    if accept_pressed || mouse_accept_idx.is_some() {
        if is_prompt {
            let val = app.palette.query.clone();
            let kind = app.palette.prompt.clone();
            close(app);
            if let Some(k) = kind {
                crate::commands::dispatch_prompt(app, k, &val);
            }
        } else if app.palette.active < app.palette.filtered.len() {
            let idx = app.palette.filtered[app.palette.active];
            let item = app.palette.items[idx].clone();
            close(app);
            crate::commands::dispatch_item(app, item);
        }
    }

    if let Some((rows, cols)) = table_accept {
        close(app);
        app.push_history();
        app.doc.insert_table_after(&mut app.caret, rows, cols);
        app.dirty = true;
        app.notify(format!("Inserted {rows} x {cols} table"));
    }
}

fn palette_row(ui: &mut egui::Ui, item: &Item, selected: bool) -> egui::Response {
    let fill = if selected {
        ui.visuals().selection.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::NONE
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(28.0);
                ui.label(egui::RichText::new(item_icon(item)).size(22.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&item.name).size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !item.hint.is_empty() {
                        ui.label(
                            egui::RichText::new(&item.hint)
                                .color(ui.visuals().weak_text_color())
                                .size(17.0),
                        );
                    }
                    ui.label(
                        egui::RichText::new(&item.group)
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                });
            });
        })
        .response
        .interact(egui::Sense::click())
}

fn item_icon(item: &Item) -> &'static str {
    use egui_phosphor::regular as ph;
    match item.group.as_str() {
        "file" => {
            if item.name.contains("Open") {
                ph::FOLDER_OPEN
            } else if item.name.contains("Save") {
                ph::FLOPPY_DISK
            } else {
                ph::FILE_PLUS
            }
        }
        "edit" => ph::PENCIL_SIMPLE,
        "fmt" => {
            if item.name.contains("bold") {
                ph::TEXT_B
            } else if item.name.contains("italic") {
                ph::TEXT_ITALIC
            } else if item.name.contains("underline") {
                ph::TEXT_UNDERLINE
            } else if item.name.contains("code") {
                ph::CODE
            } else {
                ph::TEXT_AA
            }
        }
        "insert" => {
            if item.name.contains("Table") {
                ph::TABLE
            } else if item.name.contains("Image") {
                ph::IMAGE
            } else if item.name.contains("Link") {
                ph::LINK
            } else {
                ph::PLUS
            }
        }
        "table" => ph::TABLE,
        "view" => ph::MAGNIFYING_GLASS,
        "theme" | "style" => ph::PALETTE,
        "block" => ph::ARTICLE,
        "tag" => tag_icon(&item.hint),
        "char" => ph::TEXT_T,
        "outline" => ph::LIST_MAGNIFYING_GLASS,
        _ => ph::COMMAND,
    }
}

fn tag_icon(hint: &str) -> &'static str {
    use egui_phosphor::regular as ph;
    match hint {
        "<h1>" | "<h2>" | "<h3>" | "<h4>" | "<h5>" | "<h6>" => ph::TEXT_H,
        "<ul>" => ph::LIST_BULLETS,
        "<ol>" => ph::LIST_NUMBERS,
        "<table>" | "<tr>" | "<td>" | "<th>" => ph::TABLE,
        "<blockquote>" => ph::QUOTES,
        "<pre>" | "<code>" => ph::CODE,
        "<a>" => ph::LINK,
        "<img>" => ph::IMAGE,
        _ => ph::FILE_HTML,
    }
}
