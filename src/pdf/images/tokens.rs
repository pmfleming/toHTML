#[derive(Debug, Clone)]
pub(super) enum Token {
    Number(f32),
    Name(String),
    Operator(String),
}

#[derive(Debug, Clone)]
pub(super) enum Operand {
    Number(f32),
    Name(String),
}

pub(super) fn last_numbers<const N: usize>(operands: &[Operand]) -> Option<[f32; N]> {
    let numbers: Vec<f32> = operands
        .iter()
        .filter_map(|operand| match operand {
            Operand::Number(value) => Some(*value),
            Operand::Name(_) => None,
        })
        .collect();
    if numbers.len() < N {
        return None;
    }
    let start = numbers.len() - N;
    let mut values = [0.0; N];
    values.copy_from_slice(&numbers[start..]);
    Some(values)
}

pub(super) fn last_name(operands: &[Operand]) -> Option<&str> {
    operands.iter().rev().find_map(|operand| match operand {
        Operand::Name(name) => Some(name.as_str()),
        Operand::Number(_) => None,
    })
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
            b'/' => tokens.push(Token::Name(read_name(bytes, &mut index))),
            b'[' | b']' | b'<' | b'>' => skip_delimited_token(bytes, &mut index),
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

fn read_name(bytes: &[u8], index: &mut usize) -> String {
    *index += 1;
    read_word(bytes, index)
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

fn skip_delimited_token(_bytes: &[u8], index: &mut usize) {
    *index += 1;
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
