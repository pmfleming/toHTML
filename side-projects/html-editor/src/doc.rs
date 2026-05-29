//! Document model: blocks of styled characters, tables, and editing primitives.

use serde::{Deserialize, Serialize};
mod editing;
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
    PdfPage(PdfPage),
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfPage {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub width_pt: Option<f32>,
    #[serde(default)]
    pub height_pt: Option<f32>,
    #[serde(default)]
    pub elements: Vec<PdfElement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PdfElement {
    Text(PdfTextFragment),
    Image(PdfImageElement),
    Shape(PdfShape),
    Ink(PdfInk),
    Link(PdfLinkOverlay),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfBox {
    #[serde(default)]
    pub left_pt: Option<f32>,
    #[serde(default)]
    pub top_pt: Option<f32>,
    #[serde(default)]
    pub width_pt: Option<f32>,
    #[serde(default)]
    pub height_pt: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfTextFragment {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub bounds: PdfBox,
    #[serde(default)]
    pub font_size_pt: Option<f32>,
    #[serde(default)]
    pub font_weight: Option<u16>,
    #[serde(default)]
    pub font_family: Option<String>,
    #[serde(default)]
    pub font_style: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub transform: Option<String>,
    #[serde(default)]
    pub rotation_deg: Option<f32>,
    #[serde(default)]
    pub scale_x: Option<f32>,
    #[serde(default)]
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfImageElement {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub bounds: PdfBox,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub alt: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfShape {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub bounds: PdfBox,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub border_width_pt: Option<f32>,
    #[serde(default)]
    pub border_color: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfInk {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub bounds: PdfBox,
    #[serde(default)]
    pub view_box: Option<String>,
    #[serde(default)]
    pub paths: Vec<PdfPath>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfPath {
    #[serde(default)]
    pub d: String,
    #[serde(default)]
    pub fill: Option<String>,
    #[serde(default)]
    pub stroke: Option<String>,
    #[serde(default)]
    pub stroke_width: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PdfLinkOverlay {
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub bounds: PdfBox,
    #[serde(default)]
    pub href: String,
    #[serde(default)]
    pub label: Option<String>,
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
            | Block::PdfPage(_)
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
            | Block::PdfPage(_)
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
            Block::PdfPage(page) => {
                let page = page
                    .page
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".into());
                format!("PDF page {page}")
            }
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
            Block::PdfPage(_) => "section".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(text: &str) -> Block {
        Block::Paragraph(plain(text))
    }

    fn table(rows: Vec<Vec<&str>>) -> Block {
        Block::Table(Table {
            caption: None,
            rows: rows
                .into_iter()
                .enumerate()
                .map(|(row_idx, cells)| TableRow {
                    cells: cells
                        .into_iter()
                        .map(|text| TableCell {
                            header: row_idx == 0,
                            colspan: 1,
                            rowspan: 1,
                            align: None,
                            content: plain(text),
                        })
                        .collect(),
                })
                .collect(),
        })
    }

    #[test]
    fn backspace_before_paragraph_does_not_drop_it_after_table() {
        let mut doc = Doc {
            blocks: vec![table(vec![vec!["cell"]]), paragraph("after")],
        };
        let mut caret = Caret {
            block: 1,
            char: 0,
            table_row: 0,
            table_col: 0,
        };

        assert!(!doc.backspace(&mut caret));
        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(doc.blocks[1].text(), "after");
    }

    #[test]
    fn delete_forward_before_table_does_not_drop_table() {
        let mut doc = Doc {
            blocks: vec![paragraph("before"), table(vec![vec!["cell"]])],
        };
        let mut caret = Caret {
            block: 0,
            char: "before".chars().count(),
            table_row: 0,
            table_col: 0,
        };

        assert!(!doc.delete_forward(&mut caret));
        assert_eq!(doc.blocks.len(), 2);
        assert!(matches!(doc.blocks[1], Block::Table(_)));
    }

    #[test]
    fn delete_forward_still_merges_adjacent_inline_blocks() {
        let mut doc = Doc {
            blocks: vec![paragraph("one"), paragraph("two")],
        };
        let mut caret = Caret {
            block: 0,
            char: 3,
            table_row: 0,
            table_col: 0,
        };

        assert!(doc.delete_forward(&mut caret));
        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(doc.blocks[0].text(), "onetwo");
    }

    #[test]
    fn applies_style_ranges_inside_table_cells() {
        let mut doc = Doc {
            blocks: vec![table(vec![vec!["abc"]])],
        };
        let start = Caret {
            block: 0,
            char: 0,
            table_row: 0,
            table_col: 0,
        };
        let end = Caret {
            char: 2,
            ..start.clone()
        };

        assert!(doc.apply_style_range(&start, &end, |style| style.bold = true));
        let Some(runs) = doc.current_inline(&start) else {
            panic!("table cell content should be editable");
        };
        assert!(runs[0].style.bold);
        assert!(runs[1].style.bold);
        assert!(!runs[2].style.bold);
    }

    #[test]
    fn arrow_navigation_crosses_table_cells() {
        let doc = Doc {
            blocks: vec![table(vec![vec!["a", "bb"], vec!["ccc", "d"]])],
        };
        let mut caret = Caret {
            block: 0,
            char: 1,
            table_row: 0,
            table_col: 0,
        };

        assert!(doc.move_right(&mut caret));
        assert_eq!((caret.table_row, caret.table_col, caret.char), (0, 1, 0));

        assert!(doc.move_left(&mut caret));
        assert_eq!((caret.table_row, caret.table_col, caret.char), (0, 0, 1));

        assert!(doc.move_table_row(&mut caret, 1));
        assert_eq!((caret.table_row, caret.table_col, caret.char), (1, 0, 1));
    }
}
