use crate::doc::{
    chars_to_string, Block, Doc, Image, InlineStyle, PdfBox, PdfElement, PdfImageElement, PdfInk,
    PdfLinkOverlay, PdfPage, PdfPath, PdfShape, PdfTextFragment, StyledChar, Table, TableCell,
    TableRow,
};

pub fn parse_html(input: &str) -> Doc {
    let body = extract_body(input);
    let mut blocks = parse_pdf_pages(body)
        .into_iter()
        .map(Block::PdfPage)
        .collect::<Vec<_>>();

    if let Some(article) = extract_pdf_article(body)
        .or_else(|| extract_first_tag(body, "article").map(|s| s.to_string()))
    {
        blocks.extend(parse_blocks(&article));
    } else if blocks.is_empty() {
        blocks = parse_blocks(body);
    }

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

fn parse_pdf_pages(body: &str) -> Vec<PdfPage> {
    extract_tag_items_with_open(body, "section")
        .into_iter()
        .filter(|(open, _)| {
            attr(open, "class")
                .map(|class| class.contains("pdf-recreated-page"))
                .unwrap_or(false)
        })
        .map(|(open, inner)| parse_pdf_page(&open, &inner))
        .collect()
}

fn parse_pdf_page(open: &str, inner: &str) -> PdfPage {
    let style = attr(open, "style").unwrap_or_default();
    PdfPage {
        page: attr(open, "data-page").and_then(|v| v.parse().ok()),
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-recreated-page".into()),
        width_pt: style_number(&style, "width"),
        height_pt: style_number(&style, "height"),
        style,
        elements: parse_pdf_elements(inner),
    }
}

fn parse_pdf_elements(html: &str) -> Vec<PdfElement> {
    let bytes = html.as_bytes();
    let mut elements = Vec::new();
    let mut pos = 0usize;
    while pos < bytes.len() {
        let Some(start_rel) = html[pos..].find('<') else {
            break;
        };
        let start = pos + start_rel;
        let Some(end_rel) = html[start..].find('>') else {
            break;
        };
        let open_end = start + end_rel;
        let raw_tag = &html[start + 1..open_end];
        let tag = tag_name(raw_tag);
        let after_tag = open_end + 1;

        match tag.as_str() {
            "span" if has_class(raw_tag, "pdf-text-fragment") => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "span") {
                    elements.push(PdfElement::Text(parse_pdf_text(raw_tag, inner)));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "img" if has_class(raw_tag, "pdf-image") => {
                elements.push(PdfElement::Image(parse_pdf_image(raw_tag)));
                pos = after_tag;
            }
            "div" if has_class(raw_tag, "pdf-shape") => {
                elements.push(PdfElement::Shape(parse_pdf_shape(raw_tag)));
                pos = skip_tag(html, after_tag, "div").unwrap_or(after_tag);
            }
            "svg" if has_class(raw_tag, "pdf-ink") => {
                if let Some((inner, next)) = inner_and_next(html, after_tag, "svg") {
                    elements.push(PdfElement::Ink(parse_pdf_ink(raw_tag, inner)));
                    pos = next;
                } else {
                    pos = after_tag;
                }
            }
            "a" if has_class(raw_tag, "pdf-link-overlay") => {
                elements.push(PdfElement::Link(parse_pdf_link(raw_tag)));
                pos = skip_tag(html, after_tag, "a").unwrap_or(after_tag);
            }
            _ => {
                pos = after_tag;
            }
        }
    }
    elements
}

fn parse_pdf_text(open: &str, inner: &str) -> PdfTextFragment {
    let style = attr(open, "style").unwrap_or_default();
    let transform = style_property(&style, "transform");
    PdfTextFragment {
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-text-fragment".into()),
        bounds: parse_pdf_box(&style),
        font_size_pt: style_number(&style, "font-size"),
        font_weight: style_property(&style, "font-weight").and_then(|v| v.parse().ok()),
        font_family: style_property(&style, "font-family"),
        font_style: style_property(&style, "font-style"),
        color: style_property(&style, "color"),
        rotation_deg: transform
            .as_deref()
            .and_then(|value| transform_number(value, "rotate")),
        scale_x: transform
            .as_deref()
            .and_then(|value| transform_number(value, "scaleX")),
        transform,
        style,
        text: chars_to_string(&parse_inline(inner)),
    }
}

fn parse_pdf_image(open: &str) -> PdfImageElement {
    let style = attr(open, "style").unwrap_or_default();
    PdfImageElement {
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-image".into()),
        bounds: parse_pdf_box(&style),
        src: attr(open, "src").unwrap_or_default(),
        alt: attr(open, "alt").unwrap_or_default(),
        style,
    }
}

fn parse_pdf_shape(open: &str) -> PdfShape {
    let style = attr(open, "style").unwrap_or_default();
    let border = style_property(&style, "border");
    PdfShape {
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-shape".into()),
        bounds: parse_pdf_box(&style),
        background: style_property(&style, "background"),
        border_width_pt: border.as_deref().and_then(border_width),
        border_color: border.as_deref().and_then(border_color),
        border,
        style,
    }
}

fn parse_pdf_ink(open: &str, inner: &str) -> PdfInk {
    let style = attr(open, "style").unwrap_or_default();
    PdfInk {
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-ink".into()),
        bounds: parse_pdf_box(&style),
        view_box: attr(open, "viewBox").or_else(|| attr(open, "viewbox")),
        paths: parse_pdf_paths(inner),
        style,
    }
}

fn parse_pdf_paths(inner: &str) -> Vec<PdfPath> {
    let mut paths = Vec::new();
    let mut pos = 0usize;
    while pos < inner.len() {
        let Some(start_rel) = find_ascii_ci(&inner[pos..], "<path") else {
            break;
        };
        let start = pos + start_rel;
        let Some(end_rel) = inner[start..].find('>') else {
            break;
        };
        let open = &inner[start + 1..start + end_rel];
        paths.push(PdfPath {
            d: attr(open, "d").unwrap_or_default(),
            fill: attr(open, "fill"),
            stroke: attr(open, "stroke"),
            stroke_width: attr(open, "stroke-width"),
        });
        pos = start + end_rel + 1;
    }
    paths
}

fn parse_pdf_link(open: &str) -> PdfLinkOverlay {
    let style = attr(open, "style").unwrap_or_default();
    PdfLinkOverlay {
        class_name: attr(open, "class").unwrap_or_else(|| "pdf-link-overlay".into()),
        bounds: parse_pdf_box(&style),
        href: attr(open, "href").unwrap_or_default(),
        label: attr(open, "aria-label").or_else(|| attr(open, "title")),
        style,
    }
}

fn parse_pdf_box(style: &str) -> PdfBox {
    PdfBox {
        left_pt: style_number(style, "left"),
        top_pt: style_number(style, "top"),
        width_pt: style_number(style, "width"),
        height_pt: style_number(style, "height"),
    }
}

fn has_class(open: &str, class_name: &str) -> bool {
    attr(open, "class")
        .map(|class| class.split_whitespace().any(|item| item == class_name))
        .unwrap_or(false)
}

fn style_number(style: &str, name: &str) -> Option<f32> {
    let value = style_property(style, name)?;
    css_number(&value)
}

fn css_number(value: &str) -> Option<f32> {
    value
        .trim()
        .trim_end_matches("deg")
        .trim_end_matches("pt")
        .trim_end_matches("px")
        .trim_end_matches('%')
        .parse()
        .ok()
}

fn transform_number(transform: &str, name: &str) -> Option<f32> {
    let transform_lower = transform.to_ascii_lowercase();
    let name_lower = name.to_ascii_lowercase();
    let name_start = transform_lower.find(&name_lower)?;
    let after_name = name_start + name_lower.len();
    let open = transform_lower[after_name..].find('(')? + after_name;
    let close = transform_lower[open + 1..].find(')')? + open + 1;
    css_number(&transform[open + 1..close])
}

fn border_width(border: &str) -> Option<f32> {
    border.split_whitespace().find_map(css_number)
}

fn border_color(border: &str) -> Option<String> {
    border
        .split_whitespace()
        .find(|token| token.starts_with('#') && token.len() >= 4)
        .map(str::to_string)
}

fn style_property(style: &str, name: &str) -> Option<String> {
    style.split(';').find_map(|decl| {
        let (key, value) = decl.split_once(':')?;
        if key.trim().eq_ignore_ascii_case(name) {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
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
        .map(|row_html| TableRow {
            cells: extract_row_cells(&row_html),
        })
        .filter(|row| !row.cells.is_empty())
        .collect();
    Table { caption, rows }
}

fn extract_row_cells(row_html: &str) -> Vec<TableCell> {
    let mut cells = Vec::new();
    let mut pos = 0usize;
    while pos < row_html.len() {
        let next_th = find_ascii_ci(&row_html[pos..], "<th").map(|idx| (pos + idx, "th", true));
        let next_td = find_ascii_ci(&row_html[pos..], "<td").map(|idx| (pos + idx, "td", false));
        let Some((start, tag, header)) = earliest_cell(next_th, next_td) else {
            break;
        };
        let Some(open_end_rel) = row_html[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let inner_start = open_end + 1;
        let close = format!("</{tag}>");
        let Some(end_rel) = find_ascii_ci(&row_html[inner_start..], &close) else {
            break;
        };
        let end = inner_start + end_rel;
        cells.push(parse_table_cell(
            &row_html[start + 1..open_end],
            &row_html[inner_start..end],
            header,
        ));
        pos = end + close.len();
    }
    cells
}

fn earliest_cell(
    left: Option<(usize, &'static str, bool)>,
    right: Option<(usize, &'static str, bool)>,
) -> Option<(usize, &'static str, bool)> {
    match (left, right) {
        (Some(l), Some(r)) if l.0 <= r.0 => Some(l),
        (Some(_), Some(r)) => Some(r),
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (None, None) => None,
    }
}

fn parse_table_cell(open: &str, inner: &str, header: bool) -> TableCell {
    TableCell {
        header,
        colspan: attr(open, "colspan")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        rowspan: attr(open, "rowspan")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        align: attr(open, "style").and_then(|s| {
            s.to_lowercase()
                .split("text-align:")
                .nth(1)
                .map(|v| v.trim().trim_end_matches(';').to_string())
        }),
        content: parse_inline(inner),
    }
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
        assert!(matches!(doc.blocks.first(), Some(Block::PdfPage(_))));
        assert_eq!(doc.blocks.get(1).map(Block::text), Some("Hello".into()));
    }

    #[test]
    fn parses_pdf_visual_page_subset() {
        let doc = parse_html(
            r##"<body><main class="pdf-reconstructed-document">
              <section class="pdf-recreated-page pdf-prose-page" data-page="2" style="width:596.00pt;height:842.00pt">
                <div class="pdf-shape" style="left:10.00pt;top:20.00pt;width:30.00pt;height:4.00pt;background:#000000;border:0.75pt solid #c00000"></div>
                <span class="pdf-text-fragment pdf-rotated-text" style="left:42.00pt;top:50.00pt;font-size:11.04pt;width:70.00pt;height:12.00pt;font-family:Courier New, Courier, monospace;font-weight:700;font-style:italic;color:#365f91;transform:rotate(90.00deg) scaleX(0.68);transform-origin:left top">Hello &amp; PDF</span>
                <img class="pdf-image" src="data:image/png;base64,abc" alt="Logo" style="left:1.00pt;top:2.00pt;width:3.00pt;height:4.00pt">
                <svg class="pdf-ink" style="left:0;top:0;width:10.00pt;height:10.00pt" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0L10 10" fill="#dedede"/></svg>
                <a class="pdf-link-overlay" href="https://example.com" aria-label="Example" style="left:5.00pt;top:6.00pt;width:7.00pt;height:8.00pt"></a>
              </section>
              <details class="pdf-extracted-content"><summary>Extracted</summary><article><p>Hello</p></article></details>
            </main></body>"##,
        );

        let Some(Block::PdfPage(page)) = doc.blocks.first() else {
            panic!("expected parsed PDF page");
        };
        assert_eq!(page.page, Some(2));
        assert_eq!(page.width_pt, Some(596.0));
        assert_eq!(page.height_pt, Some(842.0));
        assert_eq!(page.elements.len(), 5);
        assert!(matches!(page.elements[0], PdfElement::Shape(_)));
        assert!(matches!(page.elements[1], PdfElement::Text(_)));
        assert!(matches!(page.elements[2], PdfElement::Image(_)));
        assert!(matches!(page.elements[3], PdfElement::Ink(_)));
        assert!(matches!(page.elements[4], PdfElement::Link(_)));
        let PdfElement::Shape(shape) = &page.elements[0] else {
            panic!("expected shape");
        };
        assert_eq!(shape.border.as_deref(), Some("0.75pt solid #c00000"));
        assert_eq!(shape.border_width_pt, Some(0.75));
        assert_eq!(shape.border_color.as_deref(), Some("#c00000"));
        let PdfElement::Text(text) = &page.elements[1] else {
            panic!("expected text");
        };
        assert_eq!(
            text.font_family.as_deref(),
            Some("Courier New, Courier, monospace")
        );
        assert_eq!(text.font_weight, Some(700));
        assert_eq!(text.font_style.as_deref(), Some("italic"));
        assert_eq!(text.rotation_deg, Some(90.0));
        assert_eq!(text.scale_x, Some(0.68));
        assert_eq!(doc.blocks.get(1).map(Block::text), Some("Hello".into()));
    }

    #[test]
    fn parses_real_output_visual_layers_when_available() {
        let Some(html) =
            output_fixture("Installation Guidelines - Prevention of Moisture Ingress for Outdoor Applications 2019-9-11 (2).html")
        else {
            return;
        };
        let doc = parse_html(&html);
        let pages = pdf_pages(&doc);
        assert!(!pages.is_empty());
        let elements = pages
            .iter()
            .flat_map(|page| page.elements.iter())
            .collect::<Vec<_>>();
        assert!(elements
            .iter()
            .any(|element| matches!(element, PdfElement::Text(_))));
        assert!(elements
            .iter()
            .any(|element| matches!(element, PdfElement::Shape(_))));
        assert!(elements
            .iter()
            .any(|element| matches!(element, PdfElement::Image(_))));
        assert!(elements
            .iter()
            .any(|element| matches!(element, PdfElement::Ink(_))));
        assert!(elements
            .iter()
            .any(|element| matches!(element, PdfElement::Link(_))));
        assert!(doc
            .blocks
            .iter()
            .any(|block| !matches!(block, Block::PdfPage(_))));
    }

    #[test]
    fn parses_real_output_borders_when_available() {
        let Some(html) =
            output_fixture("Installation Guidelines - Prevention of Moisture Ingress for Outdoor Applications 2019-9-11 (2).html")
        else {
            return;
        };
        let doc = parse_html(&html);
        let pages = pdf_pages(&doc);
        let bordered_shape = pages
            .iter()
            .flat_map(|page| &page.elements)
            .find_map(|element| match element {
                PdfElement::Shape(shape) if shape.border.is_some() => Some(shape),
                _ => None,
            })
            .expect("expected at least one bordered shape in output fixture");
        assert!(bordered_shape.border_width_pt.is_some());
        assert!(bordered_shape.border_color.is_some());
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

    #[test]
    fn parses_table_cells_in_source_order() {
        let doc = parse_html(
            r#"<body><article><table><tr><td>A</td><th>B</th><td>C</td></tr></table></article></body>"#,
        );
        let Some(Block::Table(table)) = doc.blocks.first() else {
            panic!("expected table");
        };
        let row = &table.rows[0];
        assert_eq!(chars_to_string(&row.cells[0].content), "A");
        assert!(!row.cells[0].header);
        assert_eq!(chars_to_string(&row.cells[1].content), "B");
        assert!(row.cells[1].header);
        assert_eq!(chars_to_string(&row.cells[2].content), "C");
        assert!(!row.cells[2].header);
    }

    #[test]
    fn serializes_pdf_visual_pages_with_extracted_article() {
        let doc = parse_html(
            r#"<body><main class="pdf-reconstructed-document"><section class="pdf-recreated-page" data-page="1" style="width:10.00pt;height:10.00pt"><span class="pdf-text-fragment" style="left:1.00pt;top:2.00pt;font-size:3.00pt;color:#000000">Hi</span></section><details class="pdf-extracted-content"><summary>Extracted</summary><article><p>Hello</p></article></details></main></body>"#,
        );
        let html = serialize_document(&doc);
        assert!(html.contains(r#"<main class="pdf-reconstructed-document">"#));
        assert!(html.contains(r#"<section class="pdf-recreated-page" data-page="1""#));
        assert!(html.contains(r#"class="pdf-text-fragment""#));
        assert!(html.contains(r#"<details class="pdf-extracted-content" open>"#));
        assert!(html.contains("<p>Hello</p>"));
    }

    fn output_fixture(name: &str) -> Option<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("output")
            .join(name);
        std::fs::read_to_string(path).ok()
    }

    fn pdf_pages(doc: &Doc) -> Vec<&PdfPage> {
        doc.blocks
            .iter()
            .filter_map(|block| match block {
                Block::PdfPage(page) => Some(page),
                _ => None,
            })
            .collect()
    }
}
