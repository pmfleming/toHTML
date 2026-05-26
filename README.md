# toHTML

`toHTML` is a local-first Rust document converter for turning a focused subset
of document formats into standalone, structured HTML.

Target inputs:

- PDF
- DOCX
- GitHub-flavored Markdown

The goal is semantic HTML for search, indexing, archival, downstream LLM
workflows, and visual review. PDF conversion uses a narrow positioned visual
layer for extracted text, vector shapes, and embedded images while keeping the
output standalone.

## Planned Shape

```text
input file
  -> type detection
  -> format converter
  -> shared document model
  -> HTML renderer
  -> optional assets directory
```

## MVP Scope

- Rust library plus CLI
- Shared document model for headings, paragraphs, lists, tables, images, and
- page placeholders
- GitHub-flavored Markdown conversion
- DOCX headings, paragraphs, lists, tables, and images
- Selectable-text PDF extraction
- Empty placeholders for PDF pages without extractable text

## CLI

```powershell
tohtml input.md --output output.html
tohtml input.docx --format docx --output output.html
tohtml input.pdf --asset-dir assets --output output.html
tohtml input.pdf --no-images --output text-and-vectors.html
tohtml /interactive
```

## Non-Goals

- Full CSS/layout preservation
- Remote document fetching by default
- Broad CSS, CSS page reconstruction, or JavaScript output
- OCR or scanned PDF recognition
- XLSX, PPTX, HTML, image, audio, or video input
