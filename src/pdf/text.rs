pub fn extract_text(stream: &[u8]) -> Option<String> {
    let source = String::from_utf8_lossy(stream);
    let mut parser = TextParser {
        chars: source.chars().collect(),
        index: 0,
        output: String::new(),
    };
    parser.parse();
    let text = normalize_whitespace(&parser.output);
    (!text.is_empty()).then_some(text)
}

struct TextParser {
    chars: Vec<char>,
    index: usize,
    output: String,
}

impl TextParser {
    fn parse(&mut self) {
        while self.index < self.chars.len() {
            match self.current() {
                Some('(') => self.parse_literal_string(),
                Some('<') if self.peek() == Some('<') => self.skip_dictionary(),
                Some('<') => self.parse_hex_string(),
                _ => self.index += 1,
            }
        }
    }

    fn parse_literal_string(&mut self) {
        self.index += 1;
        let mut depth = 1;
        while self.index < self.chars.len() && depth > 0 {
            match self.current() {
                Some('\\') => self.parse_escape(),
                Some('(') => self.push_nested_open(&mut depth),
                Some(')') => self.push_nested_close(&mut depth),
                Some(ch) => self.push_char(ch),
                None => break,
            }
        }
        self.output.push(' ');
    }

    fn push_nested_open(&mut self, depth: &mut i32) {
        *depth += 1;
        self.push_char('(');
    }

    fn push_nested_close(&mut self, depth: &mut i32) {
        *depth -= 1;
        if *depth > 0 {
            self.output.push(')');
        }
        self.index += 1;
    }

    fn push_char(&mut self, ch: char) {
        self.output.push(ch);
        self.index += 1;
    }

    fn parse_escape(&mut self) {
        self.index += 1;
        let Some(ch) = self.current() else {
            return;
        };
        self.output.push(match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'b' => '\u{0008}',
            'f' => '\u{000c}',
            other => other,
        });
        self.index += 1;
    }

    fn parse_hex_string(&mut self) {
        self.index += 1;
        let start = self.index;
        while self.index < self.chars.len() && self.current() != Some('>') {
            self.index += 1;
        }
        let hex: String = self.chars[start..self.index]
            .iter()
            .filter(|ch| !ch.is_whitespace())
            .collect();
        self.output.push_str(&decode_hex_ascii(&hex));
        self.output.push(' ');
        if self.current() == Some('>') {
            self.index += 1;
        }
    }

    fn skip_dictionary(&mut self) {
        self.index += 2;
        while self.index + 1 < self.chars.len() {
            if self.current() == Some('>') && self.peek() == Some('>') {
                self.index += 2;
                break;
            }
            self.index += 1;
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }
}

fn decode_hex_ascii(hex: &str) -> String {
    hex.as_bytes()
        .chunks(2)
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .filter_map(|byte| u8::from_str_radix(byte, 16).ok())
        .filter(|byte| byte.is_ascii_graphic() || byte.is_ascii_whitespace())
        .map(char::from)
        .collect()
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
