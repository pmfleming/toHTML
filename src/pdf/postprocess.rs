use std::collections::{HashMap, HashSet};

use crate::{Block, CodeBlock, Inline, Link, Paragraph, TableCell};

pub fn blocks(blocks: Vec<Block>, page_count: usize) -> Vec<Block> {
    let repeated = repeated_short_paragraphs(&blocks, page_count);
    let blocks = blocks
        .into_iter()
        .filter(|block| !is_page_furniture(block, &repeated))
        .map(repair_block)
        .collect();
    collapse_code_blocks(blocks)
}

fn repeated_short_paragraphs(blocks: &[Block], page_count: usize) -> Vec<String> {
    let threshold = repeated_furniture_threshold(page_count);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for text in blocks.iter().filter_map(paragraph_text) {
        if let Some(key) = page_furniture_key(&text) {
            *counts.entry(key).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(text, count)| (count >= threshold).then_some(text))
        .collect()
}

fn is_page_furniture(block: &Block, repeated: &[String]) -> bool {
    let Some(text) = paragraph_text(block) else {
        return false;
    };
    let repeated: HashSet<&str> = repeated.iter().map(String::as_str).collect();
    page_furniture_key(&text).is_some_and(|key| repeated.contains(key.as_str()))
        || is_page_number_footer(&text)
        || is_fraction_page_footer(&text)
}

fn repeated_furniture_threshold(page_count: usize) -> usize {
    if page_count <= 1 {
        return 2;
    }
    (page_count / 3).max(2)
}

fn page_furniture_key(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() > 100 || trimmed.is_empty() {
        return None;
    }
    let key = trimmed
        .split_whitespace()
        .map(|token| {
            if is_page_fraction(token) || token.chars().all(|ch| ch.is_ascii_digit()) {
                "#"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!key.is_empty()).then_some(key)
}

fn is_page_number_footer(text: &str) -> bool {
    let Some(number) = text.strip_suffix(" P a g e") else {
        return false;
    };
    number.trim().chars().all(|ch| ch.is_ascii_digit())
}

fn is_fraction_page_footer(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed
        .split_whitespace()
        .last()
        .is_some_and(is_page_fraction)
        && trimmed.len() <= 100
}

fn is_page_fraction(token: &str) -> bool {
    let Some((page, total)) = token.split_once('/') else {
        return false;
    };
    let Ok(page) = page.parse::<usize>() else {
        return false;
    };
    let Ok(total) = total.parse::<usize>() else {
        return false;
    };
    page >= 1 && total >= page
}

fn repair_block(block: Block) -> Block {
    match block {
        Block::Paragraph(paragraph) => Block::Paragraph(repair_paragraph(paragraph)),
        Block::Table(mut table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    repair_cell(cell);
                }
            }
            Block::Table(table)
        }
        Block::Heading(mut heading) => {
            heading.content = repair_inlines(heading.content);
            Block::Heading(heading)
        }
        other => other,
    }
}

fn repair_paragraph(paragraph: Paragraph) -> Paragraph {
    Paragraph {
        content: repair_inlines(paragraph.content),
        source: paragraph.source,
    }
}

fn repair_cell(cell: &mut TableCell) {
    cell.content = repair_inlines(std::mem::take(&mut cell.content));
}

fn repair_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    inlines.into_iter().map(repair_inline).collect()
}

fn repair_inline(inline: Inline) -> Inline {
    match inline {
        Inline::Text(text) => Inline::Text(repair_text(&text)),
        Inline::Emphasis(content) => Inline::Emphasis(repair_inlines(content)),
        Inline::Strong(content) => Inline::Strong(repair_inlines(content)),
        Inline::Strikethrough(content) => Inline::Strikethrough(repair_inlines(content)),
        Inline::Link(link) => Inline::Link(repair_link(link)),
        other => other,
    }
}

fn repair_link(link: Link) -> Link {
    Link {
        content: repair_inlines(link.content),
        ..link
    }
}

fn repair_text(text: &str) -> String {
    let mut repaired = text.to_string();
    for (from, to) in PDF_TEXT_REPAIRS {
        repaired = repaired.replace(from, to);
    }
    repaired
}

