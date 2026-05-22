use crate::Image;

use crate::html::inlines::render_image_tag;

pub(super) fn render_image_block(html: &mut String, image: &Image) {
    html.push_str("    ");
    render_image_tag(html, image);
    html.push('\n');
}
