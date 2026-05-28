use std::collections::{BTreeMap, HashMap};
use std::io::Read;

use flate2::read::ZlibDecoder;

mod lexer;

use lexer::{Lexer, Token};

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

        objects.expand_object_streams();

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

    fn expand_object_streams(&mut self) {
        let streams = self
            .order
            .iter()
            .filter_map(|reference| self.objects.get(reference))
            .filter(|object| object.type_name() == Some("ObjStm"))
            .filter_map(|object| {
                let dictionary = object.dictionary()?;
                let stream = object.stream.as_ref()?;
                Some((
                    object.reference,
                    dictionary.integer("N")?,
                    dictionary.integer("First")?,
                    stream_filters(dictionary),
                    stream.clone(),
                ))
            })
            .collect::<Vec<_>>();

        for (reference, count, first, filters, stream) in streams {
            let Some(decoded) = decode_object_stream(&filters, &stream) else {
                continue;
            };
            let Some(objects) = parse_object_stream_objects(&decoded, count, first) else {
                continue;
            };
            for (object_number, body) in objects {
                if let Some(value) = Parser::new(body).parse_value() {
                    self.insert(PdfObject {
                        reference: PdfReference {
                            object: object_number,
                            generation: 0,
                        },
                        value,
                        stream: None,
                    });
                }
            }

            if let Some(object) = self.objects.get_mut(&reference) {
                object.stream = Some(decoded);
            }
        }
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

fn stream_filters(dictionary: &PdfDictionary) -> Vec<String> {
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

fn decode_object_stream(filters: &[String], data: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = data.to_vec();
    for filter in filters {
        decoded = match filter.as_str() {
            "FlateDecode" | "Fl" => {
                let mut decoder = ZlibDecoder::new(decoded.as_slice());
                let mut output = Vec::new();
                decoder.read_to_end(&mut output).ok()?;
                output
            }
            _ => return None,
        };
    }
    Some(decoded)
}

fn parse_object_stream_objects(
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

fn find_token(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Write;

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

    #[test]
    fn expands_flate_decoded_object_stream_entries() {
        let first_body = b"<< /Name /FromStream >>";
        let second_body = b"[1 2 3]";
        let header = format!("8 0 9 {} ", first_body.len());
        let mut body = header.into_bytes();
        body.extend_from_slice(first_body);
        body.extend_from_slice(second_body);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&body).unwrap();
        let compressed = encoder.finish().unwrap();
        let first = body.len() - first_body.len() - second_body.len();
        let mut pdf =
            format!("7 0 obj << /Type /ObjStm /N 2 /First {first} /Filter /FlateDecode /Length ")
                .into_bytes();
        pdf.extend_from_slice(compressed.len().to_string().as_bytes());
        pdf.extend_from_slice(b" >>\nstream\n");
        pdf.extend_from_slice(&compressed);
        pdf.extend_from_slice(b"\nendstream\nendobj");

        let objects = PdfObjects::parse(&pdf);

        assert_eq!(
            objects
                .latest(8)
                .unwrap()
                .dictionary()
                .unwrap()
                .name("Name"),
            Some("FromStream")
        );
        assert!(matches!(
            objects.latest(9).unwrap().value,
            PdfValue::Array(_)
        ));
    }
}
