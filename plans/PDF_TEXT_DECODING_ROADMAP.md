# PDF Text Decoding Roadmap

Goal: make PDF text extraction font-aware before applying readability repairs.
Rendering-oriented PDFs often store font character codes, not Unicode text, so
the decoder should recover Unicode from the active font whenever the PDF exposes
enough information.

## Phase 1: Simple Font Encoding Fallback

- Use `/ToUnicode` CMaps when present.
- For simple fonts without complete `/ToUnicode`, build a fallback map from
  `/Encoding`, `/BaseEncoding`, and `/Differences`.
- Resolve glyph names through Adobe Glyph List conventions, including `uniXXXX`,
  `uXXXX`, variant suffixes, and underscore-separated ligature components.
- Merge `/ToUnicode` over the fallback map so explicit semantic mappings win.

Status: implemented in `src/pdf/cmap`.

## Phase 2: Type0/CID CMaps

- Preserve current `/ToUnicode` handling for Type0 fonts.
- Support Identity-H/Identity-V for Adobe-Identity and Adobe-UCS collections as
  a conservative two-byte identity fallback.
- Add bundled system CMaps for Adobe-CNS1, Adobe-GB1, Adobe-Japan1, and
  Adobe-Korea1, or integrate a compact generated subset of those maps.
- Treat missing known CMap data as a diagnostic, not as a reason to run prose
  repair globally.

Status: Identity fallback implemented; full collection maps still pending.

## Phase 3: Embedded Font Tables

- Parse embedded TrueType/OpenType `cmap` tables where the PDF character codes
  can be related to glyph IDs.
- Parse CFF/Type1 glyph names and PostScript `post` names where available.
- Synthesize a per-font ToUnicode-style map only when the relationship between
  PDF code, glyph ID, and Unicode is explicit enough to be deterministic.

Status: planned.

## Phase 4: OCR Fallback

- Use OCR only for visually rendered text that remains unmapped after font
  decoding.
- Keep OCR output separate from selectable text until geometry alignment and
  confidence checks pass.
- Prefer page-level OCR for scanned/image-only pages and region-level OCR for
  unmapped text spans.

Status: planned.

## Phase 5: Glyph-Shape Inference

- Use only after semantic font decoding and OCR fail or are unavailable.
- Extract glyph outlines or isolated glyph bitmaps from embedded fonts.
- Compare normalized glyph shapes against reference fonts and solve mappings
  across whole words/runs using language confidence.
- Emit diagnostics for inferred mappings and avoid silently replacing
  high-confidence decoded text.

Status: research phase; not yet a default extraction path.
