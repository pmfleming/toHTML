# Stage 4 Quality Gate

Stage 4 added a repository-owned GitHub-flavored Markdown converter.

## Verification

- `cargo test`
- `CARGO_INCREMENTAL=0 cargo check --all-targets`
- `rust-quality-lens measure --config target/rqlens.toml`

## Lens Findings Addressed

The first Stage 4 quality-lens run showed `src/markdown/blocks.rs` as the
largest new hotspot:

- score: 448.46
- cognitive: 39
- cyclomatic: 68

The parser was split so block orchestration, block markers, list parsing, source
metadata, inline parsing, and table parsing live in separate local modules.

After the split, `src/markdown/blocks.rs` dropped to:

- score: 301.69
- cognitive: 26
- cyclomatic: 36

## Residual Findings

`src/markdown/inlines.rs` and `src/markdown/tables.rs` remain notable parser
hotspots. They are acceptable for this stage because the parser is still small,
repository-owned, and covered by focused tests.

The next meaningful quality pass for Markdown should happen after fixture-driven
golden tests exist. That will show whether the parser complexity is earning its
keep or whether specific GFM features should be simplified.
