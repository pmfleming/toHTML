use crate::doc::{Block, Doc, Image, InlineStyle, StyledChar, Table, TableCell, TableRow};

pub fn parse_html(input: &str) -> Doc {
    let body = extract_body(input);
    let article = extract_pdf_article(body)
        .or_else(|| extract_first_tag(body, "article").map(|s| s.to_string()))
        .unwrap_or_else(|| body.to_string());
    let mut blocks = parse_blocks(&article);
    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Vec::new()));
    }
    Doc { blocks }
}

fn extract_body(s: &str) -> &str {
    if let Some(start) = find_ascii_ci(s, "<body") {
        if let Some(open_end) = s[start..].find('>') {
            let body_start = start + open_end + 1;
            if let Some(body_end) = find_ascii_ci(&s[body_start..], "</body>") {
                return &s[body_start..body_start + body_end];
            }
            return &s[body_start..];
        }
    }
    s
}

fn extract_pdf_article(body: &str) -> Option<String> {
    let details_start = find_ascii_ci(body, "pdf-extracted-content")?;
    let before = rfind_ascii_ci(&body[..details_start], "<details")?;
    let after = find_ascii_ci(&body[before..], "</details>").map(|i| before + i)?;
    extract_first_tag(&body[before..after], "article").map(|s| s.to_string())
}

