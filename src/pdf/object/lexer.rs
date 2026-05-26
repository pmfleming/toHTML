use super::super::hex::decode_hex_bytes;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
    Null,
    Bool(bool),
    Integer(i64),
    Real(f32),
    Name(String),
    String(Vec<u8>),
    HexString(Vec<u8>),
    ArrayStart,
    ArrayEnd,
    DictStart,
    DictEnd,
    Word(String),
}

pub(super) struct Lexer<'a> {
    bytes: &'a [u8],
    pub(super) index: usize,
    pub(super) last_start: usize,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            index: 0,
            last_start: 0,
        }
    }

    pub(super) fn next_token(&mut self) -> Option<Token> {
        self.skip_ignored();
        self.last_start = self.index;
        match self.current()? {
            b'[' => self.single_byte(Token::ArrayStart),
            b']' => self.single_byte(Token::ArrayEnd),
            b'<' if self.peek() == Some(b'<') => {
                self.index += 2;
                Some(Token::DictStart)
            }
            b'>' if self.peek() == Some(b'>') => {
                self.index += 2;
                Some(Token::DictEnd)
            }
            b'<' => Some(Token::HexString(self.hex_string())),
            b'(' => Some(Token::String(self.literal_string())),
            b'/' => Some(Token::Name(self.name())),
            _ => self.word_token(),
        }
    }

    fn single_byte(&mut self, token: Token) -> Option<Token> {
        self.index += 1;
        Some(token)
    }

    fn literal_string(&mut self) -> Vec<u8> {
        self.index += 1;
        let mut depth = 1;
        let mut bytes = Vec::new();
        while self.index < self.bytes.len() && depth > 0 {
            match self.current() {
                Some(b'\\') => self.push_escape(&mut bytes),
                Some(b'(') => {
                    depth += 1;
                    bytes.push(b'(');
                    self.index += 1;
                }
                Some(b')') => {
                    depth -= 1;
                    if depth > 0 {
                        bytes.push(b')');
                    }
                    self.index += 1;
                }
                Some(byte) => {
                    bytes.push(byte);
                    self.index += 1;
                }
                None => break,
            }
        }
        bytes
    }

    fn push_escape(&mut self, bytes: &mut Vec<u8>) {
        self.index += 1;
        let Some(byte) = self.current() else {
            return;
        };
        match byte {
            b'n' => bytes.push(b'\n'),
            b'r' => bytes.push(b'\r'),
            b't' => bytes.push(b'\t'),
            b'b' => bytes.push(0x08),
            b'f' => bytes.push(0x0c),
            b'\n' => {}
            b'\r' => {
                if self.peek() == Some(b'\n') {
                    self.index += 1;
                }
            }
            b'0'..=b'7' => {
                self.index -= 1;
                self.push_octal(bytes);
                return;
            }
            other => bytes.push(other),
        }
        self.index += 1;
    }

    fn push_octal(&mut self, bytes: &mut Vec<u8>) {
        let start = self.index;
        let end = (start + 3).min(self.bytes.len());
        while self.index < end && matches!(self.current(), Some(b'0'..=b'7')) {
            self.index += 1;
        }
        if let Ok(value) =
            u8::from_str_radix(&String::from_utf8_lossy(&self.bytes[start..self.index]), 8)
        {
            bytes.push(value);
        }
    }

    fn hex_string(&mut self) -> Vec<u8> {
        self.index += 1;
        let start = self.index;
        while self.index < self.bytes.len() && self.current() != Some(b'>') {
            self.index += 1;
        }
        let bytes = decode_hex_bytes(&self.bytes[start..self.index]);
        if self.current() == Some(b'>') {
            self.index += 1;
        }
        bytes
    }

    fn name(&mut self) -> String {
        self.index += 1;
        let start = self.index;
        self.skip_regular();
        decode_name(&self.bytes[start..self.index])
    }

    fn word_token(&mut self) -> Option<Token> {
        let word = self.word()?;
        match word.as_str() {
            "null" => Some(Token::Null),
            "true" => Some(Token::Bool(true)),
            "false" => Some(Token::Bool(false)),
            _ => Some(
                word.parse::<i64>()
                    .map(Token::Integer)
                    .or_else(|_| word.parse::<f32>().map(Token::Real))
                    .unwrap_or(Token::Word(word)),
            ),
        }
    }

    fn word(&mut self) -> Option<String> {
        if is_delimiter(self.current()?) {
            self.index += 1;
            return None;
        }
        let start = self.index;
        self.skip_regular();
        Some(String::from_utf8_lossy(&self.bytes[start..self.index]).to_string())
    }

    fn skip_regular(&mut self) {
        while self.index < self.bytes.len() && !is_delimiter(self.bytes[self.index]) {
            self.index += 1;
        }
    }

    fn skip_ignored(&mut self) {
        while self.skip_whitespace() || self.skip_comment() {}
    }

    fn skip_whitespace(&mut self) -> bool {
        let start = self.index;
        while matches!(self.current(), Some(byte) if byte.is_ascii_whitespace()) {
            self.index += 1;
        }
        self.index != start
    }

    fn skip_comment(&mut self) -> bool {
        if self.current() != Some(b'%') {
            return false;
        }
        while self.index < self.bytes.len() && !matches!(self.current(), Some(b'\r' | b'\n')) {
            self.index += 1;
        }
        true
    }

    fn current(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index + 1).copied()
    }
}

fn decode_name(bytes: &[u8]) -> String {
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'#' && index + 2 < bytes.len() {
            if let Ok(value) =
                u8::from_str_radix(&String::from_utf8_lossy(&bytes[index + 1..index + 3]), 16)
            {
                output.push(value);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).to_string()
}

fn is_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(byte, b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'/' | b'%')
}
