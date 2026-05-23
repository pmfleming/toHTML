use std::{collections::HashMap, io::Read};

use flate2::read::ZlibDecoder;

use crate::ConvertError;

use super::hex::decode_hex_bytes;
use super::object::{PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

#[derive(Debug, Clone, Default)]
pub struct PdfPageExtraction {
    pub pages: Vec<PageContent>,
    pub warnings: Vec<String>,
    pub page_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PageContent {
    pub page_number: u32,
    pub streams: Vec<Vec<u8>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

pub fn document_pages(bytes: &[u8]) -> Result<PdfPageExtraction, ConvertError> {
    let objects = PdfObjects::parse(bytes);
    let page_refs = page_references(&objects);
    let pages = page_refs
        .iter()
        .enumerate()
        .map(|(index, reference)| page_content(bytes, &objects, *reference, index as u32 + 1))
        .collect::<Result<Vec<_>, _>>()?;
    let warnings = document_warnings(bytes, &objects, &pages);
    Ok(PdfPageExtraction {
        page_count: page_refs.len(),
        pages,
        warnings,
    })
}

fn page_references(objects: &PdfObjects) -> Vec<PdfReference> {
    catalog_pages(objects)
        .and_then(|root| walk_page_tree(objects, root))
        .filter(|pages| !pages.is_empty())
        .unwrap_or_else(|| flat_page_references(objects))
}

fn catalog_pages(objects: &PdfObjects) -> Option<PdfReference> {
    objects.values().find_map(|object| {
        (object.type_name() == Some("Catalog"))
            .then(|| object.dictionary()?.get_ref("Pages"))
            .flatten()
    })
}

fn walk_page_tree(objects: &PdfObjects, root: PdfReference) -> Option<Vec<PdfReference>> {
    let mut pages = Vec::new();
    collect_pages(objects, root, &mut pages);
    Some(pages)
}

fn collect_pages(objects: &PdfObjects, reference: PdfReference, pages: &mut Vec<PdfReference>) {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return;
    };
    match object.type_name() {
        Some("Page") => pages.push(reference),
        Some("Pages") => {
            if let Some(kids) = object
                .dictionary()
                .and_then(|dictionary| dictionary.array("Kids"))
            {
                for kid in kids.iter().filter_map(value_ref) {
                    collect_pages(objects, kid, pages);
                }
            }
        }
        _ => {}
    }
}

fn flat_page_references(objects: &PdfObjects) -> Vec<PdfReference> {
    objects
        .values()
        .filter(|object| object.type_name() == Some("Page"))
        .map(|object| object.reference)
        .collect()
}

fn page_content(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
    page_number: u32,
) -> Result<PageContent, ConvertError> {
    let Some(page) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return Ok(PageContent {
            page_number,
            streams: Vec::new(),
            width: None,
            height: None,
        });
    };
    let (width, height) = page
        .dictionary()
        .and_then(media_box_size)
        .unwrap_or((None, None));
    let content_refs = page
        .dictionary()
        .and_then(|dictionary| dictionary.get("Contents"))
        .map(content_references)
        .unwrap_or_default();
    let mut streams = Vec::new();
    for reference in content_refs {
        if let Some(stream) = stream_data_for_reference(source, objects, reference)? {
            streams.push(stream);
        }
    }
    Ok(PageContent {
        page_number,
        streams,
        width,
        height,
    })
}

fn media_box_size(dictionary: &super::object::PdfDictionary) -> Option<(Option<f32>, Option<f32>)> {
    let box_values = dictionary.array("MediaBox")?;
    let [left, bottom, right, top] = box_values else {
        return None;
    };
    let width = pdf_number(right)? - pdf_number(left)?;
    let height = pdf_number(top)? - pdf_number(bottom)?;
    Some((positive_dimension(width), positive_dimension(height)))
}

fn positive_dimension(value: f32) -> Option<f32> {
    value
        .is_finite()
        .then_some(value.abs())
        .filter(|value| *value > 0.0)
}

fn pdf_number(value: &PdfValue) -> Option<f32> {
    match value {
        PdfValue::Integer(value) => Some(*value as f32),
        PdfValue::Real(value) => Some(*value),
        _ => None,
    }
}

fn content_references(value: &PdfValue) -> Vec<PdfReference> {
    match value {
        PdfValue::Reference(reference) => vec![*reference],
        PdfValue::Array(values) => values.iter().filter_map(value_ref).collect(),
        _ => Vec::new(),
    }
}

fn stream_data_for_reference(
    source: &[u8],
    objects: &PdfObjects,
    reference: PdfReference,
) -> Result<Option<Vec<u8>>, ConvertError> {
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return stream_data_for_object(source, reference.object);
    };
    let Some(data) = &object.stream else {
        return Ok(None);
    };
    let filters = object.dictionary().map(stream_filters).unwrap_or_default();
    Ok(Some(decode_filters(&filters, data)?))
}

