use std::path::PathBuf;

use html_editor::{rich_text_preview, serialize};

fn main() {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: editor-preview <html-file>");
        std::process::exit(2);
    };

    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    let doc = serialize::parse_html(&source);
    print!("{}", rich_text_preview::render_editor_preview_html(&doc));
}
