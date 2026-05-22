use crate::{SourceFormat, SourceSpan};

pub fn markdown_source() -> Option<SourceSpan> {
    Some(SourceSpan {
        format: SourceFormat::Markdown,
        page: None,
        path: None,
        byte_range: None,
    })
}