fn value_ref(value: &PdfValue) -> Option<PdfReference> {
    match value {
        PdfValue::Reference(reference) => Some(*reference),
        _ => None,
    }
}

pub fn object_body(bytes: &[u8], object_id: u32) -> Option<&[u8]> {
    let marker = format!("{object_id} 0 obj");
    let start = find_token(bytes, marker.as_bytes(), 0)? + marker.len();
    let end = find_token(bytes, b"endobj", start)?;
    Some(&bytes[start..end])
}

pub fn stream_data_for_object(
    bytes: &[u8],
    object_id: u32,
) -> Result<Option<Vec<u8>>, ConvertError> {
    let Some(body) = object_body(bytes, object_id) else {
        return Ok(None);
    };
    let Some(stream_start) = find_token(body, b"stream", 0) else {
        return Ok(None);
    };
    let Some(stream_end) = find_token(body, b"endstream", stream_start) else {
        return Ok(None);
    };
    let header = &body[..stream_start];
    let data = trim_stream_newlines(&body[stream_start + b"stream".len()..stream_end]);
    Ok(Some(decode_stream(header, data)?))
}

pub fn named_object_refs(bytes: &[u8], name_prefix: &str) -> HashMap<String, u32> {
    pdf_tokens(bytes)
        .windows(4)
        .filter_map(|window| named_object_ref(window, name_prefix))
        .collect()
}

pub fn object_ref_after(bytes: &[u8], marker: &[u8]) -> Option<u32> {
    let marker = String::from_utf8_lossy(marker);
    pdf_tokens(bytes)
        .windows(4)
        .find_map(|window| object_ref_window(window, &marker))
}

pub fn number_after(bytes: &[u8], marker: &[u8]) -> Option<u32> {
    let marker = String::from_utf8_lossy(marker);
    pdf_tokens(bytes)
        .windows(2)
        .find(|window| window[0] == marker)
        .and_then(|window| window[1].parse().ok())
}

pub fn font_name_after(bytes: &[u8], marker: &[u8]) -> Option<String> {
    pdf_tokens(bytes)
        .windows(2)
        .find(|window| window[0].as_bytes() == marker && window[1].starts_with("/F"))
        .map(|window| strip_name_prefix(&window[1]))
}

pub fn object_ids(bytes: &[u8]) -> Vec<u32> {
    pdf_tokens(bytes)
        .windows(3)
        .filter_map(|window| {
            (window[1] == "0" && window[2] == "obj")
                .then(|| window[0].parse().ok())
                .flatten()
        })
        .collect()
}

fn decode_stream(header: &[u8], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let filters = legacy_stream_filters(header);
    decode_filters(&filters, data)
}

fn decode_filters(filters: &[String], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut decoded = data.to_vec();
    for filter in filters {
        decoded = match filter.as_str() {
            "FlateDecode" | "Fl" => flate_decode(&decoded)?,
            "ASCIIHexDecode" | "AHx" => ascii_hex_decode(&decoded),
            "ASCII85Decode" | "A85" => ascii85_decode(&decoded)?,
            "RunLengthDecode" | "RL" => run_length_decode(&decoded),
            "DCTDecode" | "JPXDecode" | "CCITTFaxDecode" | "JBIG2Decode" => decoded,
            unsupported => {
                return Err(ConvertError::Pdf(format!(
                    "unsupported PDF stream filter {unsupported}"
                )))
            }
        };
    }
    Ok(decoded)
}

fn flate_decode(data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    Ok(decoded)
}

