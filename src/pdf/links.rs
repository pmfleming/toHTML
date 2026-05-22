use crate::{Block, ConversionWarning, Inline, Link};

pub(super) fn apply_detected_links(
    blocks: &mut [Block],
    warnings: &mut Vec<ConversionWarning>,
    bytes: &[u8],
) {
    let mut uris = link_annotation_uris(bytes);
    uris.sort_by_key(|uri| std::cmp::Reverse(uri.len()));
    for uri in uris {
        if !link_uri_in_blocks(blocks, &uri) {
            warnings.push(ConversionWarning {
                message: format!(
                    "PDF link annotation target {uri} could not be associated with extracted text"
                ),
                source: None,
            });
        }
    }
}

pub(super) fn link_uri_in_blocks(blocks: &mut [Block], uri: &str) -> bool {
    let targets = visible_link_targets(uri);
    blocks.iter_mut().any(|block| match block {
        Block::Heading(heading) => link_uri_in_inlines(&mut heading.content, uri, &targets),
        Block::Paragraph(paragraph) => link_uri_in_inlines(&mut paragraph.content, uri, &targets),
        Block::List(list) => list.items.iter_mut().any(|item| {
            item.blocks
                .iter_mut()
                .any(|block| link_uri_in_blocks(std::slice::from_mut(block), uri))
        }),
        Block::Table(table) => table.rows.iter_mut().any(|row| {
            row.cells
                .iter_mut()
                .any(|cell| link_uri_in_inlines(&mut cell.content, uri, &targets))
        }),
        _ => false,
    })
}

pub(super) fn link_annotation_uris(bytes: &[u8]) -> Vec<String> {
    let mut uris = Vec::new();
    let mut cursor = 0;
    while let Some(uri_marker) = find_bytes(bytes, b"/URI", cursor) {
        cursor = uri_marker + b"/URI".len();
        if let Some(uri) = literal_after(bytes, cursor).filter(|uri| is_plausible_link_uri(uri)) {
            uris.push(uri);
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn link_uri_in_inlines(inlines: &mut Vec<Inline>, uri: &str, targets: &[String]) -> bool {
    let mut linked = false;
    *inlines = std::mem::take(inlines)
        .into_iter()
        .flat_map(|inline| link_uri_inline(inline, uri, targets, &mut linked))
        .collect();
    linked
}

fn link_uri_inline(
    inline: Inline,
    uri: &str,
    targets: &[String],
    linked: &mut bool,
) -> Vec<Inline> {
    let Inline::Text(text) = inline else {
        return vec![inline];
    };
    let Some((target, index)) = targets
        .iter()
        .find_map(|target| text.find(target).map(|index| (target, index)))
    else {
        return vec![Inline::Text(text)];
    };
    *linked = true;
    let mut output = Vec::new();
    if index > 0 {
        output.push(Inline::Text(text[..index].to_string()));
    }
    output.push(Inline::Link(Link {
        href: uri.to_string(),
        title: None,
        content: vec![Inline::Text(target.to_string())],
        source: None,
    }));
    let end = index + target.len();
    if end < text.len() {
        output.push(Inline::Text(text[end..].to_string()));
    }
    output
}

fn visible_link_targets(uri: &str) -> Vec<String> {
    let mut targets = vec![uri.to_string()];
    if let Some(without_scheme) = uri
        .strip_prefix("https://")
        .or_else(|| uri.strip_prefix("http://"))
    {
        let trimmed = without_scheme.trim_end_matches('/');
        if !trimmed.is_empty() {
            targets.push(trimmed.to_string());
        }
    }
    if let Some(address) = uri.strip_prefix("mailto:") {
        if !address.is_empty() {
            targets.push(address.to_string());
        }
    }
    targets.sort_by_key(|target| std::cmp::Reverse(target.len()));
    targets.dedup();
    targets
}

fn is_plausible_link_uri(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        value if value.starts_with("http://")
            || value.starts_with("https://")
            || value.starts_with("mailto:")
            || value.starts_with("www.")
    ) && !uri.chars().any(|ch| ch.is_control())
}

fn literal_after(bytes: &[u8], from: usize) -> Option<String> {
    let start = bytes[from..].iter().position(|byte| *byte == b'(')? + from + 1;
    let end = bytes[start..].iter().position(|byte| *byte == b')')? + start;
    Some(String::from_utf8_lossy(&bytes[start..end]).to_string())
}

fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| position + from)
}
