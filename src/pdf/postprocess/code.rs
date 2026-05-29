use crate::{Block, CodeBlock};

use super::paragraph_text;

pub(super) fn collapse_code_blocks(blocks: Vec<Block>) -> Vec<Block> {
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

pub(super) fn looks_like_xml_line(text: &str) -> bool {
    let text = text.trim();
    text.starts_with('<')
}

pub(super) fn looks_like_xml_continuation(text: &str) -> bool {
    let text = text.trim_start();
    text.starts_with("xmlns")
        || text.starts_with("xsi:")
        || text.starts_with("schemaLocation")
        || text.starts_with("targetNamespace")
}
