# Roadmap

## Phase 1: Core

- Define the internal document model.
- Implement the HTML renderer.
- Add CLI input/output handling.
- Add file type detection.

## Phase 2: First Converters

- HTML normalization and sanitization.
- DOCX paragraphs, headings, lists, tables, and images.
- XLSX sheets as HTML tables.
- Selectable-text PDF extraction.

## Phase 3: OCR

- Add an `OcrEngine` trait.
- Implement a Tesseract backend.
- Evaluate `ocrs` and Python OCR worker backends.
- Add automatic scanned-PDF detection.

## Phase 4: PPTX And Better Layout

- Extract slide text, titles, notes, tables, and images.
- Improve PDF paragraph reconstruction.
- Add table detection for PDF/OCR output.

## Phase 5: Packaging

- Stable Rust API.
- CLI releases for Windows and Linux.
- Optional JSON intermediate representation output.

