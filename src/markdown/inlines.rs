use crate::{Image, Inline, Link};

pub fn parse_inlines(text: &str) -> Vec<Inline> {
    let mut parser = InlineParser { text, index: 0 };
    parser.parse_until_end()
}

struct InlineParser<'a> {
    text: &'a str,
    index: usize,
}

impl InlineParser<'_> {
    fn parse_until_end(&mut self) -> Vec<Inline> {
        let mut inlines = Vec::new();
        while self.index < self.text.len() {
            if let Some(inline) = self.parse_special() {
                inlines.push(inline);
            } else {
                inlines.push(Inline::Text(self.take_text()));
            }
        }
        inlines
    }

    fn parse_special(&mut self) -> Option<Inline> {
        self.parse_code()
            .or_else(|| self.parse_image())
            .or_else(|| self.parse_link())
            .or_else(|| self.parse_delimited("~~", Inline::Strikethrough))
            .or_else(|| self.parse_delimited("**", Inline::Strong))
            .or_else(|| self.parse_delimited("__", Inline::Strong))
            .or_else(|| self.parse_delimited("*", Inline::Emphasis))
            .or_else(|| self.parse_delimited("_", Inline::Emphasis))
            .or_else(|| self.parse_line_break())
    }

    fn parse_code(&mut self) -> Option<Inline> {
        let rest = self.rest();
        if !rest.starts_with('`') {
            return None;
        }
        let end = rest[1..].find('`')? + 1;
        let code = rest[1..end].to_string();
        self.index += end + 1;
        Some(Inline::Code(code))
    }

    fn parse_image(&mut self) -> Option<Inline> {
        let rest = self.rest();
        if !rest.starts_with("![") {
            return None;
        }
        let (alt, target, consumed) = parse_bracket_target(&rest[1..])?;
        self.index += consumed + 1;
        Some(Inline::Image(Image {
            src: target.href,
            alt: Some(alt),
            title: target.title,
            width: None,
            height: None,
            asset_id: None,
            source: None,
        }))
    }

    fn parse_link(&mut self) -> Option<Inline> {
        let rest = self.rest();
        if !rest.starts_with('[') {
            return None;
        }
        let (label, target, consumed) = parse_bracket_target(rest)?;
        self.index += consumed;
        Some(Inline::Link(Link {
            href: target.href,
            title: target.title,
            content: parse_inlines(&label),
            source: None,
        }))
    }

    fn parse_delimited(
        &mut self,
        delimiter: &str,
        wrap: fn(Vec<Inline>) -> Inline,
    ) -> Option<Inline> {
        let rest = self.rest();
        if !rest.starts_with(delimiter) {
            return None;
        }
        let content_start = delimiter.len();
        let end = rest[content_start..].find(delimiter)? + content_start;
        let content = parse_inlines(&rest[content_start..end]);
        self.index += end + delimiter.len();
        Some(wrap(content))
    }

    fn parse_line_break(&mut self) -> Option<Inline> {
        if self.rest().starts_with("\\\n") {
            self.index += 2;
            return Some(Inline::LineBreak);
        }
        None
    }

    fn take_text(&mut self) -> String {
        let start = self.index;
        while self.index < self.text.len() && !special_start(self.rest()) {
            self.index += self.next_char_len();
        }
        self.text[start..self.index].to_string()
    }

    fn rest(&self) -> &str {
        &self.text[self.index..]
    }

    fn next_char_len(&self) -> usize {
        self.rest().chars().next().map(char::len_utf8).unwrap_or(1)
    }
}

struct LinkTarget {
    href: String,
    title: Option<String>,
}

fn parse_bracket_target(text: &str) -> Option<(String, LinkTarget, usize)> {
    let label_end = text.find(']')?;
    let after_label = text.get(label_end + 1..)?;
    if !after_label.starts_with('(') {
        return None;
    }
    let target_end = after_label.find(')')?;
    let target = parse_target(after_label[1..target_end].trim());
    Some((
        text[1..label_end].to_string(),
        target,
        label_end + target_end + 2,
    ))
}

fn parse_target(text: &str) -> LinkTarget {
    if let Some(title_start) = text.find(" \"") {
        return LinkTarget {
            href: text[..title_start].to_string(),
            title: Some(text[title_start + 2..].trim_end_matches('"').to_string()),
        };
    }
    LinkTarget {
        href: text.to_string(),
        title: None,
    }
}

fn special_start(text: &str) -> bool {
    ["`", "![", "[", "~~", "**", "__", "*", "_", "\\\n"]
        .iter()
        .any(|prefix| text.starts_with(prefix))
}