fn stream_filters(dictionary: &super::object::PdfDictionary) -> Vec<String> {
    match dictionary.get("Filter") {
        Some(PdfValue::Name(name)) => vec![name.clone()],
        Some(PdfValue::Array(values)) => values
            .iter()
            .filter_map(|value| match value {
                PdfValue::Name(name) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn legacy_stream_filters(header: &[u8]) -> Vec<String> {
    let names = [
        "ASCIIHexDecode",
        "ASCII85Decode",
        "LZWDecode",
        "FlateDecode",
        "RunLengthDecode",
        "DCTDecode",
        "JPXDecode",
        "CCITTFaxDecode",
        "JBIG2Decode",
    ];
    let header = String::from_utf8_lossy(header);
    names
        .into_iter()
        .filter(|name| header.contains(&format!("/{name}")))
        .map(str::to_string)
        .collect()
}

fn ascii_hex_decode(data: &[u8]) -> Vec<u8> {
    let digits: Vec<u8> = data
        .iter()
        .copied()
        .take_while(|byte| *byte != b'>')
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    decode_hex_bytes(&digits)
}

fn ascii85_decode(data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut output = Vec::new();
    let mut group = Vec::new();
    for byte in data
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        match byte {
            b'~' => break,
            b'z' if group.is_empty() => output.extend_from_slice(&[0, 0, 0, 0]),
            33..=117 => {
                group.push(byte - 33);
                if group.len() == 5 {
                    push_ascii85_group(&mut output, &group, 4)?;
                    group.clear();
                }
            }
            _ => {}
        }
    }
    if !group.is_empty() {
        let bytes_to_emit = group.len().saturating_sub(1);
        while group.len() < 5 {
            group.push(84);
        }
        push_ascii85_group(&mut output, &group, bytes_to_emit)?;
    }
    Ok(output)
}

fn push_ascii85_group(
    output: &mut Vec<u8>,
    group: &[u8],
    count: usize,
) -> Result<(), ConvertError> {
    let mut value = 0u32;
    for digit in group {
        value = value
            .checked_mul(85)
            .and_then(|value| value.checked_add(u32::from(*digit)))
            .ok_or_else(|| ConvertError::Pdf("invalid ASCII85 stream data".to_string()))?;
    }
    output.extend_from_slice(&value.to_be_bytes()[..count]);
    Ok(())
}

fn run_length_decode(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut index = 0;
    while let Some(length) = data.get(index).copied() {
        index += 1;
        match length {
            128 => break,
            0..=127 => {
                let count = usize::from(length) + 1;
                output.extend_from_slice(&data[index..(index + count).min(data.len())]);
                index += count;
            }
            129..=255 => {
                if let Some(byte) = data.get(index).copied() {
                    output.extend(std::iter::repeat_n(byte, 257 - usize::from(length)));
                }
                index += 1;
            }
        }
    }
    output
}

fn document_warnings(bytes: &[u8], objects: &PdfObjects, pages: &[PageContent]) -> Vec<String> {
    let mut warnings = Vec::new();
    if contains_token(bytes, b"/Encrypt") {
        warnings
            .push("PDF declares encryption; extraction may be blocked or incomplete".to_string());
    }
    if objects
        .values()
        .any(|object| object.type_name() == Some("XRef"))
    {
        warnings
            .push("PDF uses cross-reference streams; object stream support is limited".to_string());
    }
    if objects
        .values()
        .any(|object| object.type_name() == Some("ObjStm"))
    {
        warnings.push(
            "PDF uses object streams; compressed object extraction may be incomplete".to_string(),
        );
    }
    for page in pages.iter().filter(|page| page.streams.is_empty()) {
        warnings.push(format!(
            "Page {} has no supported extractable content stream",
            page.page_number
        ));
    }
    warnings
}

fn trim_stream_newlines(data: &[u8]) -> &[u8] {
    let data = data
        .strip_prefix(b"\r\n")
        .or_else(|| data.strip_prefix(b"\n"))
        .unwrap_or(data);
    data.strip_suffix(b"\r\n")
        .or_else(|| data.strip_suffix(b"\n"))
        .unwrap_or(data)
}

fn named_object_ref(window: &[String], name_prefix: &str) -> Option<(String, u32)> {
    if window[0].starts_with(name_prefix) && window[2] == "0" && window[3] == "R" {
        Some((strip_name_prefix(&window[0]), window[1].parse().ok()?))
    } else {
        None
    }
}

fn object_ref_window(window: &[String], marker: &str) -> Option<u32> {
    (window[0] == marker && window[2] == "0" && window[3] == "R")
        .then(|| window[1].parse().ok())
        .flatten()
}

fn strip_name_prefix(name: &str) -> String {
    name.trim_start_matches('/').to_string()
}

fn pdf_tokens(bytes: &[u8]) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() || is_token_delimiter(byte) => index += 1,
            _ => tokens.push(read_token(bytes, &mut index)),
        }
    }
    tokens
}

fn read_token(bytes: &[u8], index: &mut usize) -> String {
    let start = *index;
    *index += 1;
    while *index < bytes.len()
        && !bytes[*index].is_ascii_whitespace()
        && !is_token_delimiter(bytes[*index])
        && bytes[*index] != b'/'
    {
        *index += 1;
    }
    String::from_utf8_lossy(&bytes[start..*index]).to_string()
}

fn is_token_delimiter(byte: u8) -> bool {
    matches!(byte, b'<' | b'>' | b'[' | b']' | b'(' | b')')
}

fn find_token(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

fn contains_token(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