fn parse_blocks(html: &str) -> Vec<Block> {
    let bytes = html.as_bytes();
    let mut blocks = Vec::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        if bytes[pos] != b'<' {
            let next_lt = html[pos..]
                .find('<')
                .map(|n| pos + n)
                .unwrap_or(bytes.len());
            let text = html[pos..next_lt].trim();
            if !text.is_empty() {
                blocks.push(Block::Paragraph(parse_inline(text)));
            }
            pos = next_lt;
            continue;
        }
        let Some(end) = html[pos..].find('>') else {
            break;
        };
        let raw_tag = &html[pos + 1..pos + end];
        let tag_name = tag_name(raw_tag);
        let after_tag = pos + end + 1;
        match tag_name.as_str() {
            "header" | "summary" => {
                pos = skip_tag(html, after_tag, &tag_name).unwrap_or(after_tag);
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name[1..].parse::<u8>().unwrap_or(1);
                if let Some((inner, next)) = inner_and_next(html, after_tag, &tag_name) {
                    blocks.push(Block::Heading(level, parse_inline(inner)));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "p" => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "p") {
                    blocks.push(Block::Paragraph(parse_inline(inner)));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "blockquote" => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "blockquote") {
                    let nested = parse_blocks(inner);
                    if nested.len() == 1 {
                        blocks.push(nested.into_iter().next().unwrap());
                    } else {
                        blocks.push(Block::Blockquote(parse_inline(&strip_tags(inner))));
                    }
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "pre" => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "pre") {
                    blocks.push(Block::Pre(parse_inline(&strip_tags(inner))));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "ul" | "ol" => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, &tag_name) {
                    for item in extract_tag_items(inner, "li") {
                        let runs = parse_inline(&strip_block_wrappers(&item));
                        if tag_name == "ul" {
                            blocks.push(Block::Bullet(runs));
                        } else {
                            blocks.push(Block::Numbered(runs));
                        }
                    }
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "table" => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "table") {
                    blocks.push(Block::Table(parse_table(inner)));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "img" => {
                blocks.push(Block::Image(parse_image(raw_tag)));
                pos = after_tag;
            }
            "hr" => {
                if raw_tag.to_lowercase().contains("data-page-break") {
                    blocks.push(Block::PageBreak(
                        attr(raw_tag, "data-page").and_then(|v| v.parse().ok()),
                    ));
                } else {
                    blocks.push(Block::Hr);
                }
                pos = after_tag;
            }
            "div" if raw_tag.to_lowercase().contains("data-page-placeholder") => {
                blocks.push(Block::PagePlaceholder {
                    page: attr(raw_tag, "data-page").and_then(|v| v.parse().ok()),
                    reason: attr(raw_tag, "data-reason").unwrap_or_else(|| "empty".into()),
                });
                pos = skip_tag(html, after_tag, "div").unwrap_or(after_tag);
            }
            _ => {
                pos = after_tag;
            }
        }
    }
    blocks
}

fn parse_table(html: &str) -> Table {
    let caption = extract_first_tag(html, "caption").map(parse_inline);
    let rows = extract_tag_items(html, "tr")
        .into_iter()
        .map(|row_html| {
            let mut cells = Vec::new();
            cells.extend(extract_cells(&row_html, "th", true));
            cells.extend(extract_cells(&row_html, "td", false));
            TableRow { cells }
        })
        .filter(|row| !row.cells.is_empty())
        .collect();
    Table { caption, rows }
}

fn extract_cells(row_html: &str, tag: &str, header: bool) -> Vec<TableCell> {
    extract_tag_items_with_open(row_html, tag)
        .into_iter()
        .map(|(open, inner)| TableCell {
            header,
            colspan: attr(&open, "colspan")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            rowspan: attr(&open, "rowspan")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            align: attr(&open, "style").and_then(|s| {
                s.to_lowercase()
                    .split("text-align:")
                    .nth(1)
                    .map(|v| v.trim().trim_end_matches(';').to_string())
            }),
            content: parse_inline(&inner),
        })
        .collect()
}

fn parse_image(open_tag: &str) -> Image {
    Image {
        src: attr(open_tag, "src").unwrap_or_default(),
        alt: attr(open_tag, "alt").unwrap_or_default(),
        title: attr(open_tag, "title"),
        width: attr(open_tag, "width").and_then(|v| v.parse().ok()),
        height: attr(open_tag, "height").and_then(|v| v.parse().ok()),
    }
}

fn parse_inline(html: &str) -> Vec<StyledChar> {
    let mut out: Vec<StyledChar> = Vec::new();
    let mut style = InlineStyle::default();
    let mut style_stack: Vec<InlineStyle> = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'<' {
            let Some(end) = html[i..].find('>') else {
                push_char(&mut out, '<', &style);
                i += 1;
                continue;
            };
            let raw = &html[i + 1..i + end];
            i += end + 1;
            let (closing, inner) = raw
                .strip_prefix('/')
                .map(|s| (true, s))
                .unwrap_or((false, raw));
            let tag = tag_name(inner);
            if !closing {
                style_stack.push(style.clone());
                match tag.as_str() {
                    "strong" | "b" => style.bold = true,
                    "em" | "i" => style.italic = true,
                    "u" => style.underline = true,
                    "s" | "strike" | "del" => style.strike = true,
                    "code" => style.code = true,
                    "a" => {
                        if let Some(href) = attr(inner, "href") {
                            style.link = Some(href);
                        }
                    }
                    "br" => {
                        push_char(&mut out, '\n', &style);
                        style = style_stack.pop().unwrap_or_default();
                    }
                    "img" => {
                        let alt = attr(inner, "alt").unwrap_or_else(|| "[image]".into());
                        for ch in alt.chars() {
                            push_char(&mut out, ch, &style);
                        }
                        style = style_stack.pop().unwrap_or_default();
                    }
                    _ => {}
                }
            } else if let Some(previous) = style_stack.pop() {
                style = previous;
            }
        } else if b == b'&' {
            if let Some(end) = html[i..].find(';') {
                let entity = &html[i + 1..i + end];
                push_char(&mut out, decode_entity(entity), &style);
                i += end + 1;
            } else {
                push_char(&mut out, '&', &style);
                i += 1;
            }
        } else {
            let len = utf8_char_len(b).min(html.len() - i);
            if let Some(ch) = html[i..i + len].chars().next() {
                push_char(&mut out, ch, &style);
            }
            i += len;
        }
    }
    out
}

fn push_char(out: &mut Vec<StyledChar>, ch: char, style: &InlineStyle) {
    out.push(StyledChar {
        ch,
        style: style.clone(),
    });
}

fn decode_entity(entity: &str) -> char {
    match entity {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00A0}',
        e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16)
            .ok()
            .and_then(char::from_u32)
            .unwrap_or('?'),
        e if e.starts_with('#') => e[1..]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .unwrap_or('?'),
        _ => '?',
    }
}

fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else {
        4
    }
}

