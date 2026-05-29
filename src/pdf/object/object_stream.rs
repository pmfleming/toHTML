use crate::pdf::compression;

use super::{PdfDictionary, PdfValue};

pub(super) fn stream_filters(dictionary: &PdfDictionary) -> Vec<String> {
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

pub(super) fn decode_object_stream(filters: &[String], data: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = data.to_vec();
    for filter in filters {
        decoded = match filter.as_str() {
            "FlateDecode" | "Fl" => compression::zlib_decode(&decoded).ok()?,
            _ => return None,
        };
    }
    Some(decoded)
}

pub(super) fn parse_object_stream_objects(
    decoded: &[u8],
    count: i64,
    first: i64,
) -> Option<Vec<(u32, &[u8])>> {
    let count = usize::try_from(count).ok()?;
    let first = usize::try_from(first).ok()?;
    if first > decoded.len() {
        return None;
    }

    let header = std::str::from_utf8(&decoded[..first]).ok()?;
    let numbers = header
        .split_whitespace()
        .filter_map(|token| token.parse::<usize>().ok())
        .collect::<Vec<_>>();
    if numbers.len() < count * 2 {
        return None;
    }

    let entries = numbers
        .chunks_exact(2)
        .take(count)
        .filter_map(|pair| Some((u32::try_from(pair[0]).ok()?, pair[1])))
        .collect::<Vec<_>>();

    let mut objects = Vec::new();
    for (index, (object, offset)) in entries.iter().enumerate() {
        let start = first.checked_add(*offset)?;
        let end = entries
            .get(index + 1)
            .map(|(_, next_offset)| first + *next_offset)
            .unwrap_or(decoded.len());
        if start >= end || end > decoded.len() {
            continue;
        }
        objects.push((*object, &decoded[start..end]));
    }
    Some(objects)
}
