# toHTML Goal

`toHTML` is a Rust document converter for turning selected local document formats
into clean, structured HTML.

The project is intentionally narrow:

- PDF to HTML
- DOCX to HTML
- Markdown to HTML

The output is semantic HTML for search, indexing, archival, and downstream LLM
workflows. It is not intended to be a pixel-perfect renderer.

## Output Contract

The default output is a complete HTML document.

The generated document must not include JavaScript. Structure should come from
HTML elements and attributes first. Minimal CSS is allowed only where HTML cannot
express the readable-output requirement, such as cell alignment, and must not be
used to recreate PDF page positioning.

Core output structures include:

- document metadata
- headings
- paragraphs
- lists
- tables
- images
- page breaks or page placeholders
- conversion warnings when useful

## Format Scope

### Markdown

Markdown support should include GitHub-flavored Markdown features, including:

- headings
- paragraphs
- ordered and unordered lists
- task lists
- blockquotes
- fenced code blocks
- tables
- links
- images
- emphasis and strong emphasis
- strikethrough
- horizontal rules

Markdown conversion logic should be repository-owned.

### DOCX

The DOCX MVP includes:

- headings
- paragraphs
- lists
- tables
- images

Comments, footnotes, endnotes, advanced layout, and full style fidelity are out
of scope for the first pass.

ZIP decompression and XML tokenization may use external crates on this pass.
WordprocessingML interpretation and conversion behavior should be
repository-owned.

### PDF

The PDF MVP supports selectable-text PDFs only.

Scanned documents and OCR are explicitly excluded. If a page appears to have no
extractable text, the HTML should represent it as an empty page placeholder
rather than attempting OCR.

PDF conversion should prioritize stable structure over visual fidelity.

## Repository-Owned Code Rule

Prefer repository-owned code. External dependencies should be the exception, not
the normal design habit.

When any part of this project is based on code from a crate or reference
project, bring the smallest useful subset of that source into local modules in
this repository, then adapt it to this project.

Copied or reference-derived code must be:

- reduced to the smallest useful subset
- attributed where required
- adapted behind local project interfaces
- reviewed for complexity and maintainability

ZIP decompression and XML tokenization are exempt from this rule for the first
DOCX pass.

## Staged Plan

1. Write and lock this implementation goal and scope.
2. Expand the shared document model.
3. Build the full semantic HTML renderer with no JavaScript and only narrowly
   justified CSS.
4. Implement the repository-owned GitHub-flavored Markdown converter.
5. Implement DOCX conversion for headings, paragraphs, lists, tables, and images.
6. Implement selectable-text PDF conversion with empty placeholders for
   non-extractable pages.
7. Add fixture-driven golden tests for Markdown, DOCX, and PDF.
8. Add the CLI.

## Stage Quality Gate

Stages 2 through 8 follow the same completion process:

1. Complete stage `x`.
2. Push the completed stage to GitHub.
3. Run `https://github.com/pmfleming/rust-quality-lens`.
4. Use the quality-lens output to improve:
   - cognitive, cyclomatic, and effort metrics
   - leverage and locality
   - cloning and total lines of code
5. Only then begin work on stage `x + 1`.

Stage work is not considered complete until the implementation has been pushed,
quality-lens has run, and any worthwhile findings have been addressed or
explicitly documented.
