use std::collections::{BTreeMap, HashMap};

use super::hex::decode_hex_bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum PdfValue {
    Null,
    Bool(bool),
    Integer(i64),
    Real(f32),
    Name(String),
    String(Vec<u8>),
    Array(Vec<PdfValue>),
    Dictionary(PdfDictionary),
    Reference(PdfReference),
}

pub type PdfDictionary = BTreeMap<String, PdfValue>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PdfReference {
    pub object: u32,
    pub generation: u16,
}

#[derive(Debug, Clone)]
pub struct PdfObject {
    pub reference: PdfReference,
    pub value: PdfValue,
    pub stream: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct PdfObjects {
    objects: HashMap<PdfReference, PdfObject>,
    latest_generations: HashMap<u32, u16>,
    order: Vec<PdfReference>,
}

impl PdfObjects {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut objects = Self::default();
        let mut cursor = 0;

        while let Some(header) = find_indirect_object(bytes, cursor) {
            let Some(end) = find_token(bytes, b"endobj", header.body_start) else {
                break;
            };
            let body = &bytes[header.body_start..end];
            if let Some(object) = parse_indirect_object(header.reference, body) {
                objects.insert(object);
            }
            cursor = end + b"endobj".len();
        }

        objects
    }

    pub fn get(&self, reference: PdfReference) -> Option<&PdfObject> {
        self.objects.get(&reference)
    }

    pub fn latest(&self, object: u32) -> Option<&PdfObject> {
        let generation = self.latest_generations.get(&object).copied()?;
        self.get(PdfReference { object, generation })
    }

    pub fn values(&self) -> impl Iterator<Item = &PdfObject> {
        self.order
            .iter()
            .filter_map(|reference| self.objects.get(reference))
    }

    fn insert(&mut self, object: PdfObject) {
        let reference = object.reference;
        self.latest_generations
            .entry(reference.object)
            .and_modify(|generation| *generation = (*generation).max(reference.generation))
            .or_insert(reference.generation);
        self.order.retain(|existing| *existing != reference);
        self.order.push(reference);
        self.objects.insert(reference, object);
    }
}

impl PdfObject {
    pub fn dictionary(&self) -> Option<&PdfDictionary> {
        match &self.value {
            PdfValue::Dictionary(dictionary) => Some(dictionary),
            _ => None,
        }
    }

    pub fn type_name(&self) -> Option<&str> {
        self.dictionary()
            .and_then(|dictionary| dictionary.name("Type"))
    }
}

pub trait PdfDictionaryExt {
    fn get_ref(&self, key: &str) -> Option<PdfReference>;
    fn array(&self, key: &str) -> Option<&[PdfValue]>;
    fn integer(&self, key: &str) -> Option<i64>;
    fn name(&self, key: &str) -> Option<&str>;
    fn string_bytes(&self, key: &str) -> Option<&[u8]>;
}

impl PdfDictionaryExt for PdfDictionary {
    fn get_ref(&self, key: &str) -> Option<PdfReference> {
        match self.get(key)? {
            PdfValue::Reference(reference) => Some(*reference),
            _ => None,
        }
    }

    fn array(&self, key: &str) -> Option<&[PdfValue]> {
        match self.get(key)? {
            PdfValue::Array(values) => Some(values),
            _ => None,
        }
    }

