mod choices;
mod status;

use std::path::PathBuf;

use eframe::egui;
use rfd::FileDialog;

use super::{convert_file, default_output_name, default_output_path, CliError};
use choices::FormatChoice;
use status::Status;

pub(super) fn run() -> Result<(), CliError> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 290.0])
            .with_min_inner_size([460.0, 250.0]),
        ..Default::default()
    };

    eframe::run_native(
        "toHTML picker",
        options,
        Box::new(|_cc| Ok(Box::<PickerApp>::default())),
    )
    .map_err(|error| CliError::Interactive(error.to_string()))
}

#[derive(Default)]
struct PickerApp {
    input: String,
    output: String,
    asset_dir: String,
    format: FormatChoice,
    status: Status,
}

impl eframe::App for PickerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("toHTML picker");
            });
            ui.add_space(8.0);

            self.path_row(ui, "Input", true);
            self.path_row(ui, "Output", false);
            self.asset_row(ui);
            self.format_row(ui);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Convert").clicked() {
                    self.convert();
                }
                if ui.button("Clear").clicked() {
                    *self = Self::default();
                }
            });

            ui.add_space(8.0);
            self.status.show(ui);
        });
    }
}

impl PickerApp {
    fn path_row(&mut self, ui: &mut egui::Ui, label: &str, input: bool) {
        ui.horizontal(|ui| {
            ui.label(label);
            let value = if input {
                &mut self.input
            } else {
                &mut self.output
            };
            ui.text_edit_singleline(value);

            if ui.button("Pick").clicked() {
                if input {
                    self.pick_input();
                } else {
                    self.pick_output();
                }
            }
        });
    }

    fn asset_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Assets");
            ui.text_edit_singleline(&mut self.asset_dir);
            if ui.button("Pick").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.asset_dir = display_path(path);
                }
            }
        });
    }

    fn format_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Format");
            egui::ComboBox::from_id_salt("format")
                .selected_text(self.format.label())
                .show_ui(ui, |ui| {
                    for choice in FormatChoice::all() {
                        ui.selectable_value(&mut self.format, choice, choice.label());
                    }
                });
        });
    }

    fn pick_input(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Documents", &["md", "markdown", "docx", "pdf"])
            .pick_file()
        {
            self.input = display_path(path.clone());
            self.format = FormatChoice::from_path(&path).unwrap_or(FormatChoice::Auto);
            if self.output.trim().is_empty() {
                self.output = display_path(default_output_path(&path));
            }
        }
    }

    fn pick_output(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("HTML", &["html", "htm"])
            .set_directory("output")
            .set_file_name(default_output_name_for_picker(&self.input))
            .save_file()
        {
            self.output = display_path(path);
        }
    }

    fn convert(&mut self) {
        let Some(input) = non_empty_path(&self.input) else {
            self.status = Status::error("Choose an input file");
            return;
        };
        let Some(output) = non_empty_path(&self.output) else {
            self.status = Status::error("Choose an output HTML file");
            return;
        };
        let asset_dir = non_empty_path(&self.asset_dir);

        match convert_file(
            &input,
            self.format.format(),
            Some(&output),
            asset_dir.as_deref(),
        ) {
            Ok(()) => self.status = Status::ok(format!("Wrote {}", output.display())),
            Err(error) => self.status = Status::error(error.to_string()),
        }
    }
}

fn non_empty_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

fn display_path(path: PathBuf) -> String {
    path.display().to_string()
}

fn default_output_name_for_picker(input: &str) -> String {
    non_empty_path(input)
        .map(|path| default_output_name(&path))
        .unwrap_or_else(|| "output.html".to_string())
}
