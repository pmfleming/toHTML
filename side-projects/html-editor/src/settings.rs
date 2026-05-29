//! Settings: theme, font size, max width, accent — persisted to user config dir.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String, // "light" | "dark" | "sepia"
    pub font_size: f32,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    pub max_width: f32,
    #[serde(default = "default_page_margin")]
    pub page_margin: f32,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    pub accent: [u8; 3],
    #[serde(default = "default_show_tags")]
    pub show_tags: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            font_size: 17.0,
            line_height: default_line_height(),
            max_width: 760.0,
            page_margin: default_page_margin(),
            font_family: default_font_family(),
            accent: [59, 109, 240],
            show_tags: default_show_tags(),
        }
    }
}

fn default_line_height() -> f32 {
    1.55
}
fn default_page_margin() -> f32 {
    28.0
}
fn default_font_family() -> String {
    "Georgia, Times New Roman, serif".into()
}
fn default_show_tags() -> bool {
    true
}

pub fn settings_path() -> Option<std::path::PathBuf> {
    crate::paths::config_file("settings.json")
}

pub fn load() -> Settings {
    let Some(p) = settings_path() else {
        return Settings::default();
    };
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(s: &Settings) {
    let Some(p) = settings_path() else {
        return;
    };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(p, json);
    }
}

pub fn apply_visuals(ctx: &eframe::egui::Context, settings: &Settings) {
    use eframe::egui::{Color32, Visuals};
    let mut v = match settings.theme.as_str() {
        "dark" => Visuals::dark(),
        "sepia" => sepia(),
        _ => Visuals::light(),
    };
    v.hyperlink_color =
        Color32::from_rgb(settings.accent[0], settings.accent[1], settings.accent[2]);
    ctx.set_visuals(v);
}

fn sepia() -> eframe::egui::Visuals {
    use eframe::egui::{Color32, Visuals};
    let mut v = Visuals::light();
    v.window_fill = Color32::from_rgb(0xfb, 0xf6, 0xe9);
    v.panel_fill = Color32::from_rgb(0xf6, 0xef, 0xde);
    v.extreme_bg_color = Color32::from_rgb(0xf6, 0xef, 0xde);
    v.faint_bg_color = Color32::from_rgb(0xee, 0xe4, 0xc8);
    v.override_text_color = Some(Color32::from_rgb(0x4a, 0x3a, 0x26));
    v.hyperlink_color = Color32::from_rgb(0xa6, 0x68, 0x32);
    v
}
