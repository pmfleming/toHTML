use super::escape::push_attr_escaped;

pub fn push_attr(html: &mut String, name: &str, value: &str) {
    html.push(' ');
    html.push_str(name);
    html.push_str("=\"");
    push_attr_escaped(html, value);
    html.push('"');
}

pub fn push_number_attr(html: &mut String, name: &str, value: Option<u64>) {
    if let Some(value) = value {
        push_attr(html, name, &value.to_string());
    }
}

pub fn push_end_tag(html: &mut String, tag: &str) {
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
}