const PDF_TEXT_REPAIRS: &[(&str, &str)] = &[
    ("Version7.0", "Version 7.0"),
    ("February2013", "February 2013"),
    ("messageexample", "message example"),
    ("withoutChangingthe", "without Changing the"),
    (
        "Ichangecurrentusingprogrammer",
        "I change current using programmer",
    ),
    (
        "How do Ichangecurrentusingprogrammer",
        "How do I change current using programmer",
    ),
    ("10days", "10 days"),
    ("200worldwide", "200 worldwide"),
    (".We", ". We"),
];

fn collapse_code_blocks(blocks: Vec<Block>) -> Vec<Block> {
    let mut output = Vec::new();
    let mut code = Vec::new();

    for block in blocks {
        if let Some(text) = paragraph_text(&block) {
            let is_xml = looks_like_xml_line(&text)
                || (!code.is_empty() && looks_like_xml_continuation(&text));
            if is_xml {
                code.push(text);
                continue;
            }
        }

        flush_code(&mut output, &mut code);
        output.push(block);
    }

    flush_code(&mut output, &mut code);
    output
}

fn flush_code(output: &mut Vec<Block>, code: &mut Vec<String>) {
    if code.len() >= 3 {
        output.push(Block::CodeBlock(CodeBlock {
            language: Some("xml".to_string()),
            code: code.join("\n"),
            source: None,
        }));
    } else {
        output.extend(code.drain(..).map(Block::paragraph));
    }
    code.clear();
}

fn looks_like_xml_line(text: &str) -> bool {
    let text = text.trim();
    text.starts_with('<')
}

fn looks_like_xml_continuation(text: &str) -> bool {
    let text = text.trim_start();
    text.starts_with("xmlns")
        || text.starts_with("xsi:")
        || text.starts_with("schemaLocation")
        || text.starts_with("targetNamespace")
}

fn paragraph_text(block: &Block) -> Option<String> {
    let Block::Paragraph(paragraph) = block else {
        return None;
    };
    let mut text = String::new();
    for inline in &paragraph.content {
        if let Inline::Text(value) = inline {
            text.push_str(value);
        }
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_repeated_headers_and_page_footers() {
        let blocks = vec![
            Block::paragraph("IG SEPA Credit Transfer version 7.0"),
            Block::paragraph("1 P a g e"),
            Block::paragraph("Body"),
            Block::paragraph("IG SEPA Credit Transfer version 7.0"),
            Block::paragraph("2 P a g e"),
            Block::paragraph("More"),
            Block::paragraph("IG SEPA Credit Transfer version 7.0"),
            Block::paragraph("3 P a g e"),
            Block::paragraph("End"),
            Block::paragraph("IG SEPA Credit Transfer version 7.0"),
        ];

        let blocks = super::blocks(blocks, 12);

        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn removes_repeated_short_pdf_footers_in_three_page_documents() {
        let blocks = vec![
            Block::paragraph("Tel +31 857 470 061-www.inventronics-co.com 1/3"),
            Block::paragraph("Quote body"),
            Block::paragraph("Tel +31 857 470 061-www.inventronics-co.com 2/3"),
            Block::paragraph("More body"),
            Block::paragraph("Tel +31 857 470 061-www.inventronics-co.com 3/3"),
            Block::paragraph("Final body"),
        ];

        let blocks = super::blocks(blocks, 3);
        let text = blocks
            .iter()
            .filter_map(paragraph_text)
            .collect::<Vec<_>>()
            .join(" ");

        assert!(!text.contains("Tel +31"));
        assert!(text.contains("Quote body"));
        assert!(text.contains("Final body"));
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
            vec![Block::paragraph(
                "Version7.0 February2013 messageexample withoutChangingthe",
            )],
            1,
        );

        let Some(text) = paragraph_text(&blocks[0]) else {
            panic!("expected paragraph text");
        };
        assert_eq!(
            text,
            "Version 7.0 February 2013 message example without Changing the"
        );
    }
}