    fn integer(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            PdfValue::Integer(value) => Some(*value),
            _ => None,
        }
    }

    fn name(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            PdfValue::Name(value) => Some(value),
            _ => None,
        }
    }

    fn string_bytes(&self, key: &str) -> Option<&[u8]> {
        match self.get(key)? {
            PdfValue::String(bytes) => Some(bytes),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct ObjectHeader {
    reference: PdfReference,
    body_start: usize,
}

fn find_indirect_object(bytes: &[u8], from: usize) -> Option<ObjectHeader> {
    let mut lexer = Lexer::new(&bytes[from..]);
    let mut previous = Vec::new();

    while let Some(token) = lexer.next_token() {
        previous.push((token, from + lexer.last_start));
        if previous.len() > 3 {
            previous.remove(0);
        }

        let [(Token::Integer(object), _), (Token::Integer(generation), _), (Token::Word(word), _)] =
            previous.as_slice()
        else {
            continue;
        };

        if word == "obj" {
            return Some(ObjectHeader {
                reference: PdfReference {
                    object: u32::try_from(*object).ok()?,
                    generation: u16::try_from(*generation).ok()?,
                },
                body_start: from + lexer.index,
            });
        }
    }

    None
}

fn parse_indirect_object(reference: PdfReference, body: &[u8]) -> Option<PdfObject> {
    let stream_start = find_token(body, b"stream", 0);
    let value_bytes = stream_start.map_or(body, |start| &body[..start]);
    let value = Parser::new(value_bytes).parse_value()?;
    let stream = stream_start.and_then(|start| stream_bytes(body, start, &value));
    Some(PdfObject {
        reference,
        value,
        stream,
    })
}

fn stream_bytes(body: &[u8], stream_start: usize, value: &PdfValue) -> Option<Vec<u8>> {
    let data_start = skip_stream_line_end(body, stream_start + b"stream".len());
    let data_end = stream_length(value)
        .and_then(|length| data_start.checked_add(length))
        .filter(|end| *end <= body.len())
        .or_else(|| find_token(body, b"endstream", data_start));
    Some(trim_stream_suffix(&body[data_start..data_end?]).to_vec())
}

fn stream_length(value: &PdfValue) -> Option<usize> {
    match value {
        PdfValue::Dictionary(dictionary) => {
            let length = dictionary.integer("Length")?;
            usize::try_from(length).ok()
        }
        _ => None,
    }
}

fn skip_stream_line_end(bytes: &[u8], index: usize) -> usize {
    match bytes.get(index..index + 2) {
        Some(b"\r\n") => index + 2,
        _ if bytes.get(index) == Some(&b'\n') || bytes.get(index) == Some(&b'\r') => index + 1,
        _ => index,
    }
}

fn trim_stream_suffix(data: &[u8]) -> &[u8] {
    data.strip_suffix(b"\r\n")
        .or_else(|| data.strip_suffix(b"\n"))
        .unwrap_or(data)
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Vec<Token>,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            lexer: Lexer::new(bytes),
            peeked: Vec::new(),
        }
    }

    fn parse_value(&mut self) -> Option<PdfValue> {
        match self.next()? {
            Token::Null => Some(PdfValue::Null),
            Token::Bool(value) => Some(PdfValue::Bool(value)),
            Token::Integer(value) => self.number_or_reference(value),
            Token::Real(value) => Some(PdfValue::Real(value)),
            Token::Name(name) => Some(PdfValue::Name(name)),
            Token::String(bytes) | Token::HexString(bytes) => Some(PdfValue::String(bytes)),
            Token::ArrayStart => Some(PdfValue::Array(self.array())),
            Token::DictStart => Some(PdfValue::Dictionary(self.dictionary())),
            Token::Word(_) | Token::ArrayEnd | Token::DictEnd => None,
        }
    }

    fn number_or_reference(&mut self, object: i64) -> Option<PdfValue> {
        let generation = match self.next() {
            Some(Token::Integer(generation)) => generation,
            Some(token) => {
                self.push_back(token);
                return Some(PdfValue::Integer(object));
            }
            None => return Some(PdfValue::Integer(object)),
        };

        match self.next() {
            Some(Token::Word(word)) if word == "R" => Some(PdfValue::Reference(PdfReference {
                object: u32::try_from(object).ok()?,
                generation: u16::try_from(generation).ok()?,
            })),
            Some(token) => {
                self.push_back(token);
                self.push_back(Token::Integer(generation));
                Some(PdfValue::Integer(object))
            }
            None => {
                self.push_back(Token::Integer(generation));
                Some(PdfValue::Integer(object))
            }
        }
    }

    fn array(&mut self) -> Vec<PdfValue> {
        let mut values = Vec::new();
        while let Some(token) = self.next() {
            if token == Token::ArrayEnd {
                break;
            }
            self.push_back(token);
            if let Some(value) = self.parse_value() {
                values.push(value);
            }
        }
        values
    }

    fn dictionary(&mut self) -> PdfDictionary {
        let mut dictionary = PdfDictionary::new();
        while let Some(token) = self.next() {
            if token == Token::DictEnd {
                break;
            }
            let Token::Name(key) = token else {
                continue;
            };
            if let Some(value) = self.parse_value() {
                dictionary.insert(key, value);
            }
        }
        dictionary
    }

    fn next(&mut self) -> Option<Token> {
        self.peeked.pop().or_else(|| self.lexer.next_token())
    }

    fn push_back(&mut self, token: Token) {
        self.peeked.push(token);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
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

struct Lexer<'a> {
    bytes: &'a [u8],
    index: usize,
    last_start: usize,
}

impl<'a> Lexer<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            index: 0,
            last_start: 0,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
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

fn find_token(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_strings_hex_names_and_references() {
        let pdf = br#"
1 2 obj
<< /Escaped#20Name (a \(nested\) value) /Hex <4869> /Child 9 4 R >>
endobj
"#;

        let objects = PdfObjects::parse(pdf);
        let object = objects
            .get(PdfReference {
                object: 1,
                generation: 2,
            })
            .unwrap();
        let dictionary = object.dictionary().unwrap();

        assert!(dictionary.contains_key("Escaped Name"));
        assert_eq!(dictionary.get_ref("Child").unwrap().generation, 4);
        assert_eq!(
            dictionary.get("Hex"),
            Some(&PdfValue::String(b"Hi".to_vec()))
        );
    }

    #[test]
    fn keeps_latest_incremental_object_revision() {
        let pdf = br#"
1 0 obj << /Name /Old >> endobj
1 0 obj << /Name /New >> endobj
"#;

        let objects = PdfObjects::parse(pdf);

        assert_eq!(
            objects
                .latest(1)
                .unwrap()
                .dictionary()
                .unwrap()
                .name("Name"),
            Some("New")
        );
    }
}
