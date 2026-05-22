use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::ConvertError;
use crate::{Asset, Block, Heading, Image, Inline, List, ListItem, Paragraph};

use super::docx_source;
use super::names::{attr, local_name};
use super::Relationships;

pub fn push_paragraph(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    blocks: &mut Vec<Block>,
    assets: &mut Vec<Asset>,
) -> Result<(), ConvertError> {
    let paragraph = parse_paragraph(reader, relationships, assets)?;
    if paragraph.is_empty() {
        return Ok(());
    }
    let (text_block, images) = paragraph.into_blocks();
    if let Some(block) = text_block {
        blocks.push(block);
    }
    blocks.extend(images.into_iter().map(Block::Image));
    Ok(())
}

struct ParsedParagraph {
    content: Vec<Inline>,
    images: Vec<Image>,
    heading_level: Option<u8>,
    is_list: bool,
}

impl ParsedParagraph {
    fn is_empty(&self) -> bool {
        self.content.is_empty() && self.images.is_empty()
    }

    fn into_blocks(self) -> (Option<Block>, Vec<Image>) {
        let content = self.content;
        let images = self.images;
        let block = if content.is_empty() {
            None
        } else {
            Some(Self::text_block(content, self.heading_level, self.is_list))
        };
        (block, images)
    }

    fn text_block(content: Vec<Inline>, heading_level: Option<u8>, is_list: bool) -> Block {
        if let Some(level) = heading_level {
            return Block::Heading(Heading {
                level,
                content,
                source: docx_source(),
            });
        }
        if is_list {
            return Block::List(List {
                ordered: false,
                start: None,
                items: vec![ListItem {
                    checked: None,
                    blocks: vec![Block::Paragraph(Paragraph {
                        content,
                        source: docx_source(),
                    })],
                    source: docx_source(),
                }],
                source: docx_source(),
            });
        }
        Block::Paragraph(Paragraph {
            content,
            source: docx_source(),
        })
    }
}

fn parse_paragraph(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
) -> Result<ParsedParagraph, ConvertError> {
    let mut paragraph = ParsedParagraph {
        content: Vec::new(),
        images: Vec::new(),
        heading_level: None,
        is_list: false,
    };

    loop {
        match reader.read_event()? {
            Event::Start(element) => {
                handle_paragraph_start(reader, relationships, assets, &mut paragraph, &element)?
            }
            Event::Empty(element) => {
                handle_paragraph_empty(relationships, assets, &mut paragraph, &element)?
            }
            Event::Text(text) => push_decoded_text(&mut paragraph.content, text.decode()),
            Event::End(element) if local_name(element.name().as_ref()) == b"p" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(paragraph)
}

fn handle_paragraph_start(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
    paragraph: &mut ParsedParagraph,
    element: &BytesStart<'_>,
) -> Result<(), ConvertError> {
    match local_name(element.name().as_ref()) {
        b"pStyle" => paragraph.heading_level = heading_level(element)?,
        b"numPr" => paragraph.is_list = true,
        b"t" => paragraph
            .content
            .push(Inline::Text(read_text(reader, b"t")?)),
        b"tab" => paragraph.content.push(Inline::Text("\t".to_string())),
        b"br" => paragraph.content.push(Inline::LineBreak),
        b"drawing" | b"pict" => {
            paragraph
                .images
                .extend(parse_images(reader, relationships, assets)?);
        }
        _ => {}
    }
    Ok(())
}

fn handle_paragraph_empty(
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
    paragraph: &mut ParsedParagraph,
    element: &BytesStart<'_>,
) -> Result<(), ConvertError> {
    match local_name(element.name().as_ref()) {
        b"pStyle" => paragraph.heading_level = heading_level(element)?,
        b"numPr" => paragraph.is_list = true,
        b"tab" => paragraph.content.push(Inline::Text("\t".to_string())),
        b"br" => paragraph.content.push(Inline::LineBreak),
        b"blip" => add_blip_image(element, relationships, assets, &mut paragraph.images)?,
        _ => {}
    }
    Ok(())
}

fn parse_images(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
) -> Result<Vec<Image>, ConvertError> {
    let mut images = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Empty(element) | Event::Start(element)
                if local_name(element.name().as_ref()) == b"blip" =>
            {
                add_blip_image(&element, relationships, assets, &mut images)?;
            }
            Event::End(element)
                if matches!(local_name(element.name().as_ref()), b"drawing" | b"pict") =>
            {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(images)
}

fn add_blip_image(
    element: &BytesStart<'_>,
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
    images: &mut Vec<Image>,
) -> Result<(), ConvertError> {
    let Some(id) = attr(element, b"embed")? else {
        return Ok(());
    };
    let Some(path) = relationships.target(&id) else {
        return Ok(());
    };
    let asset_id = format!("docx-image-{}", assets.len() + 1);
    assets.push(Asset {
        id: asset_id.clone(),
        path: path.clone(),
        media_type: media_type(&path).map(str::to_string),
        alt: None,
        source: docx_source(),
    });
    images.push(Image {
        src: path,
        alt: None,
        title: None,
        asset_id: Some(asset_id),
        source: docx_source(),
    });
    Ok(())
}

fn heading_level(element: &BytesStart<'_>) -> Result<Option<u8>, ConvertError> {
    let Some(value) = attr(element, b"val")? else {
        return Ok(None);
    };
    let normalized = value.to_ascii_lowercase();
    if !normalized.starts_with("heading") {
        return Ok(None);
    }
    Ok(normalized
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse::<u8>()
        .ok()
        .map(|level| level.clamp(1, 6)))
}

fn read_text(reader: &mut Reader<&[u8]>, end: &[u8]) -> Result<String, ConvertError> {
    let mut text = String::new();
    loop {
        match reader.read_event()? {
            Event::Text(value) => push_decoded_text(&mut text, value.decode()),
            Event::End(element) if local_name(element.name().as_ref()) == end => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}

fn push_decoded_text<T, E>(target: &mut T, decoded: Result<std::borrow::Cow<'_, str>, E>)
where
    T: PushText,
{
    if let Ok(value) = decoded {
        target.push_text(value.as_ref());
    }
}

trait PushText {
    fn push_text(&mut self, value: &str);
}

impl PushText for String {
    fn push_text(&mut self, value: &str) {
        self.push_str(value);
    }
}

impl PushText for Vec<Inline> {
    fn push_text(&mut self, value: &str) {
        if !value.trim().is_empty() {
            self.push(Inline::Text(value.to_string()));
        }
    }
}

fn media_type(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}
