mod names;
mod paragraphs;
mod tables;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::ConvertError;
use crate::{Asset, Block, ConversionWarning, Inline, SourceFormat, SourceSpan};

use super::relationships::Relationships;
use names::local_name;
use paragraphs::push_paragraph;
use tables::parse_table;

pub struct ParsedDocument {
    pub blocks: Vec<Block>,
    pub assets: Vec<Asset>,
    pub warnings: Vec<ConversionWarning>,
}

pub fn parse_document(
    xml: &str,
    relationships: &Relationships,
) -> Result<ParsedDocument, ConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut parsed = ParsedDocument {
        blocks: Vec::new(),
        assets: Vec::new(),
        warnings: Vec::new(),
    };

    loop {
        match reader.read_event()? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"p" => {
                push_paragraph(
                    &mut reader,
                    relationships,
                    &mut parsed.blocks,
                    &mut parsed.assets,
                )?;
            }
            Event::Start(element) if local_name(element.name().as_ref()) == b"tbl" => {
                parsed.blocks.push(Block::Table(parse_table(&mut reader)?));
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if parsed.blocks.is_empty() {
        parsed.warnings.push(ConversionWarning {
            message: "DOCX document contained no supported body content".to_string(),
            source: docx_source(),
        });
    }

    Ok(parsed)
}

pub fn plain_text(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        push_inline_text(&mut text, inline);
    }
    text
}

fn push_inline_text(text: &mut String, inline: &Inline) {
    match inline {
        Inline::Text(value) | Inline::Code(value) => text.push_str(value),
        Inline::Emphasis(children) | Inline::Strong(children) | Inline::Strikethrough(children) => {
            text.push_str(&plain_text(children));
        }
        Inline::Link(link) => text.push_str(&plain_text(&link.content)),
        Inline::Image(image) => {
            if let Some(alt) = &image.alt {
                text.push_str(alt);
            }
        }
        Inline::LineBreak => text.push(' '),
    }
}

pub(super) fn docx_source() -> Option<SourceSpan> {
    Some(SourceSpan {
        format: SourceFormat::Docx,
        page: None,
        path: None,
        byte_range: None,
    })
}
