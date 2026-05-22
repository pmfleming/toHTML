mod rewrite;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use tohtml::{ConvertError, Document};
use zip::ZipArchive;

use super::{CliError, Format};

pub(super) fn write(
    format: Format,
    input: &[u8],
    document: &mut Document,
    asset_dir: &Path,
    output: Option<&Path>,
) -> Result<(), CliError> {
    match format {
        Format::Docx => write_docx(input, document, asset_dir, output),
        Format::Markdown | Format::Pdf => Ok(()),
    }
}

fn write_docx(
    input: &[u8],
    document: &mut Document,
    asset_dir: &Path,
    output: Option<&Path>,
) -> Result<(), CliError> {
    if document.assets.is_empty() {
        return Ok(());
    }

    let actual_asset_dir = actual_asset_dir(asset_dir, output);
    fs::create_dir_all(&actual_asset_dir)?;

    let mut archive = ZipArchive::new(Cursor::new(input)).map_err(ConvertError::from)?;
    let mut copied_by_path: HashMap<String, String> = HashMap::new();
    let mut src_by_id: HashMap<String, String> = HashMap::new();
    let mut src_by_original_path: HashMap<String, String> = HashMap::new();

    for asset in &mut document.assets {
        let original_path = asset.path.clone();
        let html_src = copied_asset_src(
            &mut archive,
            &actual_asset_dir,
            asset_dir,
            &mut copied_by_path,
            &original_path,
            &asset.id,
        )?;

        src_by_id.insert(asset.id.clone(), html_src.clone());
        src_by_original_path.insert(original_path, html_src.clone());
        asset.path = html_src;
    }

    rewrite::image_sources(document, &src_by_id, &src_by_original_path);
    Ok(())
}

fn copied_asset_src(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    actual_asset_dir: &Path,
    html_asset_dir: &Path,
    copied_by_path: &mut HashMap<String, String>,
    original_path: &str,
    fallback_name: &str,
) -> Result<String, CliError> {
    if let Some(src) = copied_by_path.get(original_path) {
        return Ok(src.clone());
    }

    let file_name = asset_file_name(original_path).unwrap_or(fallback_name);
    let output_path = actual_asset_dir.join(file_name);
    copy_docx_member(archive, original_path, &output_path)?;

    let src = html_asset_src(html_asset_dir, file_name);
    copied_by_path.insert(original_path.to_string(), src.clone());
    Ok(src)
}

fn actual_asset_dir(asset_dir: &Path, output: Option<&Path>) -> PathBuf {
    if asset_dir.is_absolute() {
        return asset_dir.to_path_buf();
    }

    output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join(asset_dir))
        .unwrap_or_else(|| asset_dir.to_path_buf())
}

fn asset_file_name(path: &str) -> Option<&str> {
    path.rsplit(['/', '\\'])
        .find(|part| !part.is_empty() && *part != "." && *part != "..")
}

fn copy_docx_member(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    member_name: &str,
    output_path: &Path,
) -> Result<(), CliError> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut member = archive.by_name(member_name).map_err(ConvertError::from)?;
    let mut output = fs::File::create(output_path)?;
    std::io::copy(&mut member, &mut output)?;
    Ok(())
}

fn html_asset_src(asset_dir: &Path, file_name: &str) -> String {
    asset_dir
        .join(file_name)
        .to_string_lossy()
        .replace('\\', "/")
}
