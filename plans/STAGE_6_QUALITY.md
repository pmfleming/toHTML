# Stage 6 Quality Gate

Stage 6 added selectable-text PDF conversion with empty placeholders for
non-extractable pages.

## Verification

- `cargo test`
- `CARGO_INCREMENTAL=0 cargo check --all-targets`
- `rust-quality-lens measure --config target/rqlens.toml`

## Lens Findings

The Stage 6 quality-lens run identified `src/pdf/text.rs` as the main new PDF
hotspot:

- score: 289.31
- cognitive: 18
- cyclomatic: 46

This module owns the deliberately small PDF text parser for literal strings,
hex strings, escapes, and dictionary skipping.

## Residual Findings

The PDF parser is intentionally narrow. It does not attempt OCR, layout
reconstruction, or full PDF semantics. The next useful quality signal should
come from fixture-driven tests that show whether the small parser is reliable
enough for selectable-text PDFs or whether a more formal PDF layer is warranted.

The only additional crate used in this stage is `flate2` for compressed content
streams. PDF conversion behavior remains repository-owned.
