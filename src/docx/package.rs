use std::io::{Cursor, Read};

use zip::ZipArchive;

use crate::ConvertError;

pub struct DocxPackage<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> DocxPackage<'a> {
    pub fn open(bytes: &'a [u8]) -> Result<Self, ConvertError> {
        Ok(Self {
            archive: ZipArchive::new(Cursor::new(bytes))?,
        })
    }

    pub fn read_required_string(&mut self, name: &'static str) -> Result<String, ConvertError> {
        self.read_optional_string(name)?
            .ok_or(ConvertError::MissingPart(name))
    }

    pub fn read_optional_string(&mut self, name: &str) -> Result<Option<String>, ConvertError> {
        let Ok(mut file) = self.archive.by_name(name) else {
            return Ok(None);
        };
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Some(contents))
    }

    pub fn media_paths(&mut self) -> Result<Vec<String>, ConvertError> {
        let mut paths = Vec::new();
        for index in 0..self.archive.len() {
            let file = self.archive.by_index(index)?;
            let name = file.name();
            if name.starts_with("word/media/") && !name.ends_with('/') {
                paths.push(name.to_string());
            }
        }
        Ok(paths)
    }
}
