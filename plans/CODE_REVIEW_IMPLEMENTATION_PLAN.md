# Code Review Implementation Plan

Source review: `plans/CODE_REVIEW.html`

This plan turns the code review into an implementation workflow. It does not
permit deferring review ideas. Every idea must be attempted, measured, and
closed with evidence.

## Operating Rules

1. Do not write broad CSS.
2. Prefer semantic HTML elements and attributes.
3. Use minimal CSS only where HTML cannot solve the readable-output problem.
4. Any CSS exception must be narrowly scoped, documented, and must not recreate
   PDF page positioning.
5. Every idea from `CODE_REVIEW.html` must receive an implementation attempt.
6. After each attempt, regenerate the affected output files and run
   `pdf-web-compare` against the matching files in `input/`.
7. Keep the implementation only when the comparison shows that the new output is
   closer to the input.
8. If an attempt does not improve the comparison, revert that attempt and record
   the measured result. This still counts as attempted, not deferred.
9. Run a rust-quality-lens quality cycle after retained implementation batches.
10. Continue until every code-review idea is either retained with comparison
    evidence or attempted and reverted with comparison evidence.

## Baseline

1. Snapshot the current `output/` directory.
2. Run `pdf-web-compare` for every PDF in `input/` against the current HTML file
   with the same stem in `output/`.
3. Save the baseline reports under a comparison run directory.
4. Record baseline observations per file:
   - visual similarity
   - text extraction quality
   - title and language correctness
   - headings, page breaks, lists, and tables
   - warnings and placeholders

## Batch 1: Renderer Contract And Metadata

Attempt all of the following:

1. Remove the renderer's always-emitted default `<style>` block.
2. Replace the misleading CSS renderer test with an assertion that broad/default
   CSS is absent.
3. Add the revised output contract: minimal CSS is allowed only when semantic
   HTML cannot express the needed readable structure.
4. Extract PDF `/Info /Title` into `DocumentMetadata.title`.
5. Decode PDF metadata strings consistently, including literal strings, hex
   strings, UTF-16BE with BOM, and PDFDocEncoding where needed.
6. Reuse the metadata string decoder for `/Lang`.
7. Prevent filename leaks from becoming titles when a better PDF title is not
   present.
8. Emit `Block::PageBreak` between pages when the PDF has multiple pages.

Comparison gate:

1. Regenerate every file in `output/`.
2. Run `pdf-web-compare` for every `input/` PDF and matching `output/` HTML.
3. Keep only changes that improve or preserve comparison quality without
   violating the revised CSS contract.
4. Revert non-improving attempts and record why they were not kept.

Quality gate:

1. Run the Rust test suite.
2. Run rust-quality-lens `catalog`.
3. Run rust-quality-lens focused measures for changed areas, or `measure all`
   if the batch touched shared PDF or HTML behavior.
4. Address worthwhile findings before moving to Batch 2.

## Batch 2: Unicode And Word Boundaries

Attempt all of the following:

1. Implement encoding fallbacks for fonts without `/ToUnicode`.
2. Cover Differences arrays where present.
3. Add WinAnsi, MacRoman, and StandardEncoding fallbacks where applicable.
4. Use font descriptor and font naming evidence when choosing fallback behavior.
5. Replace hard-coded `postprocess::REPAIRS` string substitutions with a real
   word-gap classifier.
6. Drive the gap classifier from measured font widths, glyph advances, body
   font size, and line context.
7. Remove per-fixture word-boundary repair behavior from core conversion.
8. Add fixtures that prove the new classifier handles the review's visible
   failures, including joined words and wrongly split words.

Comparison gate:

1. Regenerate every file in `output/`.
2. Run `pdf-web-compare` across all input/output pairs.
3. Keep only word-boundary and encoding changes that improve the comparison.
4. Revert any heuristic that fixes one sample by damaging another, then attempt
   a narrower version of that heuristic.

Quality gate:

1. Run the Rust test suite.
2. Run rust-quality-lens with attention to PDF layout, fonts, cmap, and
   postprocess modules.
3. Reduce newly introduced hotspots before moving to Batch 3.

## Batch 3: Tables, Lists, And Headings

Attempt all of the following:

1. Remove the `message_item_header` / `synthetic_message_item_header` core
   pipeline shim.
2. Replace that shim with general table-header detection based on column
   geometry, repeated header evidence, font evidence, and tagged-PDF evidence
   where available.
3. Improve table parsing for wrapped cells, numeric alignment, and table-vs-TOC
   distinction.
4. Recover list markers when extraction loses or normalizes PDF marker glyphs.
5. Support common bullet glyphs and nested marker shapes.
6. Tighten heading classification so font size alone is not enough.
7. Require extra heading evidence such as isolation, short line length, no
   terminal sentence punctuation, tagged heading role, bold font evidence, or
   repeated document-title context.
8. Consolidate split multi-line headings and document titles when comparison
   improves.
9. Add fixtures for heading false positives, bullet glyphs, repeated
   header/footer removal, and multi-column prose versus tables.

Comparison gate:

1. Regenerate all output.
2. Run `pdf-web-compare` across all input/output pairs.
3. Keep only structure changes that improve visual/text comparison.
4. Rework and retry any structure heuristic that causes regressions.

Quality gate:

1. Run the Rust test suite.
2. Run rust-quality-lens for layout, postprocess, and model/rendering impact.
3. Address worthwhile findings before moving to Batch 4.

## Batch 4: Tagged PDF, Alt Text, Duplicates, And Warnings

Attempt all of the following:

1. Walk `/StructTreeRoot` for tagged-PDF support beyond marked-content roles.
2. Apply role maps where present.
3. Use structure-tree data for headings, table headers, spans, and artifacts
   when it improves output.
4. Extract `/Alt` text from structure data for images or figures where present.
5. Preserve `ActualText` behavior and add fixtures for spans that use it.
6. Suppress overlapping duplicate text segments when they are identical and
   comparison improves.
7. Add page-specific warning attribution.
8. Add warnings for CMap decode failures.
9. Add unsupported-filter warnings with stream/filter detail.
10. Add or improve image-only page placeholders and warnings.
11. Add fixtures for incremental updates, xref streams, object streams,
    multi-byte CMaps, ActualText spans, image-only pages, repeated
    header/footer, and unsupported filters.

Comparison gate:

1. Regenerate all output.
2. Run `pdf-web-compare` across all input/output pairs.
3. Keep semantic enhancements only when they improve or preserve comparison and
   do not reduce useful extracted text.
4. Revert failed attempts, record evidence, and attempt a narrower
   implementation before closing the idea.

Quality gate:

1. Run the Rust test suite.
2. Run rust-quality-lens `measure all`.
3. Address worthwhile findings.
4. Repeat the quality cycle until retained changes are clean enough to close.

## Final Closure Checklist

1. Every idea in `CODE_REVIEW.html` has an entry in the implementation log.
2. Each entry is marked either:
   - retained with `pdf-web-compare` evidence, or
   - attempted, measured, reverted, and replaced by a narrower attempt or
     closed with evidence that no retained version improves comparison.
3. No idea is marked deferred.
4. The revised CSS policy is represented in docs and tests.
5. The final `output/` files are regenerated from the retained implementation.
6. Final `pdf-web-compare` reports exist for every file in `input/`.
7. Final Rust tests pass.
8. Final rust-quality-lens results have been reviewed and worthwhile findings
   have been addressed or recorded as intentionally accepted technical debt.