fn tag_name(tag: &str) -> String {
    tag.split_whitespace()
        .next()
        .unwrap_or("")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_lowercase()
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let needle = format!("{}=", name.to_lowercase());
    let start = lower.find(&needle)? + needle.len();
    let quote = tag.as_bytes().get(start).copied();
    if quote == Some(b'"') || quote == Some(b'\'') {
        let q = quote.unwrap() as char;
        let value_start = start + 1;
        let value_end = tag[value_start..].find(q)? + value_start;
        Some(unescape_attr(&tag[value_start..value_end]))
    } else {
        let value = tag[start..].split_whitespace().next().unwrap_or("");
        Some(unescape_attr(value.trim_end_matches('/')))
    }
}

fn unescape_attr(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn inner_and_next<'a>(html: &'a str, after_tag: usize, tag: &str) -> Option<(&'a str, usize)> {
    let close = format!("</{tag}>");
    let end = find_ascii_ci(&html[after_tag..], &close)? + after_tag;
    Some((&html[after_tag..end], end + close.len()))
}

fn skip_tag(html: &str, after_tag: usize, tag: &str) -> Option<usize> {
    let close = format!("</{tag}>");
    find_ascii_ci(&html[after_tag..], &close).map(|end| after_tag + end + close.len())
}

fn extract_first_tag<'a>(html: &'a str, tag: &str) -> Option<&'a str> {
    let open = find_ascii_ci(html, &format!("<{tag}"))?;
    let open_end = html[open..].find('>')? + open;
    let close = format!("</{tag}>");
    let end = find_ascii_ci(&html[open_end + 1..], &close)? + open_end + 1;
    Some(&html[open_end + 1..end])
}

fn extract_tag_items(html: &str, tag: &str) -> Vec<String> {
    extract_tag_items_with_open(html, tag)
        .into_iter()
        .map(|(_, inner)| inner)
        .collect()
}

fn extract_tag_items_with_open(html: &str, tag: &str) -> Vec<(String, String)> {
    let mut items = Vec::new();
    let mut pos = 0;
    while pos < html.len() {
        let Some(start_rel) = find_ascii_ci(&html[pos..], &format!("<{tag}")) else {
            break;
        };
        let start = pos + start_rel;
        let Some(open_end_rel) = html[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let inner_start = open_end + 1;
        let close = format!("</{tag}>");
        let Some(end_rel) = find_ascii_ci(&html[inner_start..], &close) else {
            break;
        };
        let end = inner_start + end_rel;
        items.push((
            html[start + 1..open_end].to_string(),
            html[inner_start..end].to_string(),
        ));
        pos = end + close.len();
    }
    items
}

fn find_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn rfind_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() {
        return Some(haystack.len());
    }
    haystack
        .windows(needle.len())
        .rposition(|window| window.eq_ignore_ascii_case(needle))
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn strip_block_wrappers(html: &str) -> String {
    if let Some(p) = extract_first_tag(html, "p") {
        p.to_string()
    } else {
        html.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::plain_chars;
    use crate::serialize::serialize_document;

    #[test]
    fn parses_tohtml_table() {
        let doc = parse_html(
            r#"<body><article><table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td><strong>2</strong></td></tr></table></article></body>"#,
        );
        assert!(matches!(doc.blocks.first(), Some(Block::Table(_))));
    }

    #[test]
    fn parses_pdf_extracted_article() {
        let doc = parse_html(
            r#"<body><main><section class="pdf-recreated-page">visual</section><details class="pdf-extracted-content"><summary>Extracted</summary><article><p>Hello</p></article></details></main></body>"#,
        );
        assert_eq!(doc.blocks.first().map(Block::text), Some("Hello".into()));
    }

    #[test]
    fn parses_unicode_text_without_byte_boundary_panic() {
        let doc = parse_html(
            r#"<body><article><h2>communication protocol</h2><ul><li>alpha • beta</li><li>gamma</li></ul></article></body>"#,
        );
        assert_eq!(
            doc.blocks.get(1).map(Block::text),
            Some("alpha • beta".into())
        );
    }

    #[test]
    fn serializes_table() {
        let doc = Doc {
            blocks: vec![Block::Table(Table {
                caption: None,
                rows: vec![TableRow {
                    cells: vec![TableCell {
                        header: true,
                        colspan: 1,
                        rowspan: 1,
                        align: None,
                        content: plain_chars("Name"),
                    }],
                }],
            })],
        };
        assert!(serialize_document(&doc).contains("<th>Name</th>"));
    }
}
