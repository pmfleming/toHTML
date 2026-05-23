#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub metadata: DocumentMetadata,
    pub blocks: Vec<Block>,
    pub assets: Vec<Asset>,
    pub warnings: Vec<ConversionWarning>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            metadata: DocumentMetadata {
                title: Some(title.into()),
                ..DocumentMetadata::default()
            },
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub source_format: Option<SourceFormat>,
    pub language: Option<String>,
    pub visual_html: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Markdown,
    Docx,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(Heading),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
    Image(Image),
    BlockQuote(BlockQuote),
    CodeBlock(CodeBlock),
    PageBreak(PageBreak),
    PagePlaceholder(PagePlaceholder),
    HorizontalRule,
    RawHtml(RawHtml),
}

impl Block {
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Self::Heading(Heading {
            level,
            content: vec![Inline::text(text)],
            source: None,
        })
    }

    pub fn paragraph(text: impl Into<String>) -> Self {
        Self::Paragraph(Paragraph {
            content: vec![Inline::text(text)],
            source: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub content: Vec<Inline>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paragraph {
    pub content: Vec<Inline>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    pub ordered: bool,
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub rows: Vec<TableRow>,
    pub caption: Option<Vec<Inline>>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    pub content: Vec<Inline>,
    pub header: bool,
    pub colspan: u16,
    pub rowspan: u16,
    pub align: Option<TableAlignment>,
    pub source: Option<SourceSpan>,
}

impl TableCell {
    pub fn text(text: impl Into<String>, header: bool) -> Self {
        Self {
            content: vec![Inline::text(text)],
            header,
            colspan: 1,
            rowspan: 1,
            align: None,
            source: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub asset_id: Option<String>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockQuote {
    pub blocks: Vec<Block>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageBreak {
    pub page_number: Option<u32>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagePlaceholder {
    pub page_number: Option<u32>,
    pub reason: PlaceholderReason,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderReason {
    Empty,
    NonExtractable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawHtml {
    pub html: String,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link(Link),
    Image(Image),
    LineBreak,
}

impl Inline {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub href: String,
    pub title: Option<String>,
    pub content: Vec<Inline>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub id: String,
    pub path: String,
    pub media_type: Option<String>,
    pub alt: Option<String>,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionWarning {
    pub message: String,
    pub source: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub format: SourceFormat,
    pub page: Option<u32>,
    pub path: Option<String>,
    pub byte_range: Option<ByteRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_can_hold_metadata_assets_and_warnings() {
        let mut document = Document::with_title("Example");
        document.metadata.source_format = Some(SourceFormat::Markdown);
        document.assets.push(Asset {
            id: "asset-1".to_string(),
            path: "assets/image.png".to_string(),
            media_type: Some("image/png".to_string()),
            alt: Some("Diagram".to_string()),
            source: None,
        });
        document.warnings.push(ConversionWarning {
            message: "Skipped unsupported detail".to_string(),
            source: None,
        });

        assert_eq!(document.metadata.title.as_deref(), Some("Example"));
        assert_eq!(
            document.metadata.source_format,
            Some(SourceFormat::Markdown)
        );
        assert_eq!(document.assets.len(), 1);
        assert_eq!(document.warnings.len(), 1);
    }

    #[test]
    fn table_cells_default_to_single_span() {
        let cell = TableCell::text("Name", true);

        assert_eq!(cell.colspan, 1);
        assert_eq!(cell.rowspan, 1);
        assert!(cell.header);
    }

    #[test]
    fn page_placeholder_records_non_extractable_pdf_pages() {
        let placeholder = Block::PagePlaceholder(PagePlaceholder {
            page_number: Some(3),
            reason: PlaceholderReason::NonExtractable,
            source: Some(SourceSpan {
                format: SourceFormat::Pdf,
                page: Some(3),
                path: None,
                byte_range: None,
            }),
        });

        assert!(matches!(
            placeholder,
            Block::PagePlaceholder(PagePlaceholder {
                reason: PlaceholderReason::NonExtractable,
                ..
            })
        ));
    }
}
