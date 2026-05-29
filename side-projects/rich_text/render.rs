use crate::doc::{group_runs, Block, InlineStyle, StyledChar};
use crate::App;
use eframe::egui::{Color32, RichText, Ui};

fn render_run_slice(ui: &mut Ui, runs: &[StyledChar], font_size: f32) {
    for (text, style) in group_runs(runs) {
        let mut text = RichText::new(text).size(font_size);
        if style.bold {
            text = text.strong();
        }
        if style.italic {
            text = text.italics();
        }
        if style.underline {
            text = text.underline();
        }
        if style.strike {
            text = text.strikethrough();
        }
        if style.code {
            text = text.monospace().background_color(Color32::from_gray(230));
        }
        if style.link.is_some() {
            text = text.color(ui.visuals().hyperlink_color).underline();
        }
        ui.label(text);
    }
}

fn render_selected_run_slice(ui: &mut Ui, runs: &[StyledChar], font_size: f32) {
    for (text, style) in group_runs(runs) {
        let mut text = RichText::new(text)
            .size(font_size)
            .background_color(ui.visuals().selection.bg_fill)
            .color(ui.visuals().selection.stroke.color);
        if style.bold {
            text = text.strong();
        }
        if style.italic {
            text = text.italics();
        }
        if style.underline {
            text = text.underline();
        }
        if style.strike {
            text = text.strikethrough();
        }
        if style.code {
            text = text.monospace();
        }
        ui.label(text);
    }
}

#[derive(Clone, PartialEq, Eq)]
enum VisualKind {
    Text(InlineStyle),
    Tag,
}

#[derive(Clone)]
struct VisualChar {
    ch: char,
    kind: VisualKind,
}

pub(super) struct TaggedRuns<'a> {
    pub app: &'a App,
    pub ui: &'a mut Ui,
    pub block: &'a Block,
    pub runs: &'a [StyledChar],
    pub font_size: f32,
    pub selection: Option<(usize, usize)>,
    pub caret_pos: usize,
    pub table_cell: Option<(usize, usize)>,
}

pub(super) fn render_inline_tagged_runs(args: TaggedRuns<'_>) {
    let TaggedRuns {
        app,
        ui,
        block,
        runs,
        font_size,
        selection,
        caret_pos,
        table_cell,
    } = args;
    let (visual, boundaries) = build_tagged_visual(block, runs, table_cell);
    let caret_pos = boundaries[caret_pos.min(runs.len())];
    let selection = selection.map(|(start, end)| {
        let start = boundaries[start.min(runs.len())];
        let end = boundaries[end.min(runs.len())];
        (start, end)
    });
    render_visual_chars_with_selection(app, ui, &visual, font_size, selection, Some(caret_pos));
}

fn build_tagged_visual(
    block: &Block,
    runs: &[StyledChar],
    table_cell: Option<(usize, usize)>,
) -> (Vec<VisualChar>, Vec<usize>) {
    let mut visual = Vec::new();
    for tag in outer_tags(block, table_cell) {
        push_tag(&mut visual, &format!("<{tag}>"));
    }

    let mut boundaries = vec![visual.len(); runs.len() + 1];
    let mut open_inline: Vec<&'static str> = Vec::new();

    for (idx, run) in runs.iter().enumerate() {
        let next_inline = style_tags(&run.style);
        if idx == 0 {
            open_tags(&mut visual, &next_inline);
            open_inline = next_inline;
        } else if open_inline != next_inline {
            boundaries[idx] = visual.len();
            sync_inline_tags(&mut visual, &mut open_inline, next_inline);
        }
        boundaries[idx] = visual.len();
        visual.push(VisualChar {
            ch: run.ch,
            kind: VisualKind::Text(run.style.clone()),
        });
    }

    boundaries[runs.len()] = visual.len();
    close_tags(&mut visual, &open_inline);

    for tag in outer_tags(block, table_cell).into_iter().rev() {
        push_tag(&mut visual, &format!("</{tag}>"));
    }

    (visual, boundaries)
}

fn outer_tags(block: &Block, table_cell: Option<(usize, usize)>) -> Vec<String> {
    if let Some((row_idx, col_idx)) = table_cell {
        let cell_tag = match block {
            Block::Table(table) => table
                .rows
                .get(row_idx)
                .and_then(|row| row.cells.get(col_idx))
                .map(|cell| if cell.header { "th" } else { "td" })
                .unwrap_or("td"),
            _ => "td",
        };
        return vec!["tr".into(), cell_tag.into()];
    }

    let mut tags = Vec::new();
    if block.is_list_item() {
        tags.push(match block {
            Block::Numbered(_) => "ol".into(),
            _ => "ul".into(),
        });
    }
    tags.push(block.tag());
    tags
}

fn style_tags(style: &InlineStyle) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if style.code {
        tags.push("code");
    }
    if style.bold {
        tags.push("strong");
    }
    if style.italic {
        tags.push("em");
    }
    if style.underline {
        tags.push("u");
    }
    if style.strike {
        tags.push("s");
    }
    if style.link.is_some() {
        tags.push("a");
    }
    tags
}

