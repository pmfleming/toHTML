use std::io::Read;

use flate2::read::ZlibDecoder;

use crate::ConvertError;

pub fn content_streams(bytes: &[u8]) -> Result<Vec<Vec<u8>>, ConvertError> {
    let mut streams = Vec::new();
    let mut cursor = 0;
    while let Some(start) = find_token(bytes, b"stream", cursor) {
        let Some(end) = find_token(bytes, b"endstream", start) else {
            break;
        };
        let header = &bytes[object_header_start(bytes, start)..start];
        let data = trim_stream_newlines(&bytes[start + b"stream".len()..end]);
        streams.push(decode_stream(header, data)?);
        cursor = end + b"endstream".len();
    }
    Ok(streams)
}

fn decode_stream(header: &[u8], data: &[u8]) -> Result<Vec<u8>, ConvertError> {
    if contains_token(header, b"/FlateDecode") {
        let mut decoder = ZlibDecoder::new(data);
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded)?;
        return Ok(decoded);
    }
    Ok(data.to_vec())
}

fn object_header_start(bytes: &[u8], stream_start: usize) -> usize {
    bytes[..stream_start]
        .windows(b"obj".len())
        .rposition(|window| window == b"obj")
        .map(|position| position + b"obj".len())
        .unwrap_or(0)
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
