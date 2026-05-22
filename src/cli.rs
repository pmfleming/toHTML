mod assets;
mod interactive;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use tohtml::{docx_to_document, markdown_to_document, pdf_to_document, render_html, ConvertError};

pub fn run_from_env() -> Result<(), CliError> {
    run(env::args().skip(1).collect())
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    let options = Options::parse(args)?;
    if options.interactive {
        return interactive::run();
    }

    let output = options
        .output
        .unwrap_or_else(|| default_output_path(&options.input));
    convert_file(
        &options.input,
        options.format,
        Some(output.as_path()),
        options.asset_dir.as_deref(),
    )
}

fn convert_file(
    input_path: &Path,
    selected_format: Option<Format>,
    output: Option<&Path>,
    asset_dir: Option<&Path>,
) -> Result<(), CliError> {
    let input = fs::read(input_path)?;
    copy_input(input_path)?;
    let format = selected_format
        .or_else(|| Format::from_path(input_path))
        .ok_or(CliError::UnknownFormat)?;
    let mut document = convert(format, &input)?;
    if let Some(asset_dir) = asset_dir {
        assets::write(format, &input, &mut document, asset_dir, output)?;
    }
    let html = render_html(&document);

    if let Some(output) = output {
        write_output(output, &html)?;
    } else {
        print!("{html}");
    }
    Ok(())
}

fn copy_input(input_path: &Path) -> Result<(), CliError> {
    let Some(destination) = input_copy_path(input_path) else {
        return Ok(());
    };
    if same_file(input_path, &destination) {
        return Ok(());
    }

    fs::create_dir_all("input")?;
    fs::copy(input_path, destination)?;
    Ok(())
}

fn input_copy_path(input_path: &Path) -> Option<PathBuf> {
    Some(PathBuf::from("input").join(input_path.file_name()?))
}

fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn write_output(output: &Path, html: &str) -> Result<(), CliError> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(output, html)?;
    Ok(())
}

fn convert(format: Format, input: &[u8]) -> Result<tohtml::Document, ConvertError> {
    match format {
        Format::Markdown => Ok(markdown_to_document(&String::from_utf8_lossy(input))),
        Format::Docx => docx_to_document(input),
        Format::Pdf => pdf_to_document(input),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Format {
    Markdown,
    Docx,
    Pdf,
}

impl Format {
    pub(super) fn parse(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Markdown),
            "docx" => Ok(Self::Docx),
            "pdf" => Ok(Self::Pdf),
            _ => Err(CliError::InvalidFormat(value.to_string())),
        }
    }

    pub(super) fn from_path(path: &Path) -> Option<Self> {
        match path
            .extension()?
            .to_string_lossy()
            .to_ascii_lowercase()
            .as_str()
        {
            "md" | "markdown" => Some(Self::Markdown),
            "docx" => Some(Self::Docx),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Docx => "DOCX",
            Self::Pdf => "PDF",
        }
    }
}

#[derive(Debug, Default)]
struct Options {
    input: PathBuf,
    output: Option<PathBuf>,
    format: Option<Format>,
    asset_dir: Option<PathBuf>,
    interactive: bool,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, CliError> {
        if args.is_empty() {
            return Err(CliError::Usage);
        }

        let mut options = Options::default();
        let mut index = 0;
        while index < args.len() {
            parse_arg(&mut options, &args, &mut index)?;
            index += 1;
        }

        if !options.interactive && options.input.as_os_str().is_empty() {
            return Err(CliError::MissingInput);
        }
        Ok(options)
    }
}

fn parse_arg(options: &mut Options, args: &[String], index: &mut usize) -> Result<(), CliError> {
    match args[*index].as_str() {
        "-o" | "--output" => options.output = Some(next_path(args, index, "--output")?),
        "--format" => options.format = Some(Format::parse(next_value(args, index, "--format")?)?),
        "--asset-dir" => options.asset_dir = Some(next_path(args, index, "--asset-dir")?),
        "--interactive" | "/interactive" => options.interactive = true,
        "-h" | "--help" => return Err(CliError::Usage),
        value if value.starts_with('-') => return Err(CliError::UnknownOption(value.to_string())),
        value => set_input(options, value)?,
    }
    Ok(())
}

