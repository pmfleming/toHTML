use std::io::Read;

use flate2::read::ZlibDecoder;

use crate::ConvertError;

use super::super::hex::decode_hex_bytes;
use super::super::object::{PdfDictionary, PdfValue};

pub(super) fn decode_legacy_stream(header: &[u8], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let filters = legacy_stream_filters(header);
    decode_filters(&filters, data)
}

pub(super) fn decode_filters(filters: &[String], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
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

fn flate_decode(data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    Ok(decoded)
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
