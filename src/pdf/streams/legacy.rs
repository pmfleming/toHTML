use std::collections::HashMap;

use crate::ConvertError;

use super::filters;

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
    Ok(Some(filters::decode_legacy_stream(header, data)?))
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

pub(super) fn contains_token(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