fn set_input(options: &mut Options, value: &str) -> Result<(), CliError> {
    if options.input.as_os_str().is_empty() {
        options.input = PathBuf::from(value);
        Ok(())
    } else {
        Err(CliError::UnexpectedArgument(value.to_string()))
    }
}

fn next_path(
    args: &[String],
    index: &mut usize,
    option: &'static str,
) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(next_value(args, index, option)?))
}

fn next_value<'a>(
    args: &'a [String],
    index: &mut usize,
    option: &'static str,
) -> Result<&'a str, CliError> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or(CliError::MissingOptionValue(option))
}

#[derive(Debug)]
pub enum CliError {
    Usage,
    MissingInput,
    MissingOptionValue(&'static str),
    UnknownOption(String),
    UnexpectedArgument(String),
    InvalidFormat(String),
    UnknownFormat,
    Interactive(String),
    Convert(ConvertError),
    Io(std::io::Error),
}

impl From<ConvertError> for CliError {
    fn from(error: ConvertError) -> Self {
        Self::Convert(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => write!(formatter, "{}", usage()),
            Self::MissingInput => write!(formatter, "missing input file\n\n{}", usage()),
            Self::MissingOptionValue(option) => write!(formatter, "missing value for {option}"),
            Self::UnknownOption(option) => write!(formatter, "unknown option: {option}"),
            Self::UnexpectedArgument(argument) => {
                write!(formatter, "unexpected argument: {argument}")
            }
            Self::InvalidFormat(format) => write!(formatter, "unsupported format: {format}"),
            Self::UnknownFormat => {
                write!(formatter, "could not detect input format; pass --format")
            }
            Self::Interactive(error) => write!(formatter, "interactive mode failed: {error}"),
            Self::Convert(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

fn usage() -> &'static str {
    "usage: tohtml <input> [--format markdown|docx|pdf] [--output file] [--asset-dir dir]\n       tohtml /interactive\n\nDefault output: output/<input-name>.html"
}

pub(super) fn default_output_path(input: &Path) -> PathBuf {
    PathBuf::from("output").join(default_output_name(input))
}

pub(super) fn default_output_name(input: &Path) -> String {
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("output"));
    PathBuf::from(stem)
        .with_extension("html")
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cli_options() {
        let options = Options::parse(vec![
            "input.md".to_string(),
            "--format".to_string(),
            "markdown".to_string(),
            "--output".to_string(),
            "out.html".to_string(),
        ])
        .unwrap();

        assert_eq!(options.input, PathBuf::from("input.md"));
        assert_eq!(options.format, Some(Format::Markdown));
        assert_eq!(options.output, Some(PathBuf::from("out.html")));
    }

    #[test]
    fn detects_format_from_extension() {
        assert_eq!(Format::from_path(Path::new("a.pdf")), Some(Format::Pdf));
        assert_eq!(Format::from_path(Path::new("a.docx")), Some(Format::Docx));
        assert_eq!(Format::from_path(Path::new("a.md")), Some(Format::Markdown));
    }

    #[test]
    fn parses_interactive_without_input() {
        let options = Options::parse(vec!["/interactive".to_string()]).unwrap();

        assert!(options.interactive);
        assert!(options.input.as_os_str().is_empty());
    }

    #[test]
    fn defaults_output_to_project_output_directory() {
        assert_eq!(
            default_output_path(Path::new("C:/docs/report.pdf")),
            PathBuf::from("output").join("report.html")
        );
    }

    #[test]
    fn input_copy_uses_project_input_directory() {
        assert_eq!(
            input_copy_path(Path::new("C:/docs/report.pdf")),
            Some(PathBuf::from("input").join("report.pdf"))
        );
    }
}
