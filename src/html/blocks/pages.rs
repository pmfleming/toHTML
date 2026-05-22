use crate::{PageBreak, PagePlaceholder, PlaceholderReason};

use crate::html::attrs::{push_attr, push_number_attr};

pub(super) fn render_page_break(html: &mut String, page_break: &PageBreak) {
    html.push_str("    <hr data-page-break");
    push_number_attr(html, "data-page", page_break.page_number.map(u64::from));
    html.push_str(">\n");
}

pub(super) fn render_page_placeholder(html: &mut String, placeholder: &PagePlaceholder) {
    html.push_str("    <div data-page-placeholder");
    push_number_attr(html, "data-page", placeholder.page_number.map(u64::from));
    push_attr(html, "data-reason", placeholder_reason(placeholder.reason));
    html.push_str("></div>\n");
}

fn placeholder_reason(reason: PlaceholderReason) -> &'static str {
    match reason {
        PlaceholderReason::Empty => "empty",
        PlaceholderReason::NonExtractable => "non-extractable",
    }
}
