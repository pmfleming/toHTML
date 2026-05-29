use std::collections::HashMap;

use super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

pub type McidMap<T> = HashMap<McidScope, T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct McidScope {
    pub page: Option<PdfReference>,
    pub mcid: u32,
}

impl McidScope {
    pub fn new(page: Option<PdfReference>, mcid: u32) -> Self {
        Self { page, mcid }
    }

    pub fn unscoped(mcid: u32) -> Self {
        Self { page: None, mcid }
    }
}

pub fn role_map(objects: &PdfObjects) -> McidMap<String> {
    info_map(objects)
        .into_iter()
        .filter_map(|(mcid, info)| info.role.map(|role| (mcid, role)))
        .collect()
}

pub fn actual_text_map(objects: &PdfObjects) -> McidMap<String> {
    info_map(objects)
        .into_iter()
        .filter_map(|(mcid, info)| info.actual_text.map(|text| (mcid, text)))
        .collect()
}

#[derive(Debug, Clone, Default)]
struct StructInfo {
    role: Option<String>,
    actual_text: Option<String>,
    page: Option<PdfReference>,
}

fn info_map(objects: &PdfObjects) -> McidMap<StructInfo> {
    let mut map = HashMap::new();
    let Some(root) = struct_tree_root(objects) else {
        return map;
    };
    let aliases = role_aliases(objects, root);
    let mut visited = Vec::new();
    walk_element(
        objects,
        root,
        StructInfo::default(),
        &aliases,
        &mut map,
        &mut visited,
    );
    map
}

fn struct_tree_root(objects: &PdfObjects) -> Option<PdfReference> {
    objects
        .values()
        .filter(|object| object.type_name() == Some("Catalog"))
        .find_map(|object| object.dictionary()?.get_ref("StructTreeRoot"))
}

fn walk_element(
    objects: &PdfObjects,
    reference: PdfReference,
    inherited: StructInfo,
    aliases: &HashMap<String, String>,
    map: &mut McidMap<StructInfo>,
    visited: &mut Vec<PdfReference>,
) {
    if visited.contains(&reference) {
        return;
    }
    visited.push(reference);
    let Some(object) = objects
        .get(reference)
        .or_else(|| objects.latest(reference.object))
    else {
        return;
    };
    let Some(dictionary) = object.dictionary() else {
        return;
    };
    let mut info = inherited;
    if let Some(role) = dictionary.name("S").map(|role| mapped_role(role, aliases)) {
        info.role = Some(role);
    }
    if let Some(page) = dictionary.get_ref("Pg") {
        info.page = Some(page);
    }
    if let Some(actual_text) = dictionary_text(dictionary, "ActualText") {
        info.actual_text = Some(actual_text);
    } else if let Some(alt) = dictionary_text(dictionary, "Alt") {
        info.actual_text = Some(alt);
    } else if let Some(expansion) = dictionary_text(dictionary, "E") {
        info.actual_text = Some(expansion);
    }
    let Some(kids) = dictionary.get("K") else {
        return;
    };
    walk_kids(objects, kids, info, aliases, map, visited);
}

fn walk_kids(
    objects: &PdfObjects,
    kids: &PdfValue,
    info: StructInfo,
    aliases: &HashMap<String, String>,
    map: &mut McidMap<StructInfo>,
    visited: &mut Vec<PdfReference>,
) {
    match kids {
        PdfValue::Integer(mcid) => record_mcid(map, &info, *mcid),
        PdfValue::Reference(reference) => {
            walk_element(objects, *reference, info, aliases, map, visited)
        }
        PdfValue::Dictionary(dictionary) => {
            walk_kid_dictionary(objects, dictionary, info, aliases, map, visited)
        }
        PdfValue::Array(values) => {
            for value in values {
                walk_kids(objects, value, info.clone(), aliases, map, visited);
            }
        }
        _ => {}
    }
}

fn walk_kid_dictionary(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    mut info: StructInfo,
    aliases: &HashMap<String, String>,
    map: &mut McidMap<StructInfo>,
    visited: &mut Vec<PdfReference>,
) {
    if let Some(role) = dictionary.name("S").map(|role| mapped_role(role, aliases)) {
        info.role = Some(role);
    }
    if let Some(page) = dictionary.get_ref("Pg") {
        info.page = Some(page);
    }
    if let Some(actual_text) = dictionary_text(dictionary, "ActualText") {
        info.actual_text = Some(actual_text);
    } else if let Some(alt) = dictionary_text(dictionary, "Alt") {
        info.actual_text = Some(alt);
    } else if let Some(expansion) = dictionary_text(dictionary, "E") {
        info.actual_text = Some(expansion);
    }
    if let Some(PdfValue::Integer(mcid)) = dictionary.get("MCID") {
        record_mcid(map, &info, *mcid);
        return;
    }
    if let Some(kids) = dictionary.get("K") {
        walk_kids(objects, kids, info, aliases, map, visited);
    }
}

