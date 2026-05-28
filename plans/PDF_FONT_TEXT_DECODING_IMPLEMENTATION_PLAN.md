# PDF Font Text Decoding Implementation Plan

Source: code review of the current PDF text decoding path on 2026-05-28.

Goal: make the converter truthfully support PDF font-based text recovery beyond
simple ToUnicode and heuristic repairs. The target behavior is:

- proper `/Encoding`, `/Differences`, and Adobe Glyph List fallback for simple
  fonts
- Type0/CID mapping through known CMap collections
- embedded font table use when available
- external OCR or page-text fallback only for truly unmapped visual text
- glyph-shape decoding for embedded-font text runs when semantic mappings fail

## Current State

Implemented:

- `/ToUnicode` CMap parsing for `beginbfchar` and `beginbfrange`
- page-local font resource CMap lookup
- simple-font fallback from `/Encoding`, `/BaseEncoding`, and `/Differences`
- WinAnsi, MacRoman, and partial StandardEncoding fallback
- compact glyph-name fallback, including `uniXXXX`, `uXXXX`, suffix stripping,
  and underscore-composed names
- conservative Identity-H/Identity-V two-byte fallback for Adobe Identity/UCS
- post-decode shifted-subset cleanup heuristics

Not implemented yet:

- complete Adobe Glyph List and full StandardEncoding/MacExpert/Symbol/
  ZapfDingbats coverage
- known CID collection mappings such as Adobe-Japan1, Adobe-CNS1, Adobe-GB1,
  and Adobe-Korea1
- embedded TrueType/OpenType/CFF/Type1 table parsing for Unicode recovery
- OCR/page-text fallback
- glyph outline or raster shape decoding

## Principles

1. Prefer deterministic PDF semantics over cleanup heuristics.
2. Treat `/ToUnicode` as authoritative when present.
3. Use fallback mappings only for missing character codes, not to override
   explicit semantic mappings.
4. Keep OCR and glyph-shape inference opt-in or clearly labeled until confidence
   gates are strong enough.
5. Emit diagnostics when text remains unmapped instead of silently returning raw
   private glyph codes.
6. Add fixture evidence before broadening a fallback path.

## Phase 1: Harden Simple Font Fallback

Files likely touched:

- `src/pdf/cmap/encoding.rs`
- `src/pdf/cmap/font_refs.rs`
- `src/pdf/cmap/tests.rs`
- `src/pdf/tests.rs`

Implementation steps:

1. Replace the hand-written glyph-name table with a generated compact AGL table.
   Keep `uniXXXX`, `uXXXX`, suffix, and underscore handling around the table.
2. Add full StandardEncoding coverage.
3. Add MacExpertEncoding if encountered in fixtures or generated tests.
4. Add Symbol and ZapfDingbats fallback tables for simple symbolic fonts.
5. Distinguish "missing mapping" from "mapped to .notdef" so callers can count
   unmapped characters.
6. Track fallback source per mapping: ToUnicode, Differences, base encoding,
   glyph name, or unknown.

Acceptance criteria:

- A simple font with `/Encoding /WinAnsiEncoding` and no `/ToUnicode` emits
  correct punctuation, bullets, and Latin-1 characters.
- A `/Differences` array maps AGL names and `uniXXXX` names correctly.
- `/ToUnicode` still wins over base encoding and differences.
- Unknown glyph names do not become raw bytes in semantic output.

Tests:

- Add unit tests for `quoteright`, `quoteleft`, `endash`, `emdash`, `fi`,
  `fl`, `bullet`, `.notdef`, `uniFB01`, and underscore-composed names.
- Add PDF fixture tests for WinAnsi, StandardEncoding, Symbol, and
  ZapfDingbats fallbacks.

## Phase 2: Implement Known Type0/CID Collections

Files likely touched:

- `src/pdf/cmap.rs`
- `src/pdf/cmap/font_refs.rs`
- new `src/pdf/cmap/predefined.rs`
- new generated data under `src/pdf/cmap/generated/`

Implementation steps:

1. Parse Type0 `/Encoding` names beyond Identity-H and Identity-V.
2. Parse descendant `/CIDSystemInfo` registry, ordering, and supplement.
3. Add predefined CMap data for the common Adobe collections:
   Adobe-Japan1, Adobe-CNS1, Adobe-GB1, and Adobe-Korea1.
4. Support horizontal and vertical variants as character-code-to-CID decoders.
5. Keep CID-to-Unicode mapping separate from Identity Unicode fallback.
6. Add diagnostics for unsupported CID collections or supplements.
7. Prevent the current Identity fallback from treating arbitrary CIDs as Unicode
   unless the font is explicitly Adobe-UCS or another verified Unicode CID
   collection.

Acceptance criteria:

- Type0 fonts with known predefined CMaps decode sample Japanese, Chinese, and
  Korean text when no `/ToUnicode` is present.
- Identity-H with Adobe-Identity no longer claims arbitrary CID values are
  Unicode text unless supported by a font table or ToUnicode.
- Unsupported collections produce warnings and unmapped counters.

Tests:

- Unit tests for CMap code-width parsing and multi-byte character consumption.
- Fixture PDFs for one known collection per supported ordering.
- Regression test proving `Adobe-Identity` CID 0x0041 is not blindly decoded as
  `A` unless the collection is known to be Unicode-safe.

## Phase 3: Use Embedded Font Tables

Files likely touched:

- new `src/pdf/fonts/embedded.rs`
- `src/pdf/fonts.rs`
- `src/pdf/cmap/font_refs.rs`
- `Cargo.toml`

Implementation steps:

1. Resolve `/FontDescriptor` from simple fonts and CID descendant fonts.
2. Extract `/FontFile`, `/FontFile2`, and `/FontFile3` streams with filters.
3. Add a font parser dependency after a small spike. Candidate capabilities:
   TrueType/OpenType `cmap`, `post`, glyph IDs, CFF charsets, and glyph outlines.
4. For TrueType/OpenType:
   - read Unicode `cmap` subtables
   - read glyph-name data from `post` where available
   - connect PDF code to glyph ID through explicit encodings, CIDToGIDMap, or
     embedded simple-font encodings
5. For CFF/Type1:
   - read charsets or glyph names
   - map glyph names through AGL
6. Synthesize a ToUnicode-style map only when the PDF code to glyph relation is
   deterministic.
7. Store confidence and source metadata for each synthesized mapping.

Acceptance criteria:

- Embedded fonts improve decoding only when table evidence establishes the
  mapping.
- No fallback guesses override `/ToUnicode`.
- Missing or encrypted font streams produce warnings, not panics.

Tests:

- Fixture with embedded TrueType `cmap` and no `/ToUnicode`.
- Fixture with CFF glyph names and no `/ToUnicode`.
- Fixture with `/CIDToGIDMap` and embedded TrueType table.
- Negative fixture where embedded font data exists but PDF code to glyph ID is
  ambiguous; expected result is unmapped diagnostics.

## Phase 4: External OCR And Page-Text Fallback

Files likely touched:

- `src/pdf/mod.rs`
- new `src/pdf/ocr.rs`
- CLI options in `src/main.rs` or `src/cli.rs`
- tests around warnings and placeholders

Implementation steps:

1. Add an explicit conversion option for OCR or external page-text fallback.
2. Define an adapter trait for external tools so tests can use a fake provider.
3. Support page-level fallback for image-only pages.
4. Support region-level fallback for visual text runs whose font mapping remains
   unmapped after Phases 1-3.
5. Align OCR text by page and bounding box before merging it into output.
6. Require confidence thresholds and overlap checks to avoid duplicating
   selectable text.
7. Preserve current behavior when OCR is disabled: warn that OCR is not
   performed.

Acceptance criteria:

- Default conversion remains deterministic and does not shell out.
- With OCR enabled, image-only pages can produce text blocks with source
  metadata indicating OCR.
- OCR text does not duplicate existing selectable text.
- Low-confidence OCR remains a warning or placeholder.

Tests:

- Unit tests with a fake OCR provider.
- Fixture for image-only page with fake OCR text.
- Fixture where selectable text and OCR overlap; expected result has no
  duplicate paragraph.
- Fixture where OCR confidence is too low; expected result remains placeholder
  plus warning.

## Phase 5: Glyph-Shape Decoding For Embedded Font Runs

Files likely touched:

- new `src/pdf/fonts/shapes.rs`
- new `src/pdf/text/glyph_inference.rs`
- `src/pdf/text/parser.rs`
- visual diagnostics in `src/pdf/visual`

Implementation steps:

1. Capture unresolved text runs with:
   - font reference
   - raw bytes
   - glyph IDs if known
   - positions and advances
   - page/run context
2. Extract glyph outlines or normalized raster masks from embedded fonts.
3. Normalize shapes for scale, translation, winding, and minor hinting
   differences.
4. Build reference comparisons for common Latin glyphs first.
5. Solve mappings at run level, not one glyph at a time, using:
   - repeated glyph consistency
   - word-shape constraints
   - language score
   - nearby mapped text
   - font width evidence
6. Mark inferred text as low/medium/high confidence.
7. Gate semantic output on confidence. Low-confidence results should be exposed
   only in diagnostics or visual overlays.

Acceptance criteria:

- A subset font with no `/ToUnicode`, no usable `/Encoding`, and embedded
  outlines can recover a small Latin phrase when the glyph shapes are clear.
- Ambiguous shapes such as `I`, `l`, `1`, `O`, and `0` require context or remain
  unresolved.
- Shape inference never runs before deterministic mapping sources.

Tests:

- Synthetic embedded subset font with remapped glyph codes for a known phrase.
- Ambiguity test for `I/l/1` and `O/0`.
- Regression test that real `/ToUnicode` output is not changed by shape
  inference.

## Phase 6: Diagnostics And Quality Gates

Implementation steps:

1. Add unmapped-character counters per page and font.
2. Add warnings for unsupported encodings, unknown CID collections, missing font
   streams, ambiguous embedded font mappings, and OCR-disabled image text.
3. Add debug-only mapping traces for fixtures.
4. Keep post-decode shifted-subset repairs behind clear score gates.
5. Regenerate affected output fixtures after each phase.

Quality gates after each phase:

1. Run `cargo test`.
2. Run fixture conversion tests.
3. Compare representative PDFs visually and semantically.
4. Review warnings for accuracy.
5. Remove or narrow any heuristic that improves one fixture by damaging another.

## Suggested Work Order

1. Harden AGL and simple encoding coverage.
2. Add unmapped diagnostics.
3. Replace unsafe Type0 Identity behavior with known-collection handling.
4. Add embedded font extraction and table parsing.
5. Add opt-in external OCR adapter.
6. Add glyph-shape inference as an experimental, diagnostic-heavy path.

## Definition Of Done

The feature set is complete only when the converter can demonstrate all five
capabilities with tests and diagnostics:

- simple-font `/Encoding` and `/Differences` fallback through AGL
- known Type0/CID collection decoding
- embedded font table recovery where deterministic
- OCR/page-text fallback for genuinely unmapped visual text
- glyph-shape decoding for embedded-font runs with confidence gates

Until then, user-facing descriptions should say that only simple-font fallback
and limited Identity Type0 fallback are currently implemented.
