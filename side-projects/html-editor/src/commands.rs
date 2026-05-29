//! Command catalog + per-command implementations + global-key dispatcher.

use crate::doc::{Block, Doc, InlineStyle};
use crate::palette::{Item, ItemKind, PromptKind};
use crate::App;

#[derive(Clone, Copy)]
pub struct Command {
    pub group: &'static str,
    pub name: &'static str,
    pub hint: &'static str,
    pub run: fn(&mut App),
}

mod catalog;
mod keys;
mod prompts;
pub use catalog::{COMMANDS, SPECIAL_CHARS, TAG_CATALOG};
pub use keys::handle_global_keys;
pub use prompts::dispatch_prompt;

pub fn all_command_items() -> Vec<Item> {
    COMMANDS
        .iter()
        .enumerate()
        .map(|(i, c)| Item {
            name: c.name.to_string(),
            hint: c.hint.to_string(),
            group: c.group.to_string(),
            search: format!("{} {} {}", c.group, c.name, c.hint),
            kind: ItemKind::Command(i),
        })
        .collect()
}

pub fn all_tag_items() -> Vec<Item> {
    TAG_CATALOG
        .iter()
        .enumerate()
        .map(|(i, (tag, label))| Item {
            name: format!("{tag} — {label}"),
            hint: format!("<{tag}>"),
            group: "tag".into(),
            search: format!("{tag} {label}"),
            kind: ItemKind::Tag(i),
        })
        .collect()
}

pub fn all_special_chars() -> Vec<Item> {
    SPECIAL_CHARS
        .iter()
        .map(|(name, ch)| Item {
            name: format!("{ch}    {name}"),
            hint: String::new(),
            group: "char".into(),
            search: name.to_string(),
            kind: ItemKind::SpecialChar(*ch),
        })
        .collect()
}

pub fn outline_items(doc: &Doc) -> Vec<Item> {
    let mut items = Vec::new();
    for (i, b) in doc.blocks.iter().enumerate() {
        if let Block::Heading(lvl, runs) = b {
            let text: String = runs.iter().map(|c| c.ch).collect();
            let indent = " ".repeat((*lvl as usize - 1) * 2);
            items.push(Item {
                name: format!(
                    "{indent}h{lvl}  {}",
                    if text.is_empty() {
                        "(empty)".into()
                    } else {
                        text.clone()
                    }
                ),
                hint: String::new(),
                group: "outline".into(),
                search: text,
                kind: ItemKind::OutlineBlock(i),
            });
        }
    }
    items
}

pub fn dispatch_item(app: &mut App, item: Item) {
    match item.kind {
        ItemKind::Command(i) => (COMMANDS[i].run)(app),
        ItemKind::Tag(i) => apply_tag(app, i),
        ItemKind::SpecialChar(c) => {
            let text = c.to_string();
            app.commit_edit(|app| {
                let style = app.current_style.clone();
                app.doc.insert_text(&mut app.caret, &text, &style)
            });
        }
        ItemKind::OutlineBlock(i) => {
            app.caret.block = i;
            app.caret.char = 0;
            app.notify("Jumped to heading");
        }
    }
}

pub fn apply_tag(app: &mut App, idx: usize) {
    let (tag, _) = TAG_CATALOG[idx];
    match tag {
        "h1" => cmd_to_h1(app),
        "h2" => cmd_to_h2(app),
        "h3" => cmd_to_h3(app),
        "h4" => cmd_to_h4(app),
        "h5" => {
            app.push_history();
            app.doc
                .transform_block_to(app.caret.block, |r| Block::Heading(5, r));
            app.dirty = true;
        }
        "h6" => {
            app.push_history();
            app.doc
                .transform_block_to(app.caret.block, |r| Block::Heading(6, r));
            app.dirty = true;
        }
        "p" => cmd_to_p(app),
        "blockquote" => cmd_to_blockquote(app),
        "ul" => cmd_to_bullet(app),
        "ol" => cmd_to_numbered(app),
        "pre" => cmd_to_pre(app),
        "table" => cmd_table(app),
        "tr" => cmd_table_row(app),
        "td" => cmd_table_col(app),
        "th" => cmd_table(app),
        "hr" => cmd_hr(app),
        "a" => cmd_link(app),
        "img" => cmd_image_url(app),
        "strong" => cmd_toggle_bold(app),
        "em" => cmd_toggle_italic(app),
        "u" => cmd_toggle_underline(app),
        "s" => cmd_toggle_strike(app),
        "code" => cmd_toggle_code(app),
        _ => {}
    }
}

// -------- Command implementations --------

fn cmd_new(app: &mut App) {
    app.push_history();
    app.doc = Doc::empty_paragraph();
    app.caret = crate::doc::Caret::default();
    app.clear_selection();
    app.file_path = None;
    app.dirty = false;
    app.notify("New document");
}

