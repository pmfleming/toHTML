use quick_xml::events::BytesStart;

use crate::ConvertError;

pub fn attr(element: &BytesStart<'_>, wanted: &[u8]) -> Result<Option<String>, ConvertError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ConvertError::Xml(error.to_string()))?;
        if local_name(attribute.key.as_ref()) == wanted {
            return Ok(Some(String::from_utf8_lossy(&attribute.value).into_owned()));
        }
    }
    Ok(None)
}

pub fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}
