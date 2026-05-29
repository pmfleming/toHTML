use super::lexer::{Lexer, Token};
use super::{PdfDictionary, PdfReference, PdfValue};

pub(super) fn parse_value(bytes: &[u8]) -> Option<PdfValue> {
    Parser::new(bytes).parse_value()
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