fn cmd_open(app: &mut App) {
    if let Some(path) = crate::files::pick_open_path() {
        match crate::files::load_html(&path) {
            Ok(doc) => {
                app.push_history();
                app.doc = doc;
                app.caret = crate::doc::Caret::default();
                app.clear_selection();
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                app.file_path = Some(path);
                app.dirty = false;
                app.notify(format!("Opened {name}"));
            }
            Err(e) => app.notify(format!("Open failed: {e}")),
        }
    }
}

fn cmd_save(app: &mut App) {
    do_save(app, false);
}
fn cmd_save_as(app: &mut App) {
    do_save(app, true);
}
fn do_save(app: &mut App, force_as: bool) {
    let path = if force_as || app.file_path.is_none() {
        let suggested = app
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document.html".to_string());
        crate::files::pick_save_path(&suggested)
    } else {
        app.file_path.clone()
    };
    let Some(path) = path else {
        return;
    };
    match crate::files::save_html(&app.doc, &path) {
        Ok(_) => {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            app.file_path = Some(path);
            app.dirty = false;
            app.notify(format!("Saved {name}"));
        }
        Err(e) => app.notify(format!("Save failed: {e}")),
    }
}

fn cmd_undo(app: &mut App) {
    app.undo();
}
fn cmd_redo(app: &mut App) {
    app.redo();
}

fn cmd_toggle_bold(app: &mut App) {
    if apply_style_to_selection(app, |style| style.bold ^= true) {
        app.notify("Bold toggled for selection");
        return;
    }
    app.current_style.bold ^= true;
    app.notify(if app.current_style.bold {
        "Bold on"
    } else {
        "Bold off"
    });
}
fn cmd_toggle_italic(app: &mut App) {
    if apply_style_to_selection(app, |style| style.italic ^= true) {
        app.notify("Italic toggled for selection");
        return;
    }
    app.current_style.italic ^= true;
    app.notify(if app.current_style.italic {
        "Italic on"
    } else {
        "Italic off"
    });
}
fn cmd_toggle_underline(app: &mut App) {
    if apply_style_to_selection(app, |style| style.underline ^= true) {
        app.notify("Underline toggled for selection");
        return;
    }
    app.current_style.underline ^= true;
    app.notify(if app.current_style.underline {
        "Underline on"
    } else {
        "Underline off"
    });
}
fn cmd_toggle_strike(app: &mut App) {
    if apply_style_to_selection(app, |style| style.strike ^= true) {
        app.notify("Strikethrough toggled for selection");
        return;
    }
    app.current_style.strike ^= true;
    app.notify(if app.current_style.strike {
        "Strikethrough on"
    } else {
        "Strikethrough off"
    });
}
fn cmd_toggle_code(app: &mut App) {
    if apply_style_to_selection(app, |style| style.code ^= true) {
        app.notify("Inline code toggled for selection");
        return;
    }
    app.current_style.code ^= true;
    app.notify(if app.current_style.code {
        "Inline code on"
    } else {
        "Inline code off"
    });
}
fn cmd_clear_style(app: &mut App) {
    if apply_style_to_selection(app, |style| *style = InlineStyle::default()) {
        app.notify("Selection style cleared");
        return;
    }
    app.current_style = InlineStyle::default();
    app.notify("Style cleared");
}

fn apply_style_to_selection(app: &mut App, toggle: impl Fn(&mut InlineStyle) + Copy) -> bool {
    let Some((start, end)) = app.selection_bounds() else {
        return false;
    };
    app.commit_edit(|app| app.doc.apply_style_range(&start, &end, toggle))
}

fn cmd_h1(app: &mut App) {
    insert_block_after(app, Block::Heading(1, vec![]));
}
fn cmd_h2(app: &mut App) {
    insert_block_after(app, Block::Heading(2, vec![]));
}
fn cmd_h3(app: &mut App) {
    insert_block_after(app, Block::Heading(3, vec![]));
}
fn cmd_h4(app: &mut App) {
    insert_block_after(app, Block::Heading(4, vec![]));
}
fn cmd_p(app: &mut App) {
    insert_block_after(app, Block::Paragraph(vec![]));
}
fn cmd_blockquote(app: &mut App) {
    insert_block_after(app, Block::Blockquote(vec![]));
}
fn cmd_bullet(app: &mut App) {
    insert_block_after(app, Block::Bullet(vec![]));
}
fn cmd_numbered(app: &mut App) {
    insert_block_after(app, Block::Numbered(vec![]));
}
fn cmd_pre(app: &mut App) {
    insert_block_after(app, Block::Pre(vec![]));
}
fn cmd_hr(app: &mut App) {
    insert_block_after(app, Block::Hr);
}
fn cmd_table(app: &mut App) {
    crate::palette::open_table_grid(app);
}
fn cmd_table_row(app: &mut App) {
    app.push_history();
    if app.doc.add_table_row(&mut app.caret) {
        app.dirty = true;
        app.notify("Added table row");
    } else {
        app.undo_history_only();
        app.notify("Place the cursor in a table first");
    }
}
fn cmd_table_col(app: &mut App) {
    app.push_history();
    if app.doc.add_table_col(&mut app.caret) {
        app.dirty = true;
        app.notify("Added table column");
    } else {
        app.undo_history_only();
        app.notify("Place the cursor in a table first");
    }
}

