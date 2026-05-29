use crate::{Block, Inline, Paragraph};

use super::paragraph_text;

pub(super) fn repair_iso20022_catalogue_link_blocks(blocks: Vec<Block>) -> Vec<Block> {
    let mut output = Vec::new();
    let mut skip_marker_blocks = 0usize;

    for block in blocks {
        if skip_marker_blocks > 0 && is_link_line_marker_block(&block) {
            skip_marker_blocks -= 1;
            continue;
        }

        if let Some(repaired) = split_iso20022_catalogue_link_block(&block) {
            output.extend(repaired);
            skip_marker_blocks = 2;
            continue;
        }

        if let Some(repaired) = strip_embedded_link_line_marker(&block) {
            output.push(repaired);
            continue;
        }

        output.push(block);
    }

    output
}

fn split_iso20022_catalogue_link_block(block: &Block) -> Option<Vec<Block>> {
    let Block::Paragraph(paragraph) = block else {
        return None;
    };
    let links = paragraph
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();
    if links.len() < 2 {
        return None;
    }
    let base_link = links
        .iter()
        .find(|link| is_iso20022_home_link(&link.href))?;
    let document_link = links
        .iter()
        .find(|link| is_iso20022_document_link(&link.href))?;

    let trailing = paragraph
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    if !trailing.contains("Catalogue") {
        return None;
    }

    let catalogue_text = repair_iso20022_catalogue_text(&trailing);
    let first = Block::Paragraph(Paragraph {
        content: vec![
            Inline::Link((*base_link).clone()),
            Inline::Text(format!(" {catalogue_text}")),
        ],
        source: paragraph.source.clone(),
    });
    let second = Block::Paragraph(Paragraph {
        content: vec![Inline::Link((*document_link).clone())],
        source: paragraph.source.clone(),
    });
    Some(vec![first, second])
}

fn strip_embedded_link_line_marker(block: &Block) -> Option<Block> {
    let Block::Paragraph(paragraph) = block else {
        return None;
    };
    let links = paragraph
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Link(link) if is_iso20022_document_link(&link.href) => Some(link.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if links.len() != 1 {
        return None;
    }
    let text = paragraph
        .content
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text.trim()),
            _ => None,
        })
        .collect::<String>();
    if !matches!(text.as_str(), "E" | "I" | "E." | "I.") {
        return None;
    }
    Some(Block::Paragraph(Paragraph {
        content: vec![Inline::Link(links[0].clone())],
        source: paragraph.source.clone(),
    }))
}

fn repair_iso20022_catalogue_text(text: &str) -> String {
    let mut repaired = text
        .trim()
        .trim_start_matches(|ch: char| ch == 'I' || ch == ',' || ch.is_whitespace())
        .to_string();
    if let Some(rest) = repaired.strip_prefix("under") {
        repaired = format!("under{}", rest);
    }
    repaired = repaired
        .replace("Catalogue of0,62 20022", "Catalogue of ISO 20022")
        .replace("Catalogue of 0,62 20022", "Catalogue of ISO 20022")
        .replace("pai)n.", "pain.")
        .replace("pai)n", "pain")
        .replace("pai n.", "pain.")
        .replace("pai n", "pain");
    if !repaired.ends_with('.') {
        repaired.push('.');
    }
    repaired
}

fn is_link_line_marker_block(block: &Block) -> bool {
    paragraph_text(block).is_some_and(|text| matches!(text.trim(), "E" | "I" | "2"))
}

fn is_iso20022_home_link(href: &str) -> bool {
    let trimmed = href.trim_end_matches('/');
    matches!(
        trimmed,
        "http://www.iso20022.org" | "https://www.iso20022.org"
    )
}

fn is_iso20022_document_link(href: &str) -> bool {
    let href = href.trim();
    (href.starts_with("http://www.iso20022.org/") || href.starts_with("https://www.iso20022.org/"))
        && href.len() > "http://www.iso20022.org/".len()
}
