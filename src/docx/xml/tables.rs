use quick_xml::events::Event;
use quick_xml::Reader;

use crate::ConvertError;
use crate::{Inline, Table, TableCell, TableRow};

use super::docx_source;
use super::names::local_name;

pub fn parse_table(reader: &mut Reader<&[u8]>) -> Result<Table, ConvertError> {
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
                if let Ok(value) = text.decode() {
                    if !value.trim().is_empty() {
                        content.push(Inline::Text(value.into_owned()));
                    }
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

fn read_text(reader: &mut Reader<&[u8]>, end: &[u8]) -> Result<String, ConvertError> {
    let mut text = String::new();
    loop {
        match reader.read_event()? {
            Event::Text(value) => {
                if let Ok(value) = value.decode() {
                    text.push_str(&value);
                }
            }
            Event::End(element) if local_name(element.name().as_ref()) == end => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}
