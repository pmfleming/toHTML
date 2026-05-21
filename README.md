# toHTML

`toHTML` is a planned local-first document converter for turning common document
formats into clean, structured HTML.

Target inputs:

- PDF
- DOCX
- XLSX
- PPTX
- HTML
- images

The goal is semantic HTML for search, indexing, archival, and downstream LLM
workflows. It is not intended to be a pixel-perfect document renderer.

## Planned Shape

```text
input file
  -> type detection
  -> format converter
  -> shared document model
  -> HTML renderer
  -> assets directory
```

## MVP Scope

- Rust library plus CLI
- Shared document model for headings, paragraphs, lists, tables, images, and
  page breaks
- HTML input normalization
- DOCX text/table extraction
- XLSX sheet-to-table conversion
- Selectable-text PDF extraction
- Image OCR through a pluggable backend

## Non-Goals

- Pixel-perfect visual reconstruction
- Full CSS/layout preservation
- Remote document fetching by default
- Cloud OCR requirement

