use super::object::{PdfDictionary, PdfDictionaryExt, PdfObjects, PdfValue};
use super::text;

pub(super) fn document_title(objects: &PdfObjects) -> Option<String> {
    let title = info_dictionary(objects)?.string_bytes("Title")?;
    let decoded = text::decode_string(title);
    clean_document_title(&decoded)
}

fn clean_document_title(title: &str) -> Option<String> {
    let mut cleaned = title.trim();
    for prefix in [
        "Microsoft Word -",
        "Microsoft PowerPoint -",
        "Microsoft Excel -",
    ] {
        if let Some(stripped) = cleaned.strip_prefix(prefix) {
            cleaned = stripped.trim();
            break;
        }
    }

    if cleaned.is_empty()
        || is_generic_watermark_title(cleaned)
        || looks_like_generated_filename(cleaned)
    {
        return None;
    }

    Some(cleaned.to_string())
}

fn is_generic_watermark_title(title: &str) -> bool {
    matches!(
        title.trim().to_ascii_lowercase().as_str(),
        "english" | "draft" | "untitled"
    )
}

fn looks_like_generated_filename(title: &str) -> bool {
    let lower = title.trim().to_ascii_lowercase();
    const EXTENSIONS: &[&str] = &[
        ".doc", ".docx", ".xls", ".xlsx", ".pdf", ".pages", ".key", ".numbers",
    ];
    EXTENSIONS
        .iter()
        .any(|extension| lower.ends_with(extension))
}

pub(super) fn document_language(objects: &PdfObjects) -> Option<String> {
    let bytes = catalog_dictionary(objects)?.string_bytes("Lang")?;
    let decoded = text::decode_string(bytes);
    let trimmed = decoded.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn catalog_dictionary(objects: &PdfObjects) -> Option<&PdfDictionary> {
    objects
        .values()
        .find(|object| object.type_name() == Some("Catalog"))
        .and_then(|object| object.dictionary())
}

fn info_dictionary(objects: &PdfObjects) -> Option<&PdfDictionary> {
    objects
        .values()
        .filter_map(|object| object.dictionary())
        .find(|dictionary| {
            dictionary.type_name_is_none()
                && (dictionary.contains_key("Title")
                    || dictionary.contains_key("Author")
                    || dictionary.contains_key("Producer")
                    || dictionary.contains_key("Creator"))
        })
}

trait PdfDictionaryTypeCheck {
    fn type_name_is_none(&self) -> bool;
}

impl PdfDictionaryTypeCheck for PdfDictionary {
    fn type_name_is_none(&self) -> bool {
        !matches!(self.get("Type"), Some(PdfValue::Name(_)))
    }
}
