use pdf_font::cmap::{self as pdf_cmap, BfString, CMapName, CharacterCollection, CidFamily};

use super::hex::code_value;

#[derive(Debug, Clone)]
pub(super) enum PredefinedCMap {
    Utf16Be,
    Embedded {
        encoding: pdf_cmap::CMap,
        unicode: pdf_cmap::CMap,
    },
}

impl PredefinedCMap {
    pub(super) fn max_code_len(&self) -> usize {
        match self {
            Self::Utf16Be => 2,
            Self::Embedded { .. } => 4,
        }
    }

    pub(super) fn decode_prefix(&self, bytes: &[u8], index: usize) -> Option<(String, usize)> {
        match self {
            Self::Utf16Be => decode_utf16be_prefix(bytes, index),
            Self::Embedded { encoding, unicode } => {
                decode_embedded_prefix(encoding, unicode, bytes, index)
            }
        }
    }
}

pub(super) fn predefined_cmap(name: &str) -> Option<PredefinedCMap> {
    if is_unicode_cmap(name) {
        return Some(PredefinedCMap::Utf16Be);
    }

    let name = CMapName::from_bytes(name.as_bytes());
    let encoding = load_cmap(name)?;
    let collection = encoding.metadata().character_collection.as_ref()?;
    let unicode = unicode_cmap_for_collection(collection)?;
    Some(PredefinedCMap::Embedded { encoding, unicode })
}

pub(super) fn identity_cmap(
    registry: &str,
    ordering: &str,
    vertical: bool,
) -> Option<PredefinedCMap> {
    if registry == "Adobe" && ordering == "UCS" {
        return Some(PredefinedCMap::Utf16Be);
    }

    let family = CidFamily::from_registry_ordering(registry.as_bytes(), ordering.as_bytes());
    let unicode = unicode_cmap_for_family(&family)?;
    let encoding = if vertical {
        pdf_cmap::CMap::identity_v()
    } else {
        pdf_cmap::CMap::identity_h()
    };

    Some(PredefinedCMap::Embedded { encoding, unicode })
}

fn is_unicode_cmap(name: &str) -> bool {
    let stem = name
        .strip_suffix("-H")
        .or_else(|| name.strip_suffix("-V"))
        .unwrap_or(name);
    stem.contains("UCS2") || stem.contains("UTF16")
}

fn decode_embedded_prefix(
    encoding: &pdf_cmap::CMap,
    unicode: &pdf_cmap::CMap,
    bytes: &[u8],
    index: usize,
) -> Option<(String, usize)> {
    let remaining = bytes.len().saturating_sub(index);
    for len in (1..=remaining.min(4)).rev() {
        let code = code_value(&bytes[index..index + len])?;
        if let Some(text) = encoding
            .lookup_cid_code(code, len as u8)
            .and_then(|cid| unicode.lookup_bf_string(cid))
            .and_then(bf_string_text)
        {
            return Some((text, len));
        }
        if let Some(text) = encoding.lookup_bf_string(code).and_then(bf_string_text) {
            return Some((text, len));
        }
    }
    None
}

fn unicode_cmap_for_collection(collection: &CharacterCollection) -> Option<pdf_cmap::CMap> {
    unicode_cmap_for_family(&collection.family)
}

fn unicode_cmap_for_family(family: &CidFamily) -> Option<pdf_cmap::CMap> {
    load_cmap(family.ucs2_cmap()?)
}

fn load_cmap(name: CMapName<'_>) -> Option<pdf_cmap::CMap> {
    let data = pdf_cmap::load_embedded(name)?;
    pdf_cmap::CMap::parse(data, pdf_cmap::load_embedded)
}

fn bf_string_text(value: BfString) -> Option<String> {
    match value {
        BfString::Char(ch) => Some(ch.to_string()),
        BfString::String(text) => Some(text),
    }
}

fn decode_utf16be_prefix(bytes: &[u8], index: usize) -> Option<(String, usize)> {
    if index + 1 >= bytes.len() {
        return None;
    }
    let code = u16::from_be_bytes([bytes[index], bytes[index + 1]]);
    let ch = char::from_u32(u32::from(code))?;
    Some((ch.to_string(), 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_embedded_shift_jis_prefixes_through_cid_collection() {
        let cmap = predefined_cmap("90ms-RKSJ-H").unwrap();

        assert_eq!(cmap.decode_prefix(b"A", 0), Some(("A".to_string(), 1)));
        assert_eq!(
            cmap.decode_prefix(&[0x82, 0xa0], 0),
            Some(("あ".to_string(), 2))
        );
    }

    #[test]
    fn decodes_utf16_known_unicode_cmaps() {
        let cmap = predefined_cmap("UniJIS-UCS2-H").unwrap();

        assert_eq!(
            cmap.decode_prefix(&[0x4e, 0x2d], 0),
            Some(("中".to_string(), 2))
        );
    }

    #[test]
    fn decodes_identity_cids_with_known_collection() {
        let cmap = identity_cmap("Adobe", "Japan1", false).unwrap();

        assert_eq!(
            cmap.decode_prefix(&[0x00, 0x22], 0),
            Some(("A".to_string(), 2))
        );
    }
}
