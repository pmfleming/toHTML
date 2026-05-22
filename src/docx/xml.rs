use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::ConvertError;
use crate::{
    Asset, Block, ConversionWarning, Heading, Image, Inline, List, ListItem, Paragraph,
    SourceFormat, SourceSpan, Table, TableCell, TableRow,
};

use super::relationships::Relationships;

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
    let mut blocks = Vec::new();
    let mut assets = Vec::new();
    let mut warnings = Vec::new();

    loop {
        match reader.read_event()? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"p" => {
                push_paragraph(&mut reader, relationships, &mut blocks, &mut assets)?;
            }
            Event::Start(element) if local_name(element.name().as_ref()) == b"tbl" => {
                blocks.push(Block::Table(parse_table(&mut reader)?));
            }
            Event::Eof => break,
            Event::Start(element) => skip_element(&mut reader, element.name().as_ref())?,
            _ => {}
        }
    }

    if blocks.is_empty() {
        warnings.push(ConversionWarning {
            message: "DOCX document contained no supported body content".to_string(),
            source: docx_source(),
        });
    }

    Ok(ParsedDocument {
        blocks,
        assets,
        warnings,
    })
}

pub fn plain_text(inlines: &[Inline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(value) | Inline::Code(value) => text.push_str(value),
            Inline::Emphasis(children)
            | Inline::Strong(children)
            | Inline::Strikethrough(children) => text.push_str(&plain_text(children)),
            Inline::Link(link) => text.push_str(&plain_text(&link.content)),
            Inline::Image(image) => {
                if let Some(alt) = &image.alt {
                    text.push_str(alt);
                }
            }
            Inline::LineBreak => text.push(' '),
        }
    }
    text
}

fn push_paragraph(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    blocks: &mut Vec<Block>,
    assets: &mut Vec<Asset>,
) -> Result<(), ConvertError> {
    let paragraph = parse_paragraph(reader, relationships, assets)?;
    if paragraph.content.is_empty() && paragraph.images.is_empty() {
        return Ok(());
    }
    let images = paragraph.images.clone();
    if !paragraph.content.is_empty() {
        blocks.push(paragraph.into_text_block());
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
    fn into_text_block(self) -> Block {
        if let Some(level) = self.heading_level {
            return Block::Heading(Heading {
                level,
                content: self.content,
                source: docx_source(),
            });
        }
        if self.is_list {
            return Block::List(List {
                ordered: false,
                start: None,
                items: vec![ListItem {
                    checked: None,
                    blocks: vec![Block::Paragraph(Paragraph {
                        content: self.content,
                        source: docx_source(),
                    })],
                    source: docx_source(),
                }],
                source: docx_source(),
            });
        }
        Block::Paragraph(Paragraph {
            content: self.content,
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
            Event::Start(element) => match local_name(element.name().as_ref()) {
                b"pStyle" => paragraph.heading_level = heading_level(&element)?,
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
            },
            Event::Empty(element) => match local_name(element.name().as_ref()) {
                b"pStyle" => paragraph.heading_level = heading_level(&element)?,
                b"numPr" => paragraph.is_list = true,
                b"tab" => paragraph.content.push(Inline::Text("\t".to_string())),
                b"br" => paragraph.content.push(Inline::LineBreak),
                b"blip" => add_blip_image(&element, relationships, assets, &mut paragraph.images)?,
                _ => {}
            },
            Event::Text(text) => {
                let value = text
                    .decode()
                    .map_err(|error| ConvertError::Xml(error.to_string()))?;
                if !value.trim().is_empty() {
                    paragraph.content.push(Inline::Text(value.into_owned()));
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"p" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(paragraph)
}

fn parse_images(
    reader: &mut Reader<&[u8]>,
    relationships: &Relationships,
    assets: &mut Vec<Asset>,
) -> Result<Vec<Image>, ConvertError> {
    let mut images = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Empty(element) | Event::Start(element) => {
                if local_name(element.name().as_ref()) == b"blip" {
                    add_blip_image(&element, relationships, assets, &mut images)?;
                }
            }
            Event::End(element)
                if matches!(local_name(element.name().as_ref()), b"drawing" | b"pict") =>
            {
                break
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

fn parse_table(reader: &mut Reader<&[u8]>) -> Result<Table, ConvertError> {
    let mut rows = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"tr" => {
                rows.push(parse_table_row(reader)?);
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"tbl" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(Table {
        rows,
        caption: None,
        source: docx_source(),
    })
}

fn parse_table_row(reader: &mut Reader<&[u8]>) -> Result<TableRow, ConvertError> {
    let mut cells = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(element) if local_name(element.name().as_ref()) == b"tc" => {
                cells.push(parse_table_cell(reader)?);
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"tr" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(TableRow {
        cells,
        source: docx_source(),
    })
}

fn parse_table_cell(reader: &mut Reader<&[u8]>) -> Result<TableCell, ConvertError> {
    let mut content = Vec::new();
    let mut header = false;
    loop {
        match reader.read_event()? {
            Event::Start(element) => match local_name(element.name().as_ref()) {
                b"t" => content.push(Inline::Text(read_text(reader, b"t")?)),
                b"tblHeader" => header = true,
                _ => {}
            },
            Event::Empty(element) if local_name(element.name().as_ref()) == b"tblHeader" => {
                header = true;
            }
            Event::Text(text) => {
                let value = text
                    .decode()
                    .map_err(|error| ConvertError::Xml(error.to_string()))?;
                if !value.trim().is_empty() {
                    content.push(Inline::Text(value.into_owned()));
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == b"tc" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(TableCell {
        content,
        header,
        colspan: 1,
        rowspan: 1,
        align: None,
        source: docx_source(),
    })
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
            Event::Text(value) => {
                text.push_str(
                    &value
                        .decode()
                        .map_err(|error| ConvertError::Xml(error.to_string()))?,
                );
            }
            Event::End(element) if local_name(element.name().as_ref()) == end => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}

fn skip_element(reader: &mut Reader<&[u8]>, end: &[u8]) -> Result<(), ConvertError> {
    let mut depth = 1usize;
    while depth > 0 {
        match reader.read_event()? {
            Event::Start(_) => depth += 1,
            Event::End(element) if local_name(element.name().as_ref()) == local_name(end) => {
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

fn attr(element: &BytesStart<'_>, wanted: &[u8]) -> Result<Option<String>, ConvertError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ConvertError::Xml(error.to_string()))?;
        if local_name(attribute.key.as_ref()) == wanted {
            return Ok(Some(String::from_utf8_lossy(&attribute.value).into_owned()));
        }
    }
    Ok(None)
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

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn docx_source() -> Option<SourceSpan> {
    Some(SourceSpan {
        format: SourceFormat::Docx,
        page: None,
        path: None,
        byte_range: None,
    })
}
