//! Open / save HTML + autosave to user data dir.

use crate::doc::Doc;
use std::path::PathBuf;

pub fn save_html(doc: &Doc, path: &std::path::Path) -> std::io::Result<()> {
    let s = crate::serialize::serialize_document(doc);
    std::fs::write(path, s)
}

pub fn load_html(path: &std::path::Path) -> std::io::Result<Doc> {
    let s = std::fs::read_to_string(path)?;
    Ok(crate::serialize::parse_html(&s))
}

pub fn pick_save_path(suggested: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("HTML", &["html", "htm"])
        .set_file_name(suggested)
        .save_file()
}

pub fn pick_open_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("HTML", &["html", "htm"])
        .pick_file()
}

pub fn autosave_path() -> Option<PathBuf> {
    crate::paths::data_file("autosave.json")
}

pub fn save_autosave(doc: &Doc) {
    let Some(path) = autosave_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(doc) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn load_autosave() -> Option<Doc> {
    let path = autosave_path()?;
    let s = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&s).ok()
}
