use std::path::Path;

use super::super::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormatChoice {
    Auto,
    Markdown,
    Docx,
    Pdf,
}

impl Default for FormatChoice {
    fn default() -> Self {
        Self::Auto
    }
}

impl FormatChoice {
    pub(super) fn all() -> [Self; 4] {
        [Self::Auto, Self::Markdown, Self::Docx, Self::Pdf]
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Markdown => Format::Markdown.label(),
            Self::Docx => Format::Docx.label(),
            Self::Pdf => Format::Pdf.label(),
        }
    }

    pub(super) fn format(self) -> Option<Format> {
        match self {
            Self::Auto => None,
            Self::Markdown => Some(Format::Markdown),
            Self::Docx => Some(Format::Docx),
            Self::Pdf => Some(Format::Pdf),
        }
    }

    pub(super) fn from_path(path: &Path) -> Option<Self> {
        match Format::from_path(path)? {
            Format::Markdown => Some(Self::Markdown),
            Format::Docx => Some(Self::Docx),
            Format::Pdf => Some(Self::Pdf),
        }
    }
}
