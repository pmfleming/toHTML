use crate::App;
use eframe::egui::{Context, Key, Modifiers};

// === Global shortcuts ===
pub fn handle_global_keys(app: &mut App, ctx: &Context) {
    let want_palette = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::T));
    let want_new = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::N));
    let want_open = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::O));
    let want_save = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S));
    let want_save_as =
        ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S));
    let want_undo = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z));
    let want_redo = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z));
    let want_redo_y = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Y));
    let want_bold = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::B));
    let want_italic = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::I));
    let want_underline = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::U));
    let want_help = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Slash));

    if want_palette {
        crate::palette::open_commands(app);
    }
    if want_new {
        super::cmd_new(app);
    }
    if want_open {
        super::cmd_open(app);
    }
    if want_save {
        super::cmd_save(app);
    }
    if want_save_as {
        super::cmd_save_as(app);
    }
    if want_undo {
        app.undo();
    }
    if want_redo || want_redo_y {
        app.redo();
    }
    if want_bold {
        super::cmd_toggle_bold(app);
    }
    if want_italic {
        super::cmd_toggle_italic(app);
    }
    if want_underline {
        super::cmd_toggle_underline(app);
    }
    if want_help {
        app.show_help = true;
    }
}
