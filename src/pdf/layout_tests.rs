use crate::{Block, Inline, TableAlignment};

use super::layout::{add_content_images_to_blocks, blocks_from_segments};
use super::text::TextSegment;
use super::visual::VisualImage;

#[test]
fn converts_aligned_lines_to_table() {
    let segments = segments(&[
        ("Model", 10.0, 100.0),
        ("Current", 120.0, 100.0),
        ("EUD-150", 10.0, 84.0),
        ("700mA", 120.0, 84.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows.len(), 2);
    assert!(table.rows[0].cells[0].header);
    assert_cell_text(&table, 1, 1, "700mA");
}

#[test]
fn clusters_nearby_words_inside_table_cells() {
    let segments = segments(&[
        ("Index", 10.0, 100.0),
        ("Or", 45.0, 100.0),
        ("MessageItem", 120.0, 100.0),
        ("1.1", 10.0, 84.0),
        ("MessageIdentification", 120.0, 84.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert_cell_text(&table, 0, 0, "Index Or");
    assert_eq!(table.rows[0].cells.len(), 2);
}

#[test]
fn keeps_wrapped_table_cell_text_inside_the_table() {
    let segments = segments(&[
        ("Definition:", 10.0, 100.0),
        ("Unique identification, as assigned by", 120.0, 100.0),
        ("a sending party.", 120.0, 86.0),
        ("Occurrence:", 10.0, 70.0),
        ("[1..1]", 120.0, 70.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows.len(), 2);
    assert_cell_text(
        &table,
        0,
        1,
        "Unique identification, as assigned by a sending party.",
    );
}

#[test]
fn appends_lowercase_wrapped_text_to_last_table_cell() {
    let segments = segments(&[
        ("Definition:", 10.0, 100.0),
        ("Unique identification", 120.0, 100.0),
        ("within the message.", 10.0, 86.0),
        ("Data Type:", 10.0, 70.0),
        ("Max35Text", 120.0, 70.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert_cell_text(&table, 0, 1, "Unique identification within the message.");
}

#[test]
fn marks_numeric_table_cells_as_right_aligned() {
    let segments = segments(&[
        ("Name", 10.0, 100.0),
        ("Count", 120.0, 100.0),
        ("A", 10.0, 84.0),
        ("300", 120.0, 84.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows[1].cells[1].align, Some(TableAlignment::Right));
}

#[test]
fn keeps_single_line_as_paragraph() {
    let segments = segments(&[("Default", 10.0, 100.0), ("Current", 70.0, 100.0)]);

    assert!(matches!(
        blocks_from_segments(&segments)[0],
        Block::Paragraph(_)
    ));
}

#[test]
fn merges_nearby_lines_into_paragraph() {
    let segments = segments(&[
        ("This is the first line", 10.0, 100.0),
        ("and this continues it.", 10.0, 86.0),
    ]);

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
    let segments = segments(&[("- Alpha", 10.0, 100.0), ("- Beta", 10.0, 86.0)]);

    let blocks = blocks_from_segments(&segments);

    let Block::List(list) = &blocks[0] else {
        panic!("expected list");
    };
    assert!(!list.ordered);
    assert_eq!(list.items.len(), 2);
}

#[test]
fn keeps_wide_gap_o_bullet_prose_out_of_tables() {
    let mut segments = segments(&[
        ("o", 108.0, 100.0),
        ("Read", 126.0, 100.0),
        ("igital", 166.0, 100.0),
        ("D", 194.0, 100.0),
        ("imming brightness level", 203.0, 100.0),
        ("D", 261.0, 100.0),
        ("I returns value between", 329.0, 100.0),
        ("0", 457.0, 100.0),
        ("-", 464.0, 100.0),
        ("200", 468.0, 100.0),
        ("o Value = dim percentage * 200", 108.0, 84.0),
    ]);
    for segment in &mut segments {
        segment.font_size = 12.0;
    }

    let blocks = blocks_from_segments(&segments);

    assert!(
        blocks.iter().all(|block| !matches!(block, Block::Table(_))),
        "{blocks:?}"
    );
    let Block::List(list) = &blocks[0] else {
        panic!("expected o-bullet list, got {blocks:?}");
    };
    assert_eq!(list.items.len(), 2);
    let Block::Paragraph(paragraph) = &list.items[0].blocks[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(
        paragraph.content,
        vec![Inline::Text(
            "Read Digital Dimming brightness level, returns value between 0-200".to_string()
        )]
    );
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
fn detects_three_column_table_with_wrapping_middle_cell_and_short_header() {
    // Header: short three-word "Index Description Type" line.
    // Row 1: 2.21 | long description that wraps | CHAN
    // Row 2 continues the description.
    // Row 3: another 2.21 | description | CHAN.
    let segments = segments(&[
        ("Index", 72.0, 200.0),
        ("Description", 200.0, 200.0),
        ("Type", 500.0, 200.0),
        ("2.21", 72.0, 184.0),
        ("Either BIC or other must be used. When is", 200.0, 184.0),
        ("CHAN", 500.0, 184.0),
        ("used, only NOTPROVIDED is allowed.", 200.0, 168.0),
        ("2.21", 72.0, 150.0),
        ("Advise EPC usage rules.", 200.0, 150.0),
        ("CHAN", 500.0, 150.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    // 1 header + 2 data rows; the wrap line extends row 1's middle cell.
    assert_eq!(table.rows.len(), 3, "expected 1 header + 2 data rows");
    assert!(table.rows[0].cells[0].header);
    assert_cell_text(&table, 0, 0, "Index");
    assert_cell_text(&table, 1, 2, "CHAN");
    let row1_description = match &table.rows[1].cells[1].content[0] {
        Inline::Text(text) => text.clone(),
        _ => panic!("expected text"),
    };
    assert!(
        row1_description.contains("Either BIC"),
        "row 1 description was {row1_description}"
    );
    assert!(
        row1_description.contains("NOTPROVIDED"),
        "wrap line was not absorbed: row 1 description was {row1_description}"
    );
    assert_cell_text(&table, 2, 2, "CHAN");
}

#[test]
fn detects_three_column_table_even_with_stray_footnote_line_before_header() {
    // Same as the previous case but with a stray "1" footnote line above the header,
    // matching documents that put a footnote marker directly before a table.
    let segments = segments(&[
        ("1", 72.0, 220.0),
        ("Index", 72.0, 200.0),
        ("Description", 200.0, 200.0),
        ("Type", 500.0, 200.0),
        ("2.21", 72.0, 184.0),
        ("Either BIC must be used. When is", 200.0, 184.0),
        ("CHAN", 500.0, 184.0),
        ("used, only NOTPROVIDED is allowed.", 200.0, 168.0),
        ("2.21", 72.0, 150.0),
        ("Advise EPC usage rules.", 200.0, 150.0),
        ("CHAN", 500.0, 150.0),
    ]);

    let blocks = blocks_from_segments(&segments);

    assert!(
        blocks.iter().any(|block| matches!(block, Block::Table(_))),
        "expected a table somewhere in the blocks, got: {blocks:?}"
    );
}

#[test]
fn detects_three_column_table_even_when_description_is_split_into_word_segments() {
    // Each word in the description is its own PDF segment. The columns should
    // still be discovered from segment positions that recur across rows
    // (col0=72, col1=200, col2=500), not from incidental mid-word positions.
    let segments = segments(&[
        ("Index", 72.0, 200.0),
        ("Description", 200.0, 200.0),
        ("Type", 500.0, 200.0),
        ("2.21", 72.0, 184.0),
        ("Either", 200.0, 184.0),
        ("BIC", 260.0, 184.0),
        ("must", 287.0, 184.0),
        ("be", 317.0, 184.0),
        ("used", 337.0, 184.0),
        ("CHAN", 500.0, 184.0),
        ("2.21", 72.0, 168.0),
        ("Advise", 200.0, 168.0),
        ("EPC", 268.0, 168.0),
        ("rules", 296.0, 168.0),
        ("CHAN", 500.0, 168.0),
    ]);

    let blocks = blocks_from_segments(&segments);

    assert!(
        blocks.iter().any(|block| matches!(block, Block::Table(_))),
        "expected a table, got: {blocks:?}"
    );
    let table = first_table(blocks);
    assert_eq!(table.rows[0].cells.len(), 3, "got rows: {:?}", table.rows);
    assert!(table.rows[0].cells[0].header);
    assert_cell_text(&table, 1, 2, "CHAN");
}

#[test]
fn keeps_clustered_formula_fragments_in_one_table_cell() {
    let segments = vec![
        segment("Welding process", 76.0, 200.0),
        segment("Load voltage", 426.0, 200.0),
        segment("Manual arc welding", 76.0, 184.0),
        segment("with coated electrodes", 145.0, 184.0),
        small_segment("U", 426.0, 184.0, 6.0),
        small_segment("2", 432.0, 182.0, 4.0),
        small_segment("= (18", 439.0, 184.0, 21.0),
        small_segment("+", 462.0, 184.0, 4.0),
        small_segment("0,04", 469.0, 184.0, 18.0),
        small_segment("I", 489.0, 184.0, 4.0),
        small_segment("2", 492.0, 182.0, 4.0),
        segment("Tungsten electrode welding", 76.0, 168.0),
        small_segment("U", 426.0, 168.0, 6.0),
        small_segment("2", 432.0, 166.0, 4.0),
        small_segment("= (10", 439.0, 168.0, 21.0),
        small_segment("+", 462.0, 168.0, 4.0),
        small_segment("0,04", 469.0, 168.0, 18.0),
        small_segment("I", 489.0, 168.0, 4.0),
        small_segment("2", 492.0, 166.0, 4.0),
        TextSegment::new("Next section heading".to_string(), 70.0, 126.0, 13.0, 120.0),
    ];

    let table = first_table(blocks_from_segments(&segments));

    assert_eq!(table.rows.len(), 3, "got rows: {:?}", table.rows);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_cell_text(&table, 1, 0, "Manual arc welding with coated electrodes");
    assert_cell_text(&table, 1, 1, "U 2 = (18 + 0,04 I 2)");
}

#[test]
fn promotes_preceding_short_line_to_table_header_when_columns_align() {
    let segments = segments(&[
        ("Version", 72.0, 200.0),
        ("Date", 200.0, 200.0),
        ("2.0", 72.0, 184.0),
        ("October 2010", 200.0, 184.0),
        ("2.1", 72.0, 168.0),
        ("February 2011", 200.0, 168.0),
    ]);

    let table = first_table(blocks_from_segments(&segments));

    assert!(table.rows[0].cells[0].header);
    assert_cell_text(&table, 0, 0, "Version");
    assert_cell_text(&table, 0, 1, "Date");
}

#[test]
fn does_not_classify_short_body_text_lines_as_headings() {
    // Three lines at the same font size, vertically close. None should be headings.
    let segments = segments(&[
        ("Body text first line.", 10.0, 100.0),
        ("Body text second line.", 10.0, 88.0),
        ("Body text third line.", 10.0, 76.0),
    ]);

    let blocks = blocks_from_segments(&segments);

    assert!(blocks
        .iter()
        .all(|block| !matches!(block, Block::Heading(_))));
}

#[test]
fn inserts_space_when_segments_are_separated_by_font_sized_gap() {
    // Two 12pt segments with a 4pt gap (~33% em) should join with a space, not concatenate.
    let segments = vec![
        TextSegment::new("Pow".to_string(), 10.0, 100.0, 12.0, 18.0),
        TextSegment::new("er".to_string(), 32.0, 100.0, 12.0, 12.0),
    ];

    let paragraph = first_paragraph(blocks_from_segments(&segments));

    assert_eq!(paragraph.content, vec![Inline::Text("Pow er".to_string())]);
}

#[test]
fn joins_tight_segments_without_inserting_a_space() {
    // Adjacent segments without gap stay joined.
    let segments = vec![
        TextSegment::new("Hello".to_string(), 10.0, 100.0, 12.0, 30.0),
        TextSegment::new("World".to_string(), 40.0, 100.0, 12.0, 30.0),
    ];

    let paragraph = first_paragraph(blocks_from_segments(&segments));

    assert_eq!(
        paragraph.content,
        vec![Inline::Text("HelloWorld".to_string())]
    );
}

#[test]
fn deduplicates_overlapping_text_segments() {
    let segments = segments(&[("Body", 10.0, 100.0), ("Body", 10.5, 100.2)]);

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

#[test]
fn emits_rotated_pdf_text_with_orientation_metadata() {
    let segments = vec![
        TextSegment::new("Rotated label".to_string(), 10.0, 120.0, 12.0, 70.0).with_rotation(90.0),
    ];

    let blocks = blocks_from_segments(&segments);

    let Block::RawHtml(raw) = &blocks[0] else {
        panic!("expected raw html for rotated text");
    };
    assert!(raw.html.contains("pdf-rotated-text"));
    assert!(raw.html.contains("data-rotation=\"90\""));
}

#[test]
fn promotes_substantial_page_images_after_the_preceding_text_block() {
    let segments = vec![segment("2.0 Section Heading", 55.0, 700.0)];
    let mut blocks = blocks_from_segments(&segments);
    let images = vec![visual_image(55.0, 518.0, 194.0, 152.0)];

    add_content_images_to_blocks(&mut blocks, &segments, &images, 600.0, 800.0, 14);

    assert!(matches!(blocks[0], Block::Paragraph(_) | Block::Heading(_)));
    let Block::Image(image) = &blocks[1] else {
        panic!("expected promoted image, got: {blocks:?}");
    };
    assert_eq!(image.alt.as_deref(), Some("PDF image on page 14"));
    assert_eq!(image.width, Some(194));
    assert_eq!(image.height, Some(152));
}

#[test]
fn does_not_promote_small_header_images_as_content() {
    let segments = vec![segment("Body text", 55.0, 700.0)];
    let mut blocks = blocks_from_segments(&segments);
    let images = vec![visual_image(55.0, 737.0, 68.0, 33.0)];

    add_content_images_to_blocks(&mut blocks, &segments, &images, 600.0, 800.0, 1);

    assert!(blocks.iter().all(|block| !matches!(block, Block::Image(_))));
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

fn assert_cell_text(table: &crate::Table, row: usize, column: usize, expected: &str) {
    assert_eq!(cell_text(table, row, column), expected);
}

fn cell_text(table: &crate::Table, row: usize, column: usize) -> &str {
    match &table.rows[row].cells[column].content[0] {
        Inline::Text(text) => text,
        _ => panic!("expected text cell"),
    }
}

fn segment(text: &str, x: f32, y: f32) -> TextSegment {
    TextSegment {
        text: text.to_string(),
        x,
        y,
        font_size: 12.0,
        width: text.len() as f32 * 6.0,
        rotation: 0.0,
        role: None,
        color: None,
        font_family: None,
        font_weight: None,
        font_style: None,
    }
}

fn small_segment(text: &str, x: f32, y: f32, width: f32) -> TextSegment {
    TextSegment::new(text.to_string(), x, y, 8.0, width)
}

fn segments(items: &[(&str, f32, f32)]) -> Vec<TextSegment> {
    items
        .iter()
        .map(|(text, x, y)| segment(text, *x, *y))
        .collect()
}

fn visual_image(x: f32, y: f32, width: f32, height: f32) -> VisualImage {
    VisualImage {
        src: "data:image/png;base64,AAAA".to_string(),
        mask_src: None,
        alt: "PDF image on page 14".to_string(),
        x,
        y,
        width,
        height,
    }
}
