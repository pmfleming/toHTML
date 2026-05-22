use crate::{BlockQuote, CodeBlock, Heading, Inline, RawHtml};

use super::render_blocks;
use crate::html::attrs::{push_attr, push_end_tag};
use crate::html::escape::push_escaped;
use crate::html::inlines::render_inlines;

pub(super) fn render_heading(html: &mut String, heading: &Heading) {
    let level = heading.level.clamp(1, 6);
    let tag = format!("h{level}");
    html.push_str("    ");
    render_wrapped_inlines(html, &tag, &heading.content);
    html.push('\n');
}

pub(super) fn render_paragraph(html: &mut String, content: &[Inline]) {
    html.push_str("    <p>");
    render_inlines(html, content);
    html.push_str("</p>\n");
}

pub(super) fn render_block_quote(html: &mut String, block_quote: &BlockQuote) {
    html.push_str("    <blockquote>\n");
    render_blocks(html, &block_quote.blocks);
    html.push_str("    </blockquote>\n");
}

pub(super) fn render_code_block(html: &mut String, code_block: &CodeBlock) {
    html.push_str("    <pre><code");
    if let Some(language) = &code_block.language {
        push_attr(html, "class", &format!("language-{language}"));
    }
    html.push('>');
    push_escaped(html, &code_block.code);
    html.push_str("</code></pre>\n");
}

pub(super) fn render_raw_html(html: &mut String, raw: &RawHtml) {
    html.push_str(&raw.html);
    if !raw.html.ends_with('\n') {
        html.push('\n');
    }
}

fn render_wrapped_inlines(html: &mut String, tag: &str, content: &[Inline]) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    render_inlines(html, content);
    push_end_tag(html, tag);
}