fn sync_inline_tags(
    visual: &mut Vec<VisualChar>,
    open: &mut Vec<&'static str>,
    next: Vec<&'static str>,
) {
    let common = open
        .iter()
        .zip(next.iter())
        .take_while(|(a, b)| a == b)
        .count();
    close_tags(visual, &open[common..]);
    open_tags(visual, &next[common..]);
    *open = next;
}

fn open_tags(visual: &mut Vec<VisualChar>, tags: &[&str]) {
    for tag in tags {
        push_tag(visual, &format!("<{tag}>"));
    }
}

fn close_tags(visual: &mut Vec<VisualChar>, tags: &[&str]) {
    for tag in tags.iter().rev() {
        push_tag(visual, &format!("</{tag}>"));
    }
}

fn push_tag(visual: &mut Vec<VisualChar>, tag: &str) {
    visual.extend(tag.chars().map(|ch| VisualChar {
        ch,
        kind: VisualKind::Tag,
    }));
}

fn render_visual_chars_with_selection(
    app: &App,
    ui: &mut Ui,
    chars: &[VisualChar],
    font_size: f32,
    selection: Option<(usize, usize)>,
    caret_pos: Option<usize>,
) {
    let Some((start, end)) = selection else {
        if let Some(pos) = caret_pos {
            render_visual_slice(ui, &chars[..pos], font_size);
            render_caret(app, ui, font_size);
            render_visual_slice(ui, &chars[pos..], font_size);
        } else {
            render_visual_slice(ui, chars, font_size);
        }
        return;
    };

    render_visual_slice(ui, &chars[..start], font_size);
    if caret_pos == Some(start) {
        render_caret(app, ui, font_size);
    }
    render_selected_visual_slice(ui, &chars[start..end], font_size);
    if caret_pos == Some(end) {
        render_caret(app, ui, font_size);
    }
    render_visual_slice(ui, &chars[end..], font_size);
}

fn render_visual_slice(ui: &mut Ui, chars: &[VisualChar], font_size: f32) {
    for (text, kind) in group_visual_chars(chars) {
        ui.label(visual_rich_text(ui, text, kind, font_size, false));
    }
}

fn render_selected_visual_slice(ui: &mut Ui, chars: &[VisualChar], font_size: f32) {
    for (text, kind) in group_visual_chars(chars) {
        ui.label(visual_rich_text(ui, text, kind, font_size, true));
    }
}

fn group_visual_chars(chars: &[VisualChar]) -> Vec<(String, VisualKind)> {
    let mut out: Vec<(String, VisualKind)> = Vec::new();
    for visual in chars {
        if let Some((text, kind)) = out.last_mut() {
            if *kind == visual.kind {
                text.push(visual.ch);
                continue;
            }
        }
        out.push((visual.ch.to_string(), visual.kind.clone()));
    }
    out
}

fn visual_rich_text(
    ui: &Ui,
    text: String,
    kind: VisualKind,
    font_size: f32,
    selected: bool,
) -> RichText {
    let mut rich = RichText::new(text).size(font_size);
    match kind {
        VisualKind::Text(style) => {
            if selected {
                rich = rich
                    .background_color(ui.visuals().selection.bg_fill)
                    .color(ui.visuals().selection.stroke.color);
            }
            if style.bold {
                rich = rich.strong();
            }
            if style.italic {
                rich = rich.italics();
            }
            if style.underline {
                rich = rich.underline();
            }
            if style.strike {
                rich = rich.strikethrough();
            }
            if style.code {
                rich = rich.monospace();
                if !selected {
                    rich = rich.background_color(Color32::from_gray(230));
                }
            }
            if style.link.is_some() {
                rich = rich.color(ui.visuals().hyperlink_color).underline();
            }
        }
        VisualKind::Tag => {
            rich = rich
                .color(ui.visuals().weak_text_color())
                .background_color(ui.visuals().faint_bg_color);
        }
    }
    rich
}

pub(super) fn render_runs_with_selection(
    app: &App,
    ui: &mut Ui,
    runs: &[StyledChar],
    font_size: f32,
    selection: Option<(usize, usize)>,
    caret_pos: Option<usize>,
) {
    let Some((start, end)) = selection else {
        if let Some(pos) = caret_pos {
            render_run_slice(ui, &runs[..pos], font_size);
            render_caret(app, ui, font_size);
            render_run_slice(ui, &runs[pos..], font_size);
        } else {
            render_run_slice(ui, runs, font_size);
        }
        return;
    };

    render_run_slice(ui, &runs[..start], font_size);
    if caret_pos == Some(start) {
        render_caret(app, ui, font_size);
    }
    render_selected_run_slice(ui, &runs[start..end], font_size);
    if caret_pos == Some(end) {
        render_caret(app, ui, font_size);
    }
    render_run_slice(ui, &runs[end..], font_size);
}

fn render_caret(app: &App, ui: &mut Ui, font_size: f32) {
    render_colored_caret(ui, font_size, app.settings.accent);
}

fn render_colored_caret(ui: &mut Ui, font_size: f32, accent: [u8; 3]) {
    let [r, g, b] = accent;
    ui.label(
        RichText::new("|")
            .size(font_size)
            .strong()
            .color(Color32::from_rgb(r, g, b)),
    );
}
