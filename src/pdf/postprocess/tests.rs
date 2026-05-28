use super::*;

#[test]
fn removes_repeated_headers_and_page_footers() {
    let blocks = vec![
        Block::paragraph("Example recurring document header"),
        Block::paragraph("1 P a g e"),
        Block::paragraph("Body"),
        Block::paragraph("Example recurring document header"),
        Block::paragraph("2 P a g e"),
        Block::paragraph("More"),
        Block::paragraph("Example recurring document header"),
        Block::paragraph("3 P a g e"),
        Block::paragraph("End"),
        Block::paragraph("Example recurring document header"),
    ];

    let blocks = super::blocks(blocks, 12);

    assert_eq!(blocks.len(), 3);
}

#[test]
fn removes_repeated_short_pdf_footers_in_three_page_documents() {
    let blocks = vec![
        Block::paragraph("Example footer contact line 1/3"),
        Block::paragraph("Quote body"),
        Block::paragraph("Example footer contact line 2/3"),
        Block::paragraph("More body"),
        Block::paragraph("Example footer contact line 3/3"),
        Block::paragraph("Final body"),
    ];

    let blocks = super::blocks(blocks, 3);
    let text = blocks
        .iter()
        .filter_map(paragraph_text)
        .collect::<Vec<_>>()
        .join(" ");

    assert!(!text.contains("Example footer"));
    assert!(text.contains("Quote body"));
    assert!(text.contains("Final body"));
}

#[test]
fn removes_dash_wrapped_page_numbers_from_semantic_output() {
    let blocks = vec![
        Block::paragraph("– 23 –"),
        Block::paragraph("Annex A"),
        Block::paragraph("- 24 -"),
    ];

    let blocks = super::blocks(blocks, 30);
    let text = blocks
        .iter()
        .filter_map(paragraph_text)
        .collect::<Vec<_>>()
        .join(" ");

    assert_eq!(text, "Annex A");
}

#[test]
fn collapses_xml_paragraphs_to_code_block() {
    let blocks = vec![
        Block::paragraph("Intro"),
        Block::paragraph("<?xml version=\"1.0\"?>"),
        Block::paragraph("<Document>"),
        Block::paragraph("</Document>"),
    ];

    let blocks = super::blocks(blocks, 1);

    assert!(matches!(blocks[1], Block::CodeBlock(_)));
}

#[test]
fn keeps_multiline_xml_opening_in_code_block() {
    let blocks = vec![
        Block::paragraph("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
        Block::paragraph("<Document xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""),
        Block::paragraph("xmlns=\"urn:iso:std:iso:20022:tech:xsd:pain.001.001.03\""),
        Block::paragraph(
            "xsi:schemaLocation=\"urn:iso:std:iso:20022:tech:xsd:pain.001.001.03 file.xsd\">",
        ),
        Block::paragraph("<CstmrCdtTrfInitn>"),
        Block::paragraph("</CstmrCdtTrfInitn>"),
        Block::paragraph("</Document>"),
    ];

    let blocks = super::blocks(blocks, 1);

    let Block::CodeBlock(code) = &blocks[0] else {
        panic!("expected xml code block");
    };
    assert!(code.code.starts_with("<?xml"));
    assert!(code.code.contains("<Document xmlns:xsi"));
    assert!(code.code.contains("xsi:schemaLocation"));
}

#[test]
fn repairs_common_pdf_word_joins() {
    let blocks = super::blocks(
        vec![Block::paragraph("Version7.0 published2024 withoutChanging")],
        1,
    );

    let Some(text) = paragraph_text(&blocks[0]) else {
        panic!("expected paragraph text");
    };
    assert_eq!(text, "Version 7.0 published 2024 without Changing");
}

#[test]
fn strips_leading_license_punctuation_attached_to_prose() {
    let blocks = super::blocks(
        vec![Block::paragraph(
            "--`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested under conditions.",
        )],
        1,
    );

    let Some(text) = paragraph_text(&blocks[0]) else {
        panic!("expected paragraph text");
    };
    assert_eq!(text, "equipment tested under conditions.");
}

#[test]
fn strips_embedded_license_punctuation_attached_to_prose() {
    let blocks = super::blocks(
            vec![Block::paragraph(
                "which can be produced by --`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---equipment tested under conditions.",
            )],
            1,
        );

    let Some(text) = paragraph_text(&blocks[0]) else {
        panic!("expected paragraph text");
    };
    assert_eq!(
        text,
        "which can be produced by equipment tested under conditions."
    );
}

#[test]
fn strips_trailing_license_punctuation_attached_to_prose() {
    let blocks = super::blocks(
        vec![Block::paragraph(
            "equipment; --`,```,,,,,`,,````````,,,,``,`-`-`,,`,,`,`,,`---",
        )],
        1,
    );

    let Some(text) = paragraph_text(&blocks[0]) else {
        panic!("expected paragraph text");
    };
    assert_eq!(text, "equipment;");
}

#[test]
fn repairs_joined_common_words() {
    let blocks = super::blocks(vec![Block::paragraph("document andcan be subject")], 1);

    let Some(text) = paragraph_text(&blocks[0]) else {
        panic!("expected paragraph text");
    };
    assert_eq!(text, "document and can be subject");
}

#[test]
fn splits_iso20022_catalogue_prose_from_following_url_line() {
    let blocks = super::blocks(
            vec![
                Block::Paragraph(Paragraph {
                    content: vec![
                        Inline::Link(Link {
                            href: "http://www.iso20022.org/".to_string(),
                            title: None,
                            content: vec![Inline::Text("www.iso20022.org".to_string())],
                            source: None,
                        }),
                        Inline::Link(Link {
                            href: "http://www.iso20022.org/documents/messages/pain/schemas/pain.001.001.03.zip"
                                .to_string(),
                            title: None,
                            content: vec![Inline::Text(
                                "www.iso20022.org/documents/messages/pain/schemas/pain.001.001.03.zip"
                                    .to_string(),
                            )],
                            source: None,
                        }),
                        Inline::Text(
                            "I under “Catalogue of0,62 20022 messages”, with “pai n.001.001.03” as reference."
                                .to_string(),
                        ),
                    ],
                    source: None,
                }),
                Block::paragraph("E"),
                Block::paragraph("2"),
            ],
            1,
        );

    assert_eq!(blocks.len(), 2);
    let Block::Paragraph(first) = &blocks[0] else {
        panic!("expected first paragraph");
    };
    assert!(matches!(first.content[0], Inline::Link(_)));
    let first_text = paragraph_text(&blocks[0]).unwrap();
    assert_eq!(
        first_text,
        " under “Catalogue of ISO 20022 messages”, with “pain.001.001.03” as reference."
    );
    let Block::Paragraph(second) = &blocks[1] else {
        panic!("expected second paragraph");
    };
    assert!(matches!(second.content.as_slice(), [Inline::Link(_)]));
}
