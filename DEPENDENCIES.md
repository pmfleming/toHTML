# Dependency Policy

External dependencies are exceptions, not the default design habit. When a
crate is retained, its use should sit behind a local project interface so the
rest of the codebase depends on this repository's vocabulary rather than on a
third-party API.

Current retained exceptions:

- `flate2`: zlib/deflate correctness for PDF streams and PNG image data. All
  production use goes through `src/pdf/compression.rs`.
- `encoding_rs`: predefined PDF CMap legacy/CJK decoding. Use is isolated in
  `src/pdf/cmap/predefined.rs`.
- `quick-xml`: DOCX XML parsing. Use is isolated under `src/docx/xml/` and
  `src/docx/relationships.rs`.
- `zip`: DOCX package and CLI asset extraction. Use is isolated in package or
  asset boundary modules.
- `ttf-parser`: embedded font inspection. Use is isolated under
  `src/pdf/cmap/embedded.rs` and its submodules.
- `eframe` and `rfd`: optional main CLI GUI picker dependencies. They are only
  built with the `interactive-gui` feature.

Local reference-derived code:

- `src/pdf/cmap/encoding/agl.rs`: generated from Adobe Glyph List
  `glyphlist.txt`, table version 2.0, from
  `https://github.com/adobe-type-tools/agl-aglfn`. The generated module retains
  Adobe's copyright, redistribution conditions, and warranty disclaimer in its
  header. It is exposed only through the local
  `encoding::glyph_name_unicode(name)` path.

For copied or reference-derived code, keep the smallest useful subset, attribute
the source/license in the local module, adapt it behind project-owned functions,
and revisit the subset when tests or real fixtures require broader coverage.
