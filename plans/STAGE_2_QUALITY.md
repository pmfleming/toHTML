# Stage 2 Quality Gate

Stage 2 expanded the shared document model and kept a compatibility HTML
renderer in place so the crate continued to compile and test.

## Verification

- `cargo test`
- `CARGO_INCREMENTAL=0 cargo check --all-targets`
- `rust-quality-lens measure --config target/rqlens.toml`

## Lens Findings Addressed

The first quality-lens run showed `src/lib.rs` as the primary hotspot because it
contained both the public API and all renderer logic.

The renderer was moved into `src/html.rs`, leaving `src/lib.rs` as a thin public
surface. Repeated attribute and image rendering logic was collapsed into shared
helpers.

## Residual Findings

The second quality-lens run still reports `src/html.rs` as the largest hotspot.
That is expected at this point because Stage 3 is dedicated to replacing the
compatibility renderer with the full no-CSS/no-JS HTML renderer.

The model also has small token clone groups around repeated source metadata
fields. Those fields are intentionally explicit for now so Markdown, DOCX, and
PDF converters can attach provenance consistently. If they remain noisy after
the converters exist, the model should be revisited with real usage in hand.
