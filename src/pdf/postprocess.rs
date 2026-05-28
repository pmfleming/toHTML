use std::collections::{HashMap, HashSet};

use crate::{Block, CodeBlock, Inline, Link, Paragraph, TableCell};

pub fn blocks(blocks: Vec<Block>, page_count: usize) -> Vec<Block> {
    let repeated = repeated_short_paragraphs(&blocks, page_count);
    let blocks = blocks
        .into_iter()
        .filter(|block| !is_page_furniture(block, &repeated))
        .map(repair_block)
        .collect();
    let blocks = repair_iso20022_catalogue_link_blocks(blocks);
    collapse_code_blocks(blocks)
}

pub(super) fn link_artifacts(blocks: Vec<Block>) -> Vec<Block> {
    repair_iso20022_catalogue_link_blocks(blocks)
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
        || is_dash_wrapped_page_number(&text)
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

fn is_dash_wrapped_page_number(text: &str) -> bool {
    let trimmed = text.trim();
    let Some(without_left) = trimmed
        .strip_prefix('–')
        .or_else(|| trimmed.strip_prefix('-'))
    else {
        return false;
    };
    let Some(number) = without_left
        .trim()
        .strip_suffix('–')
        .or_else(|| without_left.trim().strip_suffix('-'))
    else {
        return false;
    };
    let number = number.trim();
    !number.is_empty() && number.len() <= 4 && number.chars().all(|ch| ch.is_ascii_digit())
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

fn repair_iso20022_catalogue_link_blocks(blocks: Vec<Block>) -> Vec<Block> {
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

fn repair_text(text: &str) -> String {
    let repaired = strip_license_artifact_runs(text);
    if looks_like_xml_line(&repaired) || looks_like_xml_continuation(&repaired) {
        return repaired;
    }
    repair_joined_word_boundaries(&repaired)
}

fn strip_license_artifact_runs(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < chars.len() {
        if is_license_artifact_char(chars[index]) {
            let run_start = index;
            while index < chars.len() && is_license_artifact_char(chars[index]) {
                index += 1;
            }

            let mut next_text = index;
            while next_text < chars.len() && chars[next_text].is_whitespace() {
                next_text += 1;
            }

            if index - run_start >= 12
                && (next_text == chars.len() || chars[next_text].is_alphanumeric())
            {
                if output.trim().is_empty() {
                    output.clear();
                } else if next_text == chars.len() {
                    while output.ends_with(char::is_whitespace) {
                        output.pop();
                    }
                } else if next_text < chars.len() && !output.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
                index = next_text;
                continue;
            }

            for ch in &chars[run_start..index] {
                output.push(*ch);
            }
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn is_license_artifact_char(ch: char) -> bool {
    matches!(ch, '`' | ',' | '-' | '\'' | '’' | '“' | '”')
}

fn repair_joined_word_boundaries(text: &str) -> String {
    text.split_whitespace()
        .map(repair_joined_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn repair_joined_token(token: &str) -> String {
    if token
        .chars()
        .any(|ch| matches!(ch, '<' | '>' | '/' | '_' | '\\'))
    {
        return token.to_string();
    }

    let chars = token.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(token.len());
    for index in 0..chars.len() {
        if joined_boundary(&chars, index) {
            output.push(' ');
        }
        output.push(chars[index]);
    }
    split_common_joined_pairs(&output)
}

fn joined_boundary(chars: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let left = chars[index - 1];
    let right = chars[index];
    if left.is_ascii_digit() && right.is_ascii_lowercase() {
        return true;
    }
    if left.is_ascii_lowercase() && right.is_ascii_digit() {
        let digit_run = chars[index..]
            .iter()
            .take_while(|ch| ch.is_ascii_digit())
            .count();
        return digit_run >= 2 || chars.get(index + digit_run) == Some(&'.');
    }
    left.is_ascii_lowercase() && right.is_ascii_uppercase() && index >= 5
}

fn split_common_joined_pairs(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            for (left, right) in COMMON_JOINED_WORD_PAIRS {
                if token.eq_ignore_ascii_case(&format!("{left}{right}")) {
                    return preserve_first_word_case(token, left, right);
                }
            }
            token.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn preserve_first_word_case(token: &str, left: &str, right: &str) -> String {
    let split_at = left.len();
    let (actual_left, _) = token.split_at(split_at.min(token.len()));
    format!("{actual_left} {right}")
}

const COMMON_JOINED_WORD_PAIRS: &[(&str, &str)] =
    &[("and", "can"), ("of", "over"), ("is", "a"), ("as", "a")];

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
mod tests;
