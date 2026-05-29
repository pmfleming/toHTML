use super::{is_delimiter, MarkedProps, Reader};

impl<'a> Reader<'a> {
    pub(super) fn dictionary_marked_props(&mut self) -> MarkedProps {
        self.index += 2;
        let mut depth = 1;
        let mut props = MarkedProps::default();
        while self.index + 1 < self.bytes.len() && depth > 0 {
            match (self.current(), self.peek()) {
                (Some(b'<'), Some(b'<')) => self.enter_dictionary(&mut depth),
                (Some(b'>'), Some(b'>')) => self.exit_dictionary(&mut depth),
                (Some(b'/'), _) => match self.name_at_current().as_str() {
                    "ActualText" => props.actual_text = self.read_actual_text_value(),
                    "MCID" => props.mcid = self.read_mcid_value(),
                    _ => self.index += 1,
                },
                _ => self.index += 1,
            }
        }
        props
    }

    fn read_mcid_value(&mut self) -> Option<u32> {
        self.name();
        self.skip_ignored();
        let start = self.index;
        while matches!(self.current(), Some(byte) if byte.is_ascii_digit()) {
            self.index += 1;
        }
        let digits = &self.bytes[start..self.index];
        if digits.is_empty() {
            return None;
        }
        String::from_utf8_lossy(digits).parse().ok()
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
}
