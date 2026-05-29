use crate::palette::PromptKind;
use crate::App;

pub fn dispatch_prompt(app: &mut App, kind: PromptKind, value: &str) {
    let v = value.trim();
    match kind {
        PromptKind::LinkUrl => {
            if v.is_empty() {
                return;
            }
            let value = v.to_string();
            app.commit_edit(|app| {
                let mut style = app.current_style.clone();
                style.link = Some(value.clone());
                app.doc.insert_text(&mut app.caret, &value, &style)
            });
        }
        PromptKind::ImageUrl => {
            let marker = format!("[image: {}]", v);
            if app.commit_edit(|app| {
                app.doc
                    .insert_text(&mut app.caret, &marker, &Default::default())
            }) {
                app.notify("Image stored as marker (no image rendering in v1)");
            }
        }
        PromptKind::FontSize => {
            if let Ok(n) = v.parse::<f32>() {
                app.settings.font_size = n.clamp(8.0, 96.0);
                crate::settings::save(&app.settings);
            }
        }
        PromptKind::LineHeight => {
            if let Ok(n) = v.parse::<f32>() {
                app.settings.line_height = n.clamp(1.0, 2.4);
                crate::settings::save(&app.settings);
            }
        }
        PromptKind::MaxWidth => {
            if let Ok(n) = v.parse::<f32>() {
                app.settings.max_width = n.clamp(300.0, 2400.0);
                crate::settings::save(&app.settings);
            }
        }
        PromptKind::PageMargin => {
            if let Ok(n) = v.parse::<f32>() {
                app.settings.page_margin = n.clamp(0.0, 160.0);
                crate::settings::save(&app.settings);
            }
        }
        PromptKind::FontFamily => {
            if !v.is_empty() {
                app.settings.font_family = v.to_string();
                crate::settings::save(&app.settings);
            }
        }
        PromptKind::Accent => {
            if let Some(c) = parse_color(v) {
                app.settings.accent = c;
                crate::settings::save(&app.settings);
                app.queue_visuals = true;
            }
        }
        PromptKind::TableSize => {
            let (rows, cols) = parse_table_size(v).unwrap_or((3, 3));
            app.push_history();
            app.doc.insert_table_after(&mut app.caret, rows, cols);
            app.dirty = true;
            app.notify(format!("Inserted {rows} x {cols} table"));
        }
    }
}

fn parse_table_size(s: &str) -> Option<(usize, usize)> {
    let normalized = s.replace(['X', ','], "x").replace(' ', "");
    let (rows, cols) = normalized.split_once('x')?;
    Some((rows.parse().ok()?, cols.parse().ok()?))
}

fn parse_color(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        Some([
            u8::from_str_radix(&s[0..2], 16).ok()?,
            u8::from_str_radix(&s[2..4], 16).ok()?,
            u8::from_str_radix(&s[4..6], 16).ok()?,
        ])
    } else {
        None
    }
}
