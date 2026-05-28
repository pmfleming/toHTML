//! Document model: blocks of styled characters, tables, and editing primitives.

use serde::{Deserialize, Serialize};
mod runs;
mod table;

pub use runs::group_runs;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineStyle {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strike: bool,
    #[serde(default)]
    pub code: bool,
    #[serde(default)]
    pub link: Option<String>,
}

impl InlineStyle {
    pub fn merge_toggle(&mut self, other: &InlineStyle) {
        if other.bold {
            self.bold ^= true;
        }
        if other.italic {
            self.italic ^= true;
        }
        if other.underline {
            self.underline ^= true;
        }
        if other.strike {
            self.strike ^= true;
        }
        if other.code {
            self.code ^= true;
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StyledChar {
    pub ch: char,
    #[serde(default)]
    pub style: InlineStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Block {
    Heading(u8, Vec<StyledChar>),
    Paragraph(Vec<StyledChar>),
    Blockquote(Vec<StyledChar>),
    Bullet(Vec<StyledChar>),
    Numbered(Vec<StyledChar>),
    Pre(Vec<StyledChar>),
    Table(Table),
    Image(Image),
    PageBreak(Option<u32>),
    PagePlaceholder { page: Option<u32>, reason: String },
    RawHtml(String),
    Hr,
}

impl Default for Block {
    fn default() -> Self {
        Block::Paragraph(Vec::new())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Table {
    #[serde(default)]
    pub caption: Option<Vec<StyledChar>>,
    pub rows: Vec<TableRow>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TableCell {
    #[serde(default)]
    pub header: bool,
    #[serde(default)]
    pub colspan: u16,
    #[serde(default)]
    pub rowspan: u16,
    #[serde(default)]
    pub align: Option<String>,
    pub content: Vec<StyledChar>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    #[serde(default)]
    pub alt: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

impl Block {
    pub fn inline(&self) -> Option<&Vec<StyledChar>> {
        match self {
            Block::Heading(_, v)
            | Block::Paragraph(v)
            | Block::Blockquote(v)
            | Block::Bullet(v)
            | Block::Numbered(v)
            | Block::Pre(v) => Some(v),
            Block::Table(_)
            | Block::Image(_)
            | Block::PageBreak(_)
            | Block::PagePlaceholder { .. }
            | Block::RawHtml(_)
            | Block::Hr => None,
        }
    }
    pub fn inline_mut(&mut self) -> Option<&mut Vec<StyledChar>> {
        match self {
            Block::Heading(_, v)
            | Block::Paragraph(v)
            | Block::Blockquote(v)
            | Block::Bullet(v)
            | Block::Numbered(v)
            | Block::Pre(v) => Some(v),
            Block::Table(_)
            | Block::Image(_)
            | Block::PageBreak(_)
            | Block::PagePlaceholder { .. }
            | Block::RawHtml(_)
            | Block::Hr => None,
        }
    }
    pub fn len(&self) -> usize {
        self.inline().map(|v| v.len()).unwrap_or(0)
    }
    pub fn text(&self) -> String {
        match self {
            Block::Table(table) => table
                .rows
                .iter()
                .map(|row| {
                    row.cells
                        .iter()
                        .map(|cell| chars_to_string(&cell.content))
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Block::Image(image) => image.alt.clone(),
            Block::PageBreak(page) => page
                .map(|n| format!("page break {n}"))
                .unwrap_or_else(|| "page break".into()),
            Block::PagePlaceholder { page, reason } => {
                let page = page.map(|n| n.to_string()).unwrap_or_else(|| "?".into());
                format!("page {page}: {reason}")
            }
            Block::RawHtml(html) => html.clone(),
            _ => self
                .inline()
                .map(|v| chars_to_string(v))
                .unwrap_or_default(),
        }
    }
    pub fn tag(&self) -> String {
        match self {
            Block::Heading(l, _) => format!("h{l}"),
            Block::Paragraph(_) => "p".into(),
            Block::Blockquote(_) => "blockquote".into(),
            Block::Bullet(_) => "li".into(),
            Block::Numbered(_) => "li".into(),
            Block::Pre(_) => "pre".into(),
            Block::Table(_) => "table".into(),
            Block::Image(_) => "img".into(),
            Block::PageBreak(_) => "hr".into(),
            Block::PagePlaceholder { .. } => "div".into(),
            Block::RawHtml(_) => "raw".into(),
            Block::Hr => "hr".into(),
        }
    }
    pub fn is_list_item(&self) -> bool {
        matches!(self, Block::Bullet(_) | Block::Numbered(_))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Doc {
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Caret {
    pub block: usize,
    pub char: usize,
    pub table_row: usize,
    pub table_col: usize,
}

impl Doc {
    pub fn empty_paragraph() -> Self {
        Self {
            blocks: vec![Block::Paragraph(Vec::new())],
        }
    }

    pub fn welcome() -> Self {
        let p = |s: &str| Block::Paragraph(plain(s));
        let h = |l, s: &str| Block::Heading(l, plain(s));
        Self {
            blocks: vec![
                h(1, "Welcome"),
                p("A minimal command-palette HTML editor in Rust + egui."),
                p("Press Ctrl+T for the filtering command palette.  Type < (inside the editor) for the HTML-tag palette.  Ctrl+/ shows all shortcuts."),
                h(2, "Try"),
                Block::Bullet(plain("Ctrl+B / Ctrl+I — bold / italic for the next characters you type")),
                Block::Bullet(plain("Ctrl+O / Ctrl+S — open / save an HTML file")),
                Block::Bullet(plain("Use < for HTML tag commands, including table creation")),
                Block::Bullet(plain("Ctrl+Z / Ctrl+Shift+Z — undo / redo")),
                Block::Bullet(plain("Backspace at start of a line — joins with the previous block")),
                p("Inline tag markers show the current HTML element while the text stays WYSIWYG."),
            ],
        }
    }

    pub fn full_text(&self) -> String {
        let mut s = String::new();
        for b in &self.blocks {
            s.push_str(&b.text());
            s.push('\n');
        }
        s
    }

    pub fn clamp_caret(&mut self, caret: &mut Caret) {
        if self.blocks.is_empty() {
            self.blocks.push(Block::Paragraph(Vec::new()));
        }
        if caret.block >= self.blocks.len() {
            caret.block = self.blocks.len() - 1;
        }
        if let Block::Table(table) = &self.blocks[caret.block] {
            let max_row = table.rows.len().saturating_sub(1);
            caret.table_row = caret.table_row.min(max_row);
            let max_col = table
                .rows
                .get(caret.table_row)
                .map(|r| r.cells.len().saturating_sub(1))
                .unwrap_or(0);
            caret.table_col = caret.table_col.min(max_col);
        } else {
            caret.table_row = 0;
            caret.table_col = 0;
        }
        let max = self.current_len(caret);
        if caret.char > max {
            caret.char = max;
        }
    }

    pub fn tag_stack(&self, caret: &Caret) -> String {
        let mut tags: Vec<String> = vec!["body".into()];
        let Some(b) = self.blocks.get(caret.block) else {
            return tags.join("  ›  ");
        };
        // List wrapper
        if b.is_list_item() {
            tags.push(match b {
                Block::Numbered(_) => "ol".into(),
                _ => "ul".into(),
            });
        }
        tags.push(b.tag());
        if let Block::Table(table) = b {
            tags.push("tr".into());
            let cell_tag = table
                .rows
                .get(caret.table_row)
                .and_then(|row| row.cells.get(caret.table_col))
                .map(|cell| if cell.header { "th" } else { "td" })
                .unwrap_or("td");
            tags.push(cell_tag.into());
        }
        // Inline style around caret (look at char before caret if not at zero)
        if let Some(runs) = self.current_inline(caret) {
            if !runs.is_empty() {
                let idx = caret.char.saturating_sub(1).min(runs.len() - 1);
                let s = &runs[idx].style;
                if s.code {
                    tags.push("code".into());
                }
                if s.bold {
                    tags.push("strong".into());
                }
                if s.italic {
                    tags.push("em".into());
                }
                if s.underline {
                    tags.push("u".into());
                }
                if s.strike {
                    tags.push("s".into());
                }
                if s.link.is_some() {
                    tags.push("a".into());
                }
            }
        }
        tags.join("  ›  ")
    }

    pub fn insert_text(&mut self, caret: &mut Caret, text: &str, style: &InlineStyle) {
        if self.blocks.is_empty() {
            self.blocks.push(Block::Paragraph(Vec::new()));
            caret.block = 0;
            caret.char = 0;
        }
        if let Some(runs) = self.current_inline_mut(caret) {
            for c in text.chars() {
                runs.insert(
                    caret.char,
                    StyledChar {
                        ch: c,
                        style: style.clone(),
                    },
                );
                caret.char += 1;
            }
        }
    }

    pub fn backspace(&mut self, caret: &mut Caret) {
        if caret.block >= self.blocks.len() {
            return;
        }
        if caret.char == 0 {
            if matches!(self.blocks[caret.block], Block::Table(_)) {
                return;
            }
            // Merge with previous block if any
            if caret.block == 0 {
                return;
            }
            let cur = self.blocks.remove(caret.block);
            caret.block -= 1;
            let new_char = self.blocks[caret.block].len();
            let cur_inline = match cur {
                Block::Hr => None,
                _ => Some(cur.inline().cloned().unwrap_or_default()),
            };
            if let (Some(items), Some(prev)) = (cur_inline, self.blocks[caret.block].inline_mut()) {
                prev.extend(items);
            }
            caret.char = new_char;
            return;
        }
        if let Some(runs) = self.current_inline_mut(caret) {
            if caret.char > 0 && caret.char <= runs.len() {
                runs.remove(caret.char - 1);
                caret.char -= 1;
            }
        }
    }

    pub fn delete_forward(&mut self, caret: &mut Caret) {
        let len = self.current_len(caret);
        if caret.char < len {
            if let Some(runs) = self.current_inline_mut(caret) {
                runs.remove(caret.char);
            }
        } else if caret.block + 1 < self.blocks.len() {
            let next = self.blocks.remove(caret.block + 1);
            let next_inline = match next {
                Block::Hr => None,
                _ => Some(next.inline().cloned().unwrap_or_default()),
            };
            if let (Some(items), Some(cur)) = (next_inline, self.blocks[caret.block].inline_mut()) {
                cur.extend(items);
            }
        }
    }

    pub fn split_block(&mut self, caret: &mut Caret) {
        if self.blocks.is_empty() {
            self.blocks.push(Block::Paragraph(Vec::new()));
            caret.block = 0;
            caret.char = 0;
            return;
        }
        let cur_kind = self.blocks[caret.block].clone();
        if matches!(cur_kind, Block::Table(_)) {
            let style = self.current_style_at(caret);
            self.insert_text(caret, "\n", &style);
            return;
        }
        // Empty list item → exit list (turn into paragraph after the list-style block)
        if cur_kind.is_list_item() && cur_kind.len() == 0 {
            self.blocks[caret.block] = Block::Paragraph(Vec::new());
            caret.char = 0;
            return;
        }
        // Empty blockquote → escape
        if matches!(cur_kind, Block::Blockquote(_)) && cur_kind.len() == 0 {
            self.blocks[caret.block] = Block::Paragraph(Vec::new());
            caret.char = 0;
            return;
        }
        let tail: Vec<StyledChar> = {
            let Some(runs) = self.blocks[caret.block].inline_mut() else {
                return;
            };
            runs.split_off(caret.char.min(runs.len()))
        };
        let new_block = match &cur_kind {
            Block::Heading(_, _) => Block::Paragraph(tail), // Enter in heading → paragraph
            Block::Bullet(_) => Block::Bullet(tail),
            Block::Numbered(_) => Block::Numbered(tail),
            Block::Blockquote(_) => Block::Blockquote(tail),
            Block::Pre(_) => Block::Pre(tail),
            _ => Block::Paragraph(tail),
        };
        self.blocks.insert(caret.block + 1, new_block);
        caret.block += 1;
        caret.char = 0;
    }

    pub fn move_left(&self, caret: &mut Caret) {
        if caret.char > 0 {
            caret.char -= 1;
            return;
        }
        if caret.block > 0 {
            caret.block -= 1;
            caret.char = self.blocks[caret.block].len();
        }
    }
    pub fn move_right(&self, caret: &mut Caret) {
        let len = self.current_len(caret);
        if caret.char < len {
            caret.char += 1;
            return;
        }
        if caret.block + 1 < self.blocks.len() {
            caret.block += 1;
            caret.char = 0;
            caret.table_row = 0;
            caret.table_col = 0;
        }
    }

    pub fn transform_block_to<F: FnOnce(Vec<StyledChar>) -> Block>(&mut self, idx: usize, mk: F) {
        let Some(b) = self.blocks.get_mut(idx) else {
            return;
        };
        let runs = if let Some(v) = b.inline_mut() {
            std::mem::take(v)
        } else {
            return;
        };
        *b = mk(runs);
    }

    pub fn apply_style_range(&mut self, lo: &Caret, hi: &Caret, toggle: impl Fn(&mut InlineStyle)) {
        for bi in lo.block..=hi.block {
            let Some(b) = self.blocks.get_mut(bi) else {
                continue;
            };
            let Some(runs) = b.inline_mut() else {
                continue;
            };
            let len = runs.len();
            let start = if bi == lo.block { lo.char.min(len) } else { 0 };
            let end = if bi == hi.block {
                hi.char.min(len)
            } else {
                len
            };
            for i in start..end {
                if let Some(c) = runs.get_mut(i) {
                    toggle(&mut c.style);
                }
            }
        }
    }

    pub fn current_style_at(&self, caret: &Caret) -> InlineStyle {
        let Some(runs) = self.current_inline(caret) else {
            return InlineStyle::default();
        };
        if runs.is_empty() {
            return InlineStyle::default();
        }
        let idx = caret.char.saturating_sub(1).min(runs.len() - 1);
        runs[idx].style.clone()
    }

    pub fn current_len(&self, caret: &Caret) -> usize {
        self.current_inline(caret)
            .map(|runs| runs.len())
            .unwrap_or(0)
    }

    pub fn current_inline(&self, caret: &Caret) -> Option<&Vec<StyledChar>> {
        match self.blocks.get(caret.block)? {
            Block::Table(table) => table
                .rows
                .get(caret.table_row)?
                .cells
                .get(caret.table_col)
                .map(|cell| &cell.content),
            block => block.inline(),
        }
    }

    pub fn current_inline_mut(&mut self, caret: &Caret) -> Option<&mut Vec<StyledChar>> {
        match self.blocks.get_mut(caret.block)? {
            Block::Table(table) => table
                .rows
                .get_mut(caret.table_row)?
                .cells
                .get_mut(caret.table_col)
                .map(|cell| &mut cell.content),
            block => block.inline_mut(),
        }
    }

    pub fn move_to_block_end(&mut self, caret: &mut Caret, block: usize) {
        caret.block = block.min(self.blocks.len().saturating_sub(1));
        caret.table_row = 0;
        caret.table_col = 0;
        self.clamp_caret(caret);
        caret.char = self.current_len(caret);
    }

    pub fn move_to_table_cell(&mut self, caret: &mut Caret, block: usize, row: usize, col: usize) {
        caret.block = block.min(self.blocks.len().saturating_sub(1));
        caret.table_row = row;
        caret.table_col = col;
        self.clamp_caret(caret);
        caret.char = self.current_len(caret);
    }
}

fn plain(s: &str) -> Vec<StyledChar> {
    s.chars()
        .map(|c| StyledChar {
            ch: c,
            style: InlineStyle::default(),
        })
        .collect()
}

#[cfg(test)]
pub fn plain_chars(s: &str) -> Vec<StyledChar> {
    plain(s)
}

pub fn chars_to_string(runs: &[StyledChar]) -> String {
    runs.iter().map(|c| c.ch).collect()
}
