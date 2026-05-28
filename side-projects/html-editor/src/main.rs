mod commands;
mod doc;
mod files;
mod palette;
#[path = "../../rich_text/mod.rs"]
mod rich_text;
mod serialize;
mod settings;

use std::path::PathBuf;

use doc::{Caret, Doc, InlineStyle};
use eframe::egui;
use palette::Palette;

pub struct App {
    pub doc: Doc,
    pub caret: Caret,
    pub selection_anchor: Option<Caret>,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    pub current_style: InlineStyle,
    pub palette: Palette,
    pub settings: settings::Settings,
    pub history: Vec<Doc>,
    pub redo_stack: Vec<Doc>,
    pub notification: Option<String>,
    pub show_help: bool,
    pub queue_visuals: bool,
    pub editor_has_focus: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        let settings = settings::load();
        settings::apply_visuals(&cc.egui_ctx, &settings);
        let doc = files::load_autosave().unwrap_or_else(Doc::welcome);
        Self {
            doc,
            caret: Caret::default(),
            selection_anchor: None,
            file_path: None,
            dirty: false,
            current_style: InlineStyle::default(),
            palette: Palette::default(),
            settings,
            history: Vec::new(),
            redo_stack: Vec::new(),
            notification: None,
            show_help: false,
            queue_visuals: false,
            editor_has_focus: true,
        }
    }

    pub fn push_history(&mut self) {
        self.history.push(self.doc.clone());
        self.redo_stack.clear();
        if self.history.len() > 200 {
            self.history.remove(0);
        }
    }

    pub fn undo_history_only(&mut self) {
        self.history.pop();
    }

    pub fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.redo_stack.push(self.doc.clone());
            self.doc = previous;
            self.doc.clamp_caret(&mut self.caret);
            self.clear_selection();
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.history.push(self.doc.clone());
            self.doc = next;
            self.doc.clamp_caret(&mut self.caret);
            self.clear_selection();
            self.dirty = true;
        }
    }

    pub fn notify(&mut self, message: impl Into<String>) {
        self.notification = Some(message.into());
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn selection_bounds(&self) -> Option<(Caret, Caret)> {
        let anchor = self.selection_anchor.as_ref()?;
        if !same_edit_target(anchor, &self.caret) || anchor.char == self.caret.char {
            return None;
        }
        if anchor.char < self.caret.char {
            Some((anchor.clone(), self.caret.clone()))
        } else {
            Some((self.caret.clone(), anchor.clone()))
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_bounds() else {
            self.clear_selection();
            return false;
        };
        let Some(runs) = self.doc.current_inline_mut(&start) else {
            self.clear_selection();
            return false;
        };
        runs.drain(start.char..end.char);
        self.caret = start;
        self.clear_selection();
        self.dirty = true;
        true
    }

    fn title(&self) -> String {
        let name = self
            .file_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".into());
        if self.dirty {
            format!("{name} *")
        } else {
            name
        }
    }

    fn show_help(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help;
        egui::Window::new("Shortcuts")
            .open(&mut open)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Ctrl+T opens the filtering command palette.");
                ui.label("< opens the filtering HTML tag palette.");
                ui.label("Enter splits the current block. Backspace joins at the start of a block.");
                ui.label("All file, style, theme, table, and tag commands are available from the palette.");
        });
        self.show_help = open;
    }
}

fn same_edit_target(a: &Caret, b: &Caret) -> bool {
    a.block == b.block && a.table_row == b.table_row && a.table_col == b.table_col
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.queue_visuals {
            settings::apply_visuals(&ctx, &self.settings);
            self.queue_visuals = false;
        }

        commands::handle_global_keys(self, &ctx);
        if self.palette.open {
            palette::show(self, &ctx);
        } else {
            rich_text::handle_input(self, &ctx);
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let available = ui.available_width();
            let page_width = self.settings.max_width.min(available).max(320.0);
            ui.vertical_centered(|ui| {
                ui.set_width(page_width);
                ui.add_space(self.settings.page_margin);
                rich_text::show(self, ui);
                ui.add_space(self.settings.page_margin);
            });
        });

        if self.show_help {
            self.show_help(&ctx);
        }

        if let Some(message) = &self.notification {
            egui::Area::new("notification".into())
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -18.0])
                .show(&ctx, |ui| {
                    ui.label(
                        egui::RichText::new(message)
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                });
        }

        files::save_autosave(&self.doc);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "HTML Editor - {}",
            self.title()
        )));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "HTML Editor",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