fn insert_block_after(app: &mut App, block: Block) {
    app.push_history();
    let pos = app.caret.block + 1;
    let editable = !matches!(block, Block::Hr);
    app.doc.blocks.insert(pos, block);
    if editable {
        app.caret.block = pos;
        app.caret.char = 0;
    } else if app.caret.block + 1 >= app.doc.blocks.len() {
        app.doc.blocks.push(Block::Paragraph(vec![]));
        app.caret.block = app.doc.blocks.len() - 1;
        app.caret.char = 0;
    }
    app.dirty = true;
}

fn cmd_link(app: &mut App) {
    crate::palette::open_prompt(app, PromptKind::LinkUrl, "Link URL", "https://");
}
fn cmd_image_url(app: &mut App) {
    crate::palette::open_prompt(app, PromptKind::ImageUrl, "Image URL", "");
}
fn cmd_special_char(app: &mut App) {
    crate::palette::open_special_chars(app);
}
fn cmd_outline(app: &mut App) {
    crate::palette::open_outline(app);
}
fn cmd_toggle_tags(app: &mut App) {
    app.settings.show_tags ^= true;
    crate::settings::save(&app.settings);
}
fn cmd_help(app: &mut App) {
    app.show_help = true;
}

fn cmd_theme_light(app: &mut App) {
    app.settings.theme = "light".into();
    crate::settings::save(&app.settings);
    app.queue_visuals = true;
}
fn cmd_theme_dark(app: &mut App) {
    app.settings.theme = "dark".into();
    crate::settings::save(&app.settings);
    app.queue_visuals = true;
}
fn cmd_theme_sepia(app: &mut App) {
    app.settings.theme = "sepia".into();
    crate::settings::save(&app.settings);
    app.queue_visuals = true;
}

fn cmd_font_size(app: &mut App) {
    let cur = app.settings.font_size.to_string();
    crate::palette::open_prompt(app, PromptKind::FontSize, "Font size (e.g. 17)", &cur);
}
fn cmd_line_height(app: &mut App) {
    let cur = app.settings.line_height.to_string();
    crate::palette::open_prompt(app, PromptKind::LineHeight, "Line height (e.g. 1.55)", &cur);
}
fn cmd_max_width(app: &mut App) {
    let cur = app.settings.max_width.to_string();
    crate::palette::open_prompt(app, PromptKind::MaxWidth, "Max width (px)", &cur);
}
fn cmd_page_margin(app: &mut App) {
    let cur = app.settings.page_margin.to_string();
    crate::palette::open_prompt(app, PromptKind::PageMargin, "Page margin (px)", &cur);
}
fn cmd_font_family(app: &mut App) {
    let cur = app.settings.font_family.clone();
    crate::palette::open_prompt(app, PromptKind::FontFamily, "Font family", &cur);
}
fn cmd_accent(app: &mut App) {
    let [r, g, b] = app.settings.accent;
    crate::palette::open_prompt(
        app,
        PromptKind::Accent,
        "Accent color (#RRGGBB)",
        &format!("#{r:02x}{g:02x}{b:02x}"),
    );
}
fn cmd_reset_styles(app: &mut App) {
    app.settings = crate::settings::Settings::default();
    crate::settings::save(&app.settings);
    app.queue_visuals = true;
    app.notify("Styles reset");
}

fn cmd_to_p(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, Block::Paragraph);
    app.dirty = true;
}
fn cmd_to_h1(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, |r| Block::Heading(1, r));
    app.dirty = true;
}
fn cmd_to_h2(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, |r| Block::Heading(2, r));
    app.dirty = true;
}
fn cmd_to_h3(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, |r| Block::Heading(3, r));
    app.dirty = true;
}
fn cmd_to_h4(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, |r| Block::Heading(4, r));
    app.dirty = true;
}
fn cmd_to_blockquote(app: &mut App) {
    app.push_history();
    app.doc
        .transform_block_to(app.caret.block, Block::Blockquote);
    app.dirty = true;
}
fn cmd_to_bullet(app: &mut App) {
    app.push_history();
    app.doc.transform_block_to(app.caret.block, Block::Bullet);
    app.dirty = true;
}
fn cmd_to_numbered(app: &mut App) {
    app.push_history();
    app.doc.transform_block_to(app.caret.block, Block::Numbered);
    app.dirty = true;
}
fn cmd_to_pre(app: &mut App) {
    app.push_history();
    app.doc.transform_block_to(app.caret.block, Block::Pre);
    app.dirty = true;
}

fn cmd_delete_block(app: &mut App) {
    if app.doc.blocks.len() <= 1 {
        app.notify("Cannot delete the only block");
        return;
    }
    app.push_history();
    app.doc.blocks.remove(app.caret.block);
    if app.caret.block >= app.doc.blocks.len() {
        app.caret.block = app.doc.blocks.len() - 1;
    }
    app.caret.char = 0;
    app.dirty = true;
}
