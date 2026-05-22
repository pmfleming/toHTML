# Stage 8 Quality Gate

Stage 8 added the CLI for converting Markdown, DOCX, and PDF inputs to full
HTML output.

## Verification

- `cargo test`
- `CARGO_INCREMENTAL=0 cargo check --all-targets`
- `rust-quality-lens measure --config target/rqlens.toml`

## Lens Findings Addressed

The first Stage 8 quality-lens run showed `src/main.rs` as a new hotspot:

- score: 341.63
- cognitive: 14
- cyclomatic: 70

The CLI parser and runtime were moved into `src/cli.rs`, leaving `src/main.rs`
as a small process wrapper.

## Residual Findings

After the split, `src/cli.rs` remains a contained hotspot:

- score: 335.54
- cognitive: 12
- cyclomatic: 71

That is acceptable for this stage because the CLI has no external parser
dependency and is covered by focused tests. A future pass can split argument
parsing and command execution if the CLI surface grows.
