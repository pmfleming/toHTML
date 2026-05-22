# Stage 3 Quality Gate

Stage 3 changed the renderer from an `<article>` fragment renderer into a full
HTML document renderer with no CSS and no JavaScript.

## Verification

- `cargo test`
- `CARGO_INCREMENTAL=0 cargo check --all-targets`
- `rust-quality-lens measure --config target/rqlens.toml`

## Lens Findings Addressed

The first Stage 3 quality-lens run showed `src/html/blocks.rs` as the dominant
renderer hotspot.

Block rendering was split by block family:

- lists
- media
- pages
- tables
- text-like blocks

This reduced the single block-rendering module hotspot and made the renderer
easier to work on incrementally.

## Residual Findings

The split introduced some locality noise because the block-family modules share
the same renderer helpers for attributes, escaping, and inline rendering. That
tradeoff is acceptable for now because the shared helpers keep escaping and
attribute behavior consistent.

The remaining clone groups in `src/model.rs` are still around explicit source
metadata fields. Those should be revisited after Markdown, DOCX, and PDF
converters all exercise the model.
