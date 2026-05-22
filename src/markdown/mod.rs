mod blocks;
mod inlines;
mod tables;

use crate::{Document, SourceFormat};

pub fn markdown_to_document(input: &str) -> Document {
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Markdown);
    document.blocks = blocks::parse_blocks(input);
    document.metadata.title = first_heading_title(&document.blocks);
    document
}

fn first_heading_title(blocks: &[crate::Block]) -> Option<String> {
    blocks.iter().find_map(|block| match block {
        crate::Block::Heading(heading) if heading.level == 1 => Some(plain_text(&heading.content)),
        _ => None,
    })
}

fn plain_text(inlines: &[crate::Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            crate::Inline::Text(value) | crate::Inline::Code(value) => text.push_str(value),
            crate::Inline::Emphasis(children)
            | crate::Inline::Strong(children)
            | crate::Inline::Strikethrough(children) => text.push_str(&plain_text(children)),
            crate::Inline::Link(link) => text.push_str(&plain_text(&link.content)),
            crate::Inline::Image(image) => {
                if let Some(alt) = &image.alt {
                    text.push_str(alt);
                }
            }
            crate::Inline::LineBreak => text.push(' '),
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{render_html, Block, Inline, TableAlignment};

    #[test]
    fn parses_headings_paragraphs_and_inline_markup() {
        let document =
            markdown_to_document("# Title\n\nHello **strong** and [link](https://example.test).");

        assert_eq!(document.metadata.title.as_deref(), Some("Title"));
        assert_eq!(document.blocks.len(), 2);
        assert!(matches!(document.blocks[0], Block::Heading(_)));
        assert!(matches!(document.blocks[1], Block::Paragraph(_)));

        let html = render_html(&document);
        assert!(html.contains("<strong>strong</strong>"));
        assert!(html.contains("<a href=\"https://example.test\">link</a>"));
    }

    #[test]
    fn parses_task_lists() {
        let document = markdown_to_document("- [x] Done\n- [ ] Todo");

        let Block::List(list) = &document.blocks[0] else {
            panic!("expected list");
        };

        assert_eq!(list.items[0].checked, Some(true));
        assert_eq!(list.items[1].checked, Some(false));
    }

    #[test]
    fn parses_gfm_table_alignment() {
        let document = markdown_to_document("| Name | Count |\n| :--- | ---: |\n| A | 3 |");

        let Block::Table(table) = &document.blocks[0] else {
            panic!("expected table");
        };

        assert!(table.rows[0].cells[0].header);
        assert_eq!(table.rows[1].cells[1].content, vec![Inline::text("3")]);
        assert_eq!(table.rows[1].cells[0].align, Some(TableAlignment::Left));
        assert_eq!(table.rows[1].cells[1].align, Some(TableAlignment::Right));
    }
}
