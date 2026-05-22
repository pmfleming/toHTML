use crate::{Block, Inline, TableAlignment};

use super::layout::blocks_from_segments;
use super::text::TextSegment;

#[test]
fn converts_aligned_lines_to_table() {
    let segments = vec![
        segment("Model", 10.0, 100.0),
        segment("Current", 120.0, 100.0),
        segment("EUD-150", 10.0, 84.0),
        segment("700mA", 120.0, 84.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows.len(), 2);
    assert!(table.rows[0].cells[0].header);
    assert_eq!(
        table.rows[1].cells[1].content,
        vec![Inline::Text("700mA".to_string())]
    );
}

#[test]
fn clusters_nearby_words_inside_table_cells() {
    let segments = vec![
        segment("Index", 10.0, 100.0),
        segment("Or", 45.0, 100.0),
        segment("MessageItem", 120.0, 100.0),
        segment("1.1", 10.0, 84.0),
        segment("MessageIdentification", 120.0, 84.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(
        table.rows[0].cells[0].content,
        vec![Inline::Text("Index Or".to_string())]
    );
    assert_eq!(table.rows[0].cells.len(), 2);
}

#[test]
fn keeps_wrapped_table_cell_text_inside_the_table() {
    let segments = vec![
        segment("Definition:", 10.0, 100.0),
        segment("Unique identification, as assigned by", 120.0, 100.0),
        segment("a sending party.", 120.0, 86.0),
        segment("Occurrence:", 10.0, 70.0),
        segment("[1..1]", 120.0, 70.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows.len(), 2);
    assert_eq!(
        table.rows[0].cells[1].content,
        vec![Inline::Text(
            "Unique identification, as assigned by a sending party.".to_string()
        )]
    );
}

#[test]
fn uses_message_item_header_text_for_following_rows() {
    let segments = vec![
        segment(
            "Index Or MessageItem <XMLTag> Mult. Represent./Type",
            10.0,
            100.0,
        ),
        segment("2.1", 10.0, 84.0),
        segment("PaymentInformationIdentification", 120.0, 84.0),
        segment("<PmtInfId>", 340.0, 84.0),
        segment("[1..1]", 430.0, 84.0),
        segment("Text", 520.0, 84.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(
        table.rows[0].cells[0].content,
        vec![Inline::Text("Index Or".to_string())]
    );
    assert_eq!(
        table.rows[0].cells[4].content,
        vec![Inline::Text("Represent./Type".to_string())]
    );
    assert_eq!(
        table.rows[1].cells[1].content,
        vec![Inline::Text("PaymentInformationIdentification".to_string())]
    );
}

#[test]
fn appends_lowercase_wrapped_text_to_last_table_cell() {
    let segments = vec![
        segment("Definition:", 10.0, 100.0),
        segment("Unique identification", 120.0, 100.0),
        segment("within the message.", 10.0, 86.0),
        segment("Data Type:", 10.0, 70.0),
        segment("Max35Text", 120.0, 70.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(
        table.rows[0].cells[1].content,
        vec![Inline::Text(
            "Unique identification within the message.".to_string()
        )]
    );
}

#[test]
fn marks_numeric_table_cells_as_right_aligned() {
    let segments = vec![
        segment("Name", 10.0, 100.0),
        segment("Count", 120.0, 100.0),
        segment("A", 10.0, 84.0),
        segment("300", 120.0, 84.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows[1].cells[1].align, Some(TableAlignment::Right));
}

#[test]
fn keeps_single_line_as_paragraph() {
    let segments = vec![
        segment("Default", 10.0, 100.0),
        segment("Current", 70.0, 100.0),
    ];

    assert!(matches!(
        blocks_from_segments(&segments)[0],
        Block::Paragraph(_)
    ));
}

#[test]
fn merges_nearby_lines_into_paragraph() {
    let segments = vec![
        segment("This is the first line", 10.0, 100.0),
        segment("and this continues it.", 10.0, 86.0),
    ];

    let paragraph = first_paragraph(blocks_from_segments(&segments));

    assert_eq!(
        paragraph.content,
        vec![Inline::Text(
            "This is the first line and this continues it.".to_string()
        )]
    );
}

#[test]
fn infers_unordered_lists_from_markers() {
    let segments = vec![
        segment("- Alpha", 10.0, 100.0),
        segment("- Beta", 10.0, 86.0),
    ];

    let blocks = blocks_from_segments(&segments);

    let Block::List(list) = &blocks[0] else {
        panic!("expected list");
    };
    assert!(!list.ordered);
    assert_eq!(list.items.len(), 2);
}

#[test]
fn infers_large_text_heading() {
    let segments = vec![
        TextSegment::new("Overview".to_string(), 10.0, 120.0, 18.0, 70.0),
        segment("Body text continues here.", 10.0, 96.0),
    ];

    let blocks = blocks_from_segments(&segments);

    assert!(matches!(blocks[0], Block::Heading(_)));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
}

#[test]
fn uses_tagged_heading_roles() {
    let segments = vec![
        TextSegment::new("Tagged".to_string(), 10.0, 120.0, 12.0, 42.0)
            .with_role(Some("H1".to_string())),
    ];

    let Block::Heading(heading) = &blocks_from_segments(&segments)[0] else {
        panic!("expected heading");
    };
    assert_eq!(heading.level, 1);
}

#[test]
fn deduplicates_overlapping_text_segments() {
    let segments = vec![segment("Body", 10.0, 100.0), segment("Body", 10.5, 100.2)];

    let paragraph = first_paragraph(blocks_from_segments(&segments));

    assert_eq!(paragraph.content, vec![Inline::Text("Body".to_string())]);
}

#[test]
fn maps_tagged_inline_emphasis() {
    let segments = vec![
        TextSegment::new("Important".to_string(), 10.0, 120.0, 12.0, 54.0)
            .with_role(Some("Strong".to_string())),
    ];

    let paragraph = first_paragraph(blocks_from_segments(&segments));

    assert!(matches!(paragraph.content[0], Inline::Strong(_)));
}

fn first_table(blocks: Vec<Block>) -> crate::Table {
    let Block::Table(table) = blocks.into_iter().next().unwrap() else {
        panic!("expected table");
    };
    table
}

fn first_paragraph(blocks: Vec<Block>) -> crate::Paragraph {
    let Block::Paragraph(paragraph) = blocks.into_iter().next().unwrap() else {
        panic!("expected paragraph");
    };
    paragraph
}

fn segment(text: &str, x: f32, y: f32) -> TextSegment {
    TextSegment {
        text: text.to_string(),
        x,
        y,
        font_size: 12.0,
        width: text.len() as f32 * 6.0,
        role: None,
    }
}
