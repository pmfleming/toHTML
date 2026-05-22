use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ConvertError {
    InvalidArchive(String),
    MissingPart(&'static str),
    Xml(String),
    Io(String),
    Pdf(String),
}

impl Display for ConvertError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArchive(message) => write!(formatter, "invalid archive: {message}"),
            Self::MissingPart(part) => write!(formatter, "missing required document part: {part}"),
            Self::Xml(message) => write!(formatter, "xml error: {message}"),
            Self::Io(message) => write!(formatter, "io error: {message}"),
            Self::Pdf(message) => write!(formatter, "pdf error: {message}"),
        }
    }
}

impl std::error::Error for ConvertError {}

impl From<zip::result::ZipError> for ConvertError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::InvalidArchive(error.to_string())
    }
}

impl From<std::io::Error> for ConvertError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<quick_xml::Error> for ConvertError {
    fn from(error: quick_xml::Error) -> Self {
        Self::Xml(error.to_string())
    }
}
