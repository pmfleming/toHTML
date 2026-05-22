use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::ConvertError;

#[derive(Debug, Clone, Default)]
pub struct Relationships {
    targets: HashMap<String, String>,
}

impl Relationships {
    pub fn parse(xml: &str) -> Result<Self, ConvertError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut targets = HashMap::new();

        loop {
            match reader.read_event()? {
                Event::Empty(element) | Event::Start(element) => {
                    if local_name(element.name().as_ref()) == b"Relationship" {
                        add_relationship(&mut targets, &element)?;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self { targets })
    }

    pub fn target(&self, id: &str) -> Option<String> {
        self.targets.get(id).map(|target| normalize_target(target))
    }
}

fn add_relationship(
    targets: &mut HashMap<String, String>,
    element: &BytesStart<'_>,
) -> Result<(), ConvertError> {
    let Some(id) = attr(element, b"Id")? else {
        return Ok(());
    };
    let Some(target) = attr(element, b"Target")? else {
        return Ok(());
    };
    targets.insert(id, target);
    Ok(())
}

fn normalize_target(target: &str) -> String {
    if target.starts_with('/') {
        target.trim_start_matches('/').to_string()
    } else if target.starts_with("word/") {
        target.to_string()
    } else {
        format!("word/{target}")
    }
}

fn attr(element: &BytesStart<'_>, wanted: &[u8]) -> Result<Option<String>, ConvertError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ConvertError::Xml(error.to_string()))?;
        if local_name(attribute.key.as_ref()) == wanted {
            return Ok(Some(String::from_utf8_lossy(&attribute.value).into_owned()));
        }
    }
    Ok(None)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}