fn role_aliases(objects: &PdfObjects, root: PdfReference) -> HashMap<String, String> {
    let Some(root) = objects.get(root).or_else(|| objects.latest(root.object)) else {
        return HashMap::new();
    };
    let Some(dictionary) = root.dictionary() else {
        return HashMap::new();
    };
    let Some(PdfValue::Dictionary(role_map)) = dictionary.get("RoleMap") else {
        return HashMap::new();
    };
    role_map
        .iter()
        .filter_map(|(from, to)| match to {
            PdfValue::Name(to) => Some((from.clone(), to.clone())),
            _ => None,
        })
        .collect()
}

fn mapped_role(role: &str, aliases: &HashMap<String, String>) -> String {
    aliases
        .get(role)
        .cloned()
        .unwrap_or_else(|| role.to_string())
}

fn record_mcid(map: &mut McidMap<StructInfo>, info: &StructInfo, mcid: i64) {
    let Ok(mcid) = u32::try_from(mcid) else {
        return;
    };
    map.entry(McidScope::new(info.page, mcid))
        .or_insert_with(|| info.clone());
}

fn dictionary_text(dictionary: &PdfDictionary, key: &str) -> Option<String> {
    dictionary
        .string_bytes(key)
        .map(super::text::decode_string)
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mcid_to_struct_role() {
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /StructTreeRoot 2 0 R >> endobj
2 0 obj << /Type /StructTreeRoot /K 3 0 R >> endobj
3 0 obj << /Type /StructElem /S /H1 /K [0 4 0 R] >> endobj
4 0 obj << /Type /StructElem /S /P /K [1] >> endobj
%%EOF"#;

        let objects = PdfObjects::parse(pdf);
        let map = role_map(&objects);

        assert_eq!(
            map.get(&McidScope::unscoped(0)).map(String::as_str),
            Some("H1")
        );
        assert_eq!(
            map.get(&McidScope::unscoped(1)).map(String::as_str),
            Some("P")
        );
    }

    #[test]
    fn applies_struct_tree_role_map_aliases() {
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /StructTreeRoot 2 0 R >> endobj
2 0 obj << /Type /StructTreeRoot /RoleMap << /Title /H1 >> /K 3 0 R >> endobj
3 0 obj << /Type /StructElem /S /Title /K [0] >> endobj
%%EOF"#;

        let objects = PdfObjects::parse(pdf);
        let map = role_map(&objects);

        assert_eq!(
            map.get(&McidScope::unscoped(0)).map(String::as_str),
            Some("H1")
        );
    }

    #[test]
    fn maps_structure_actual_text_to_mcid() {
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /StructTreeRoot 2 0 R >> endobj
2 0 obj << /Type /StructTreeRoot /K 3 0 R >> endobj
3 0 obj << /Type /StructElem /S /Span /ActualText (semantic replacement) /K [0] >> endobj
%%EOF"#;

        let objects = PdfObjects::parse(pdf);
        let map = actual_text_map(&objects);

        assert_eq!(
            map.get(&McidScope::unscoped(0)).map(String::as_str),
            Some("semantic replacement")
        );
    }

    #[test]
    fn scopes_structure_actual_text_by_page_reference() {
        let pdf = br#"%PDF-1.4
1 0 obj << /Type /Catalog /StructTreeRoot 2 0 R >> endobj
2 0 obj << /Type /StructTreeRoot /K [5 0 R 6 0 R] >> endobj
3 0 obj << /Type /Page >> endobj
4 0 obj << /Type /Page >> endobj
5 0 obj << /Type /StructElem /S /Span /Pg 3 0 R /ActualText (first) /K [0] >> endobj
6 0 obj << /Type /StructElem /S /Span /Pg 4 0 R /ActualText (second) /K [0] >> endobj
%%EOF"#;

        let objects = PdfObjects::parse(pdf);
        let map = actual_text_map(&objects);

        assert_eq!(
            map.get(&McidScope::new(
                Some(PdfReference {
                    object: 3,
                    generation: 0
                }),
                0
            ))
            .map(String::as_str),
            Some("first")
        );
        assert_eq!(
            map.get(&McidScope::new(
                Some(PdfReference {
                    object: 4,
                    generation: 0
                }),
                0
            ))
            .map(String::as_str),
            Some("second")
        );
    }
}
