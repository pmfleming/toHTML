use super::strings::decode_hex_bytes;
use super::syntax::is_delimiter;

pub(super) enum Token {
    Literal(Vec<u8>),
    Array(Vec<ArrayToken>),
    Hex(Vec<u8>),
    Name(String),
    ActualText(Vec<u8>),
    Word(String),
}

#[derive(Debug, Clone)]
pub(super) enum ArrayToken {
    Text(Vec<u8>),
    Adjustment(f32),
}

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_ignored();
        match self.current()? {
            b'(' => Some(Token::Literal(self.literal_string())),
            b'[' => Some(Token::Array(self.array())),
            b'<' if self.peek() == Some(b'<') => self
                .dictionary_actual_text()
                .map(Token::ActualText)
                .or_else(|| self.next_token()),
            b'<' => Some(Token::Hex(self.hex_string())),
            b'/' => Some(Token::Name(self.name())),
            _ => self.word().map(Token::Word),
        }
    }

    fn literal_string(&mut self) -> Vec<u8> {
        self.index += 1;
        let mut depth = 1;
        let mut bytes = Vec::new();

        while self.index < self.bytes.len() && depth > 0 {
            match self.current() {
                Some(b'\\') => self.push_escape(&mut bytes),
                Some(b'(') => self.push_nested_open(&mut bytes, &mut depth),
                Some(b')') => self.push_nested_close(&mut bytes, &mut depth),
                Some(byte) => self.push_raw_byte(&mut bytes, byte),
                None => break,
            }
        }

        bytes
    }

    fn array(&mut self) -> Vec<ArrayToken> {
        self.index += 1;
        let mut items = Vec::new();

        while self.index < self.bytes.len() {
            self.skip_ignored();
            match self.current() {
                Some(b']') => break,
                Some(b'(') => items.push(ArrayToken::Text(self.literal_string())),
                Some(b'<') if self.peek() == Some(b'<') => {
                    self.dictionary_actual_text();
                }
                Some(b'<') => items.push(ArrayToken::Text(self.hex_string())),
                Some(b'/') => {
                    self.name();
                }
                Some(_) => self.push_array_word(&mut items),
                None => break,
            }
        }

        self.skip_array_end();
        items
    }

    fn hex_string(&mut self) -> Vec<u8> {
        self.index += 1;
        let start = self.index;
        while self.index < self.bytes.len() && self.current() != Some(b'>') {
            self.index += 1;
        }

        let bytes = decode_hex_bytes(&self.bytes[start..self.index]);
        self.skip_hex_end();
        bytes
    }

    fn name(&mut self) -> String {
        self.index += 1;
        let start = self.index;
        self.skip_word();
        String::from_utf8_lossy(&self.bytes[start..self.index]).to_string()
    }

    fn word(&mut self) -> Option<String> {
        if matches!(self.current(), Some(byte) if is_delimiter(byte)) {
            self.index += 1;
            return None;
        }

        let start = self.index;
        self.skip_word();
        Some(String::from_utf8_lossy(&self.bytes[start..self.index]).to_string())
    }

    fn push_array_word(&mut self, items: &mut Vec<ArrayToken>) {
        if let Some(word) = self.word() {
            if let Ok(adjustment) = word.parse::<f32>() {
                items.push(ArrayToken::Adjustment(adjustment));
            }
        }
    }

    fn push_nested_open(&mut self, bytes: &mut Vec<u8>, depth: &mut i32) {
        *depth += 1;
        self.push_raw_byte(bytes, b'(');
    }

    fn push_nested_close(&mut self, bytes: &mut Vec<u8>, depth: &mut i32) {
        *depth -= 1;
        if *depth > 0 {
            bytes.push(b')');
        }
        self.index += 1;
    }

    fn push_raw_byte(&mut self, bytes: &mut Vec<u8>, byte: u8) {
        bytes.push(byte);
        self.index += 1;
    }

    fn push_escape(&mut self, bytes: &mut Vec<u8>) {
        self.index += 1;
        let Some(byte) = self.current() else {
            return;
        };

        match byte {
            b'n' => self.push_escaped_byte(bytes, b'\n'),
            b'r' => self.push_escaped_byte(bytes, b'\r'),
            b't' => self.push_escaped_byte(bytes, b'\t'),
            b'b' => self.push_escaped_byte(bytes, 0x08),
            b'f' => self.push_escaped_byte(bytes, 0x0c),
            b'\n' => self.index += 1,
            b'\r' => self.skip_cr_escape(),
            b'0'..=b'7' => self.push_octal_escape(bytes),
            other => self.push_escaped_byte(bytes, other),
        }
    }

    fn skip_cr_escape(&mut self) {
        self.index += 1;
        if self.current() == Some(b'\n') {
            self.index += 1;
        }
    }

    fn push_escaped_byte(&mut self, bytes: &mut Vec<u8>, byte: u8) {
        bytes.push(byte);
        self.index += 1;
    }

    fn push_octal_escape(&mut self, bytes: &mut Vec<u8>) {
        let start = self.index;
        let end = (start + 3).min(self.bytes.len());
        while self.index < end && matches!(self.current(), Some(b'0'..=b'7')) {
            self.index += 1;
        }

        let octal = String::from_utf8_lossy(&self.bytes[start..self.index]);
        if let Ok(value) = u8::from_str_radix(&octal, 8) {
            bytes.push(value);
        }
    }

    fn dictionary_actual_text(&mut self) -> Option<Vec<u8>> {
        self.index += 2;
        let mut depth = 1;
        let mut actual_text = None;
        while self.index + 1 < self.bytes.len() && depth > 0 {
            match (self.current(), self.peek()) {
                (Some(b'<'), Some(b'<')) => self.enter_dictionary(&mut depth),
                (Some(b'>'), Some(b'>')) => self.exit_dictionary(&mut depth),
                (Some(b'/'), _) if self.name_at_current() == "ActualText" => {
                    actual_text = self.read_actual_text_value();
                }
                _ => self.index += 1,
            }
        }
        actual_text
    }

    fn name_at_current(&self) -> String {
        let mut index = self.index + 1;
        while index < self.bytes.len() && !is_delimiter(self.bytes[index]) {
            index += 1;
        }
        String::from_utf8_lossy(&self.bytes[self.index + 1..index]).to_string()
    }

    fn read_actual_text_value(&mut self) -> Option<Vec<u8>> {
        self.name();
        self.skip_ignored();
        match self.current()? {
            b'(' => Some(self.literal_string()),
            b'<' if self.peek() != Some(b'<') => Some(self.hex_string()),
            _ => None,
        }
    }

    fn enter_dictionary(&mut self, depth: &mut i32) {
        *depth += 1;
        self.index += 2;
    }

    fn exit_dictionary(&mut self, depth: &mut i32) {
        *depth -= 1;
        self.index += 2;
    }

    fn skip_array_end(&mut self) {
        if self.current() == Some(b']') {
            self.index += 1;
        }
    }

    fn skip_hex_end(&mut self) {
        if self.current() == Some(b'>') {
            self.index += 1;
        }
    }

    fn skip_word(&mut self) {
        if matches!(self.current(), Some(byte) if is_delimiter(byte)) {
            self.index += 1;
            return;
        }

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
