use std::collections::HashMap;

use super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfReference, PdfValue};

pub fn role_map(objects: &PdfObjects) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let Some(root) = struct_tree_root(objects) else {
        return map;
    };
    let aliases = role_aliases(objects, root);
    let mut visited = Vec::new();
    walk_element(objects, root, None, &aliases, &mut map, &mut visited);
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
    inherited_role: Option<&str>,
    aliases: &HashMap<String, String>,
    map: &mut HashMap<u32, String>,
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
    let role = dictionary
        .name("S")
        .map(|role| mapped_role(role, aliases))
        .or_else(|| inherited_role.map(str::to_string));
    let Some(kids) = dictionary.get("K") else {
        return;
    };
    walk_kids(objects, kids, role.as_deref(), aliases, map, visited);
}

fn walk_kids(
    objects: &PdfObjects,
    kids: &PdfValue,
    role: Option<&str>,
    aliases: &HashMap<String, String>,
    map: &mut HashMap<u32, String>,
    visited: &mut Vec<PdfReference>,
) {
    match kids {
        PdfValue::Integer(mcid) => record_mcid(map, role, *mcid),
        PdfValue::Reference(reference) => {
            walk_element(objects, *reference, role, aliases, map, visited)
        }
        PdfValue::Dictionary(dictionary) => {
            walk_kid_dictionary(objects, dictionary, role, aliases, map, visited)
        }
        PdfValue::Array(values) => {
            for value in values {
                walk_kids(objects, value, role, aliases, map, visited);
            }
        }
        _ => {}
    }
}

fn walk_kid_dictionary(
    objects: &PdfObjects,
    dictionary: &PdfDictionary,
    role: Option<&str>,
    aliases: &HashMap<String, String>,
    map: &mut HashMap<u32, String>,
    visited: &mut Vec<PdfReference>,
) {
    if let Some(PdfValue::Integer(mcid)) = dictionary.get("MCID") {
        record_mcid(map, role, *mcid);
        return;
    }
    if let Some(kids) = dictionary.get("K") {
        let nested_role = dictionary
            .name("S")
            .map(|role| mapped_role(role, aliases))
            .or_else(|| role.map(str::to_string));
        walk_kids(objects, kids, nested_role.as_deref(), aliases, map, visited);
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

fn record_mcid(map: &mut HashMap<u32, String>, role: Option<&str>, mcid: i64) {
    let Some(role) = role else {
        return;
    };
    let Ok(mcid) = u32::try_from(mcid) else {
        return;
    };
    map.entry(mcid).or_insert_with(|| role.to_string());
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

        assert_eq!(map.get(&0).map(String::as_str), Some("H1"));
        assert_eq!(map.get(&1).map(String::as_str), Some("P"));
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

        assert_eq!(map.get(&0).map(String::as_str), Some("H1"));
    }
}
