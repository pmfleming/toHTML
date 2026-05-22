use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use tohtml::{docx_to_document, markdown_to_document, pdf_to_document, render_html, ConvertError};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    let options = Options::parse(args)?;
    let input = fs::read(&options.input)?;
    let format = options
        .format
        .or_else(|| Format::from_path(&options.input))
        .ok_or(CliError::UnknownFormat)?;
    let html = render_html(&convert(format, &input)?);

    if let Some(output) = options.output {
        fs::write(output, html)?;
    } else {
        print!("{html}");
    }
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
enum Format {
    Markdown,
    Docx,
    Pdf,
}

impl Format {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Markdown),
            "docx" => Ok(Self::Docx),
            "pdf" => Ok(Self::Pdf),
            _ => Err(CliError::InvalidFormat(value.to_string())),
        }
    }

    fn from_path(path: &Path) -> Option<Self> {
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
}

#[derive(Debug, Default)]
struct Options {
    input: PathBuf,
    output: Option<PathBuf>,
    format: Option<Format>,
    asset_dir: Option<PathBuf>,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, CliError> {
        if args.is_empty() {
            return Err(CliError::Usage);
        }

        let mut options = Options::default();
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "-o" | "--output" => {
                    options.output = Some(next_path(&args, &mut index, "--output")?)
                }
                "--format" => {
                    options.format =
                        Some(Format::parse(next_value(&args, &mut index, "--format")?)?)
                }
                "--asset-dir" => {
                    options.asset_dir = Some(next_path(&args, &mut index, "--asset-dir")?)
                }
                "-h" | "--help" => return Err(CliError::Usage),
                value if value.starts_with('-') => {
                    return Err(CliError::UnknownOption(value.to_string()))
                }
                value => set_input(&mut options, value)?,
            }
            index += 1;
        }

        if options.input.as_os_str().is_empty() {
            return Err(CliError::MissingInput);
        }
        Ok(options)
    }
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
enum CliError {
    Usage,
    MissingInput,
    MissingOptionValue(&'static str),
    UnknownOption(String),
    UnexpectedArgument(String),
    InvalidFormat(String),
    UnknownFormat,
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
            Self::Convert(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

fn usage() -> &'static str {
    "usage: tohtml <input> [--format markdown|docx|pdf] [--output file] [--asset-dir dir]"
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
}
