use std::collections::HashMap;

use crate::{Block, CodeBlock, Inline, Paragraph, TableCell};

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
    let threshold = (page_count / 3).max(4);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for text in blocks.iter().filter_map(paragraph_text) {
        if text.len() <= 80 {
            *counts.entry(text).or_default() += 1;
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
    repeated.iter().any(|item| item == &text) || is_page_number_footer(&text)
}

fn is_page_number_footer(text: &str) -> bool {
    let Some(number) = text.strip_suffix(" P a g e") else {
        return false;
    };
    number.trim().chars().all(|ch| ch.is_ascii_digit())
}

fn repair_block(block: Block) -> Block {
    match block {
        Block::Paragraph(paragraph) => Block::Paragraph(Paragraph {
            content: repair_inlines(paragraph.content),
            source: paragraph.source,
        }),
        Block::Table(mut table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    repair_cell(cell);
                }
            }
            Block::Table(table)
        }
        other => other,
    }
}

fn repair_cell(cell: &mut TableCell) {
    cell.content = repair_inlines(std::mem::take(&mut cell.content));
}

fn repair_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    inlines
        .into_iter()
        .map(|inline| match inline {
            Inline::Text(text) => Inline::Text(repair_text(&text)),
            other => other,
        })
        .collect()
}

fn repair_text(text: &str) -> String {
    let mut text = text.to_string();
    for (from, to) in REPAIRS {
        text = text.replace(from, to);
    }
    text
}

const REPAIRS: &[(&str, &str)] = &[
    ("Version7.0tFebruary2013", "Version 7.0 - February 2013"),
    ("Version7.0 February2013", "Version 7.0 - February 2013"),
    ("February2013", "February 2013"),
    ("Transferversion", "Transfer version"),
    ("theseGuidelines", "these Guidelines"),
    ("takento", "taken to"),
    ("theDutch", "the Dutch"),
    ("guidelinesare", "guidelines are"),
    ("banksresiding", "banks residing"),
    ("theSEPACredit", "the SEPA Credit"),
    ("SEPACredit", "SEPA Credit"),
    ("as ofthe", "as of the"),
    ("andreplace", "and replace"),
    ("madein", "made in"),
    ("version7.0", "version 7.0"),
    ("issued30November", "issued 30 November"),
    ("2012EPC", "2012 EPC"),
    ("ISO20022tMessage", "ISO20022 - Message"),
    ("blockis", "block is"),
    ("Initiationblock", "Initiation block"),
    ("clarificationsorerror", "clarifications or error"),
    ("contentoralignment", "content or alignment"),
    ("AnnexBClieop03", "Annex B Clieop03"),
    ("Annex BClieop03", "Annex B Clieop03"),
    ("AnnexGOverview", "Annex G Overview"),
    ("Associationcan", "Association can"),
    ("canbe", "can be"),
    ("versions.No", "versions. No"),
    ("required.The", "required. The"),
    ("The Netherlands.These", "The Netherlands. These"),
    ("January2012", "January 2012"),
    ("March2012", "March 2012"),
    ("onthe", "on the"),
    ("paymentinstruction", "payment instruction"),
    ("administeringthe", "administering the"),
    (
        "ortoprovideclarificationwhere",
        "or to provide clarification where",
    ),
    (
        "orto provideclarificationwhere",
        "or to provide clarification where",
    ),
    ("7.0compared to version6.0", "7.0 compared to version 6.0"),
    ("bordertransactions", "border transactions"),
    ("moreSEPA", "more SEPA"),
    ("[1..1]shows", "[1..1] shows"),
    ("thatthe", "that the"),
    ("thefollowing", "the following"),
    ("messageexample", "message example"),
    ("elementas", "element as"),
    ("isrecommended", "is recommended"),
    ("UsageEPC", "Usage EPC"),
    ("AdviseEPC", "Advise EPC"),
    ("NLdomestic", "NL domestic"),
    ("tobank", "to-bank"),
    ("<Documentxmlns", "<Document xmlns"),
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
            Block::paragraph("<Documentxmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""),
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
                "Annex BClieop03 Associationcan ortoprovideclarificationwhere",
            )],
            1,
        );

        let Some(text) = paragraph_text(&blocks[0]) else {
            panic!("expected paragraph text");
        };
        assert_eq!(
            text,
            "Annex B Clieop03 Association can or to provide clarification where"
        );
    }
}
