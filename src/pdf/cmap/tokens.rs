use super::hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CMapToken {
    Word(String),
    Name(String),
    Integer(i64),
    Hex(Vec<u8>),
    ArrayStart,
    ArrayEnd,
}

pub(super) fn cmap_tokens(bytes: &[u8]) -> Vec<CMapToken> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'%' => {
                while index < bytes.len() && !matches!(bytes[index], b'\r' | b'\n') {
                    index += 1;
                }
            }
            b'[' => {
                tokens.push(CMapToken::ArrayStart);
                index += 1;
            }
            b']' => {
                tokens.push(CMapToken::ArrayEnd);
                index += 1;
            }
            b'/' => {
                index += 1;
                let start = index;
                while index < bytes.len() && !is_cmap_delimiter(bytes[index]) {
                    index += 1;
                }
                tokens.push(CMapToken::Name(
                    String::from_utf8_lossy(&bytes[start..index]).to_string(),
                ));
            }
            b'<' if bytes.get(index + 1) == Some(&b'<') => index += 2,
            b'>' if bytes.get(index + 1) == Some(&b'>') => index += 2,
            b'(' => index = skip_literal_string(bytes, index),
            b'<' => {
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index] != b'>' {
                    index += 1;
                }
                tokens.push(CMapToken::Hex(hex::hex_bytes_lossy(&bytes[start..index])));
                if index < bytes.len() {
                    index += 1;
                }
            }
            _ => {
                let start = index;
                while index < bytes.len() && !is_cmap_delimiter(bytes[index]) {
                    index += 1;
                }
                let word = String::from_utf8_lossy(&bytes[start..index]).to_string();
                if let Ok(value) = word.parse::<i64>() {
                    tokens.push(CMapToken::Integer(value));
                } else {
                    tokens.push(CMapToken::Word(word));
                }
            }
        }
    }
    tokens
}

fn is_cmap_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'/' | b'<' | b'>' | b'[' | b']' | b'(' | b')')
}

fn skip_literal_string(bytes: &[u8], mut index: usize) -> usize {
    index += 1;
    let mut depth = 1usize;
    while index < bytes.len() && depth > 0 {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            _ => index += 1,
        }
    }
    index
}
