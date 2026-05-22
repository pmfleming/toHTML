use crate::{Image, Inline, Link};

use super::attrs::{push_attr, push_end_tag};
use super::escape::push_escaped;

pub fn render_inlines(html: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        render_inline(html, inline);
    }
}

pub fn render_image_tag(html: &mut String, image: &Image) {
    html.push_str("<img");
    push_attr(html, "src", &image.src);
    push_attr(html, "alt", image.alt.as_deref().unwrap_or(""));
    if let Some(title) = &image.title {
        push_attr(html, "title", title);
    }
    html.push('>');
}

fn render_inline(html: &mut String, inline: &Inline) {
    match inline {
        Inline::Text(text) => push_escaped(html, text),
        Inline::Emphasis(content) => render_wrapped_inlines(html, "em", content),
        Inline::Strong(content) => render_wrapped_inlines(html, "strong", content),
        Inline::Strikethrough(content) => render_wrapped_inlines(html, "del", content),
        Inline::Code(code) => render_inline_code(html, code),
        Inline::Link(link) => render_link(html, link),
        Inline::Image(image) => render_image_tag(html, image),
        Inline::LineBreak => html.push_str("<br>"),
    }
}

fn render_inline_code(html: &mut String, code: &str) {
    html.push_str("<code>");
    push_escaped(html, code);
    html.push_str("</code>");
}

fn render_wrapped_inlines(html: &mut String, tag: &str, content: &[Inline]) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    render_inlines(html, content);
    push_end_tag(html, tag);
}

fn render_link(html: &mut String, link: &Link) {
    html.push_str("<a");
    push_attr(html, "href", &link.href);
    if let Some(title) = &link.title {
        push_attr(html, "title", title);
    }
    html.push('>');
    render_inlines(html, &link.content);
    html.push_str("</a>");
}
