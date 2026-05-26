#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
    Number(f32),
    NumberArray(Vec<f32>),
    Operator(String),
}

pub(super) fn tokenize(bytes: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        skip_ignored(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }
        match bytes[index] {
            b'(' => skip_literal_string(bytes, &mut index),
            b'<' if bytes.get(index + 1) != Some(&b'<') => skip_hex_string(bytes, &mut index),
            b'[' => tokens.push(Token::NumberArray(read_number_array(bytes, &mut index))),
            b']' | b'<' | b'>' | b'/' => skip_delimited_token(bytes, &mut index),
            _ => {
                let word = read_word(bytes, &mut index);
                if let Ok(value) = word.parse::<f32>() {
                    tokens.push(Token::Number(value));
                } else if !word.is_empty() {
                    tokens.push(Token::Operator(word));
                }
            }
        }
    }
    tokens
}

fn read_number_array(bytes: &[u8], index: &mut usize) -> Vec<f32> {
    *index += 1;
    let mut values = Vec::new();
    while *index < bytes.len() {
        skip_ignored(bytes, index);
        match bytes.get(*index) {
            Some(b']') => {
                *index += 1;
                break;
            }
            Some(b'(') => {
                skip_literal_string(bytes, index);
            }
            Some(b'<') if bytes.get(*index + 1) != Some(&b'<') => {
                skip_hex_string(bytes, index);
            }
            Some(_) => {
                let word = read_word(bytes, index);
                if let Ok(value) = word.parse::<f32>() {
                    values.push(value);
                } else if word.is_empty() {
                    *index += 1;
                }
            }
            None => break,
        }
    }
    values
}
fn skip_ignored(bytes: &[u8], index: &mut usize) {
    loop {
        while bytes.get(*index).is_some_and(u8::is_ascii_whitespace) {
            *index += 1;
        }
        if bytes.get(*index) != Some(&b'%') {
            break;
        }
        while *index < bytes.len() && !matches!(bytes[*index], b'\r' | b'\n') {
            *index += 1;
        }
    }
}

fn skip_literal_string(bytes: &[u8], index: &mut usize) {
    *index += 1;
    let mut depth = 1;
    while *index < bytes.len() && depth > 0 {
        match bytes[*index] {
            b'\\' => *index = (*index + 2).min(bytes.len()),
            b'(' => {
                depth += 1;
                *index += 1;
            }
            b')' => {
                depth -= 1;
                *index += 1;
            }
            _ => *index += 1,
        }
    }
}

fn skip_hex_string(bytes: &[u8], index: &mut usize) {
    *index += 1;
    while *index < bytes.len() && bytes[*index] != b'>' {
        *index += 1;
    }
    if *index < bytes.len() {
        *index += 1;
    }
}

fn skip_delimited_token(bytes: &[u8], index: &mut usize) {
    *index += 1;
    while *index < bytes.len()
        && !bytes[*index].is_ascii_whitespace()
        && !matches!(
            bytes[*index],
            b'[' | b']' | b'<' | b'>' | b'(' | b')' | b'/'
        )
    {
        *index += 1;
    }
}

fn read_word(bytes: &[u8], index: &mut usize) -> String {
    let start = *index;
    while *index < bytes.len()
        && !bytes[*index].is_ascii_whitespace()
        && !matches!(
            bytes[*index],
            b'[' | b']' | b'<' | b'>' | b'(' | b')' | b'/'
        )
    {
        *index += 1;
    }
    String::from_utf8_lossy(&bytes[start..*index]).to_string()
}
