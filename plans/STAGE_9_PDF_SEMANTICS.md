# Stage 9 PDF Semantics

This follow-up stage keeps PDF output aligned with the project goal: semantic
HTML for indexing, archival, and downstream LLM use, not visual replacement.
Output font and pixel fidelity are intentionally out of scope.

## Progress

- Replaced broad parenthesized-byte scanning with operator-aware text extraction.
- Added ToUnicode CMap parsing for `beginbfchar` and `beginbfrange` mappings.
- Tracks active PDF font, font size, and common text-positioning operators.
- Groups positioned text into lines and detects simple aligned text tables.
- Preserves meaningful `TJ` array spacing when large text adjustments imply a
  word gap.
- Emits a conversion warning when PDFs contain image XObjects that may include
  non-selectable text.

## Current Sample Findings

`How To Program a Driver.pdf` now avoids binary/image/font stream garbage and
extracts the selectable headings and link-like text. The spec-sheet tables in
the sample are image XObjects, so they are not recoverable as semantic tables
without OCR.

Remaining text-joining issues in this sample include compressed word boundaries
such as `withoutChangingthe`, `Ichange`, and `programm er`. These require better
font-width handling and/or more advanced word-boundary inference.

## Next Work

- Parse font width data and use it for segment advance estimates instead of the
  current fallback width heuristic.
- Expand CMap support for multi-byte codes, more range forms, and embedded
  object-reference edge cases.
- Improve word joining with confidence-based gap classification and focused
  fixtures for tight glyph positioning.
- Reconstruct tables from selectable text by clustering x-aligned cells across
  nearby lines.
- Keep scanned/image-only table extraction out of scope unless OCR is explicitly
  added as a separate stage.

## Quality Gate Notes

The PDF text and CMap work added meaningful capability, but
`rust-quality-lens` still reports PDF reader/parser modules as the top hotspots.
Further parser decomposition is part of this stage before considering the PDF
semantic pass complete.
