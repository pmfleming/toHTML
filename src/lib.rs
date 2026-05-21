#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph(String),
    List { ordered: bool, items: Vec<String> },
    Table(Table),
    Image { src: String, alt: Option<String> },
    PageBreak,
    RawHtml(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    pub text: String,
    pub header: bool,
}

pub fn render_html(document: &Document) -> String {
    let mut html = String::from("<article>\n");

    if let Some(title) = &document.title {
        html.push_str("  <header>\n");
        html.push_str("    <h1>");
        push_escaped(&mut html, title);
        html.push_str("</h1>\n");
        html.push_str("  </header>\n");
    }

    for block in &document.blocks {
        render_block(&mut html, block);
    }

    html.push_str("</article>\n");
    html
}

fn render_block(html: &mut String, block: &Block) {
    match block {
        Block::Heading { level, text } => {
            let level = (*level).clamp(1, 6);
            html.push_str("  <h");
            html.push_str(&level.to_string());
            html.push('>');
            push_escaped(html, text);
            html.push_str("</h");
            html.push_str(&level.to_string());
            html.push_str(">\n");
        }
        Block::Paragraph(text) => {
            html.push_str("  <p>");
            push_escaped(html, text);
            html.push_str("</p>\n");
        }
        Block::List { ordered, items } => {
            let tag = if *ordered { "ol" } else { "ul" };
            html.push_str("  <");
            html.push_str(tag);
            html.push_str(">\n");
            for item in items {
                html.push_str("    <li>");
                push_escaped(html, item);
                html.push_str("</li>\n");
            }
            html.push_str("  </");
            html.push_str(tag);
            html.push_str(">\n");
        }
        Block::Table(table) => render_table(html, table),
        Block::Image { src, alt } => {
            html.push_str("  <img src=\"");
            push_attr_escaped(html, src);
            html.push_str("\" alt=\"");
            if let Some(alt) = alt {
                push_attr_escaped(html, alt);
            }
            html.push_str("\">\n");
        }
        Block::PageBreak => html.push_str("  <hr data-page-break>\n"),
        Block::RawHtml(raw) => {
            html.push_str(raw);
            if !raw.ends_with('\n') {
                html.push('\n');
            }
        }
    }
}

fn render_table(html: &mut String, table: &Table) {
    html.push_str("  <table>\n");
    for row in &table.rows {
        html.push_str("    <tr>");
        for cell in &row.cells {
            let tag = if cell.header { "th" } else { "td" };
            html.push('<');
            html.push_str(tag);
            html.push('>');
            push_escaped(html, &cell.text);
            html.push_str("</");
            html.push_str(tag);
            html.push('>');
        }
        html.push_str("</tr>\n");
    }
    html.push_str("  </table>\n");
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn push_attr_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_document() {
        let document = Document {
            title: Some("Example".to_string()),
            blocks: vec![Block::Paragraph("Hello <world>".to_string())],
        };

        let html = render_html(&document);

        assert!(html.contains("<h1>Example</h1>"));
        assert!(html.contains("<p>Hello &lt;world&gt;</p>"));
    }
}

