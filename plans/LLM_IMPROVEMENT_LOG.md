# LLM Improvement Log

## Cycle 1

Failure class:

Generated Office filenames and watermark words were being promoted from PDF
metadata into HTML document titles and synthetic article headings.

Target PDFs:

`DREU20210303IN01.pdf`, `IAF-Dimitri.pdf`, and `IEC 61000-3-2 2018.pdf`.

Acceptance check:

`DREU20210303IN01.html` must not contain `Microsoft Word -`.
`IAF-Dimitri.html` must not use `IAF210906-1.xlsx` as its title.
`IEC 61000-3-2 2018.html` must not use `English` as its document title or
synthetic article heading.

Files changed:

`src/pdf/mod.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` passed.
- regenerated outputs: `output/DREU20210303IN01.html`,
  `output/IAF-Dimitri.html`, and `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packets: `compare/cycle-1-before/*` and
  `compare/cycle-1-after/*`.
- LLM visual review: before/after side-by-side visual pages showed no page
  rendering regression. The improvement is metadata/header cleanup, verified in
  the HTML text because the affected title is not visible in the default
  collapsed visual page screenshot.

## Cycle 2

Failure class:

HCE NDA uses a shifted subset font where punctuation-heavy cipher runs were not
being repaired, leaving the document largely unreadable.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`.

Acceptance check:

The visual packet should show the HCE title and at least some body phrases
moving from cipher text toward readable English without deleting document
content.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, with residual follow-up required.

Evidence:

- tests: `cargo test` passed.
- regenerated outputs: `output/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.html`.
- pdf-web-compare packets: `compare/cycle-2-before/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`
  and `compare/cycle-2-after/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`.
- LLM visual review: the before packet showed a cipher title and large unreadable
  headings. The after packet shows `MUTUALCONFIDENTIALITY` and
  `andtechniqueasitrelatesto` as readable text, but still has large layout
  overlap and many unrepaired mixed runs. Keep the improvement, and continue
  HCE in a later cycle focused on line order and mixed CMap runs.

## Cycle 3

Failure class:

DREU contained decorative symbol-font glyphs such as `}uW`, `C}uW`, and
`}v]]}vW` in both the reconstructed page and extracted article.

Target PDFs:

`DREU20210303IN01.pdf`.

Acceptance check:

The visual packet should remove decorative glyph clutter while preserving real
content such as customer name, contact name, pricing table, general terms, and
attachments.

Files changed:

`src/pdf/text/emitter.rs`, `src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` passed.
- regenerated outputs: `output/DREU20210303IN01.html`.
- pdf-web-compare packets: `compare/cycle-3-before/DREU20210303IN01` and
  `compare/cycle-3-after/DREU20210303IN01`.
- LLM visual review: the before packet showed icon-glyph garbage before the
  customer/contact fields and before the terms section. The after packet removes
  that clutter and keeps the meaningful text and tables visible.

## Cycle 4

Failure class:

The HCE shifted-subset repair over-shifted already-readable chunks in mixed
encoded/plain title text, producing `MUTUALCONFIabkqfALITY`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`.

Acceptance check:

The HCE title should render as `MUTUALCONFIDENTIALITY`, not as a partially
shifted hybrid.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` passed.
- regenerated outputs: `output/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.html`.
- pdf-web-compare packet: `compare/cycle-4-after/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`.
- LLM visual review: the after packet shows the title text as
  `MUTUALCONFIDENTIALITY`. Large overlap and remaining cipher text are still
  present and should be addressed by a later layout/decode cycle.

## Cycle 5

Failure class:

HCE still contained shifted-subset party/address text such as `5RDG`,
`%LQ-LDQJ`, `+DQJ]KRX`, `&KLQD`, and `UHSUHVHQ` after the earlier title repair.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`.

Acceptance check:

The HCE visual packet should change the party/address line from visible cipher
tokens toward readable text without regressing the already-repaired title.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-5-before/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`
  and `compare/cycle-5-after/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`.
- LLM visual review: the before packet showed the HCE party/address line as
  cipher text (`5RDG %LQ-LDQJ ...`). The after packet shows that same line as
  `Road BinJiang Dist Hangzhou China and their represen...`. The title remains
  `MUTUALCONFIDENTIALITY`. Major layout overlap and additional cipher fragments
  remain for later cycles.

## Five-Cycle Quality Run

Scope:

Ran the required quality pass after five retained improvement cycles.

Files changed:

`src/pdf/mod.rs`.

Result:

kept.

Evidence:

- commands: `python -m rust_quality_lens.cli catalog --config rqlens.toml` and
  `python -m rust_quality_lens.cli measure all --config rqlens.toml`.
- tests: `cargo test` and `cargo check --all-targets` passed after the quality
  edit.
- clone improvement: extracted repeated one-page PDF metadata fixture setup in
  `src/pdf/mod.rs` tests. The previous top clone group
  `src/pdf/mod.rs:394-398`, `413-417`, `435-439`, `454-458`, and `488-492`
  is no longer in the top clone report.
- current hotspot note: `src/pdf/text/strings.rs` is now the top hotspot
  (`score=625.11`, `cognitive=56`, `cyclomatic=132`) because the retained HCE
  decoder improvement added heuristic branches. It should be the next quality
  target once another functional cycle is complete.
- current clone note: the top remaining clone group is now in
  `src/pdf/layout_tests.rs`.
- rqlens warning: leverage AST style analysis completed with a warning because
  Cargo could not open
  `C:\Code\rust-quality-lens\rust_helpers\target\debug\.cargo-lock`
  (`Access is denied`). The overall `measure all` command still completed and
  wrote the expected artifacts.

## Cycle 6

Failure class:

HCE page text state was reset between page content streams. A later stream
continued with a scaled text matrix but no fresh `Tf`, so the parser used its
default 12pt font and rendered normal body fragments as 48pt-clamped oversized
text.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`.

Acceptance check:

The HCE visual packet should remove the huge overlapping body fragments such as
`andtechniqueasitrelatesto`, `incurredinconnectionwithallsuchlitigation`, and
`HCELegal`, while preserving the text and the cycle-5 readable address line.

Files changed:

`src/pdf/mod.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added `carries_text_state_across_page_content_streams`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-6-after/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`
  and regression sample `compare/cycle-6-sample/DREU20210303IN01`.
- LLM visual review: the prior retained HCE packet had massive overlapping
  48pt text across the middle and bottom of page 1. The cycle-6 packet renders
  those same fragments at normal body scale (`10.02pt` or `7.98pt`), leaving the
  page visibly less obstructed. Text remains incomplete and partially ciphered,
  but the targeted font-size/state defect is improved.

## Cycle 7

Failure class:

Generated PDF HTML exposed page sections and `data-page-break` markers, but the
CSS did not ask browsers or print renderers to treat them as page boundaries.
Long documents such as `Digital Dimming V2.0 Communication Protocol Rev. A.pdf`
therefore read as one continuous flow in the web comparison.

Target PDFs:

`Digital Dimming V2.0 Communication Protocol Rev. A.pdf`, with all PDF outputs
regenerated because the change is shared HTML rendering behavior.

Acceptance check:

Generated PDF HTML should contain explicit screen page gaps and print page-break
rules for each `.pdf-recreated-page`, plus page-break behavior for extracted
content `hr[data-page-break]` markers.

Files changed:

`src/html/mod.rs`, `side-projects/pdf-web-compare/pdf_web_compare_app.py`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: `renders_pdf_reconstructed_html_with_minimal_visual_styles`
  now checks for `break-after: page`, `hr[data-page-break]`, and `@media print`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packet: `compare/cycle-7-after/Digital Dimming V2.0 Communication Protocol Rev. A`.
- LLM visual review: the generated HTML now has explicit `.pdf-recreated-page`
  page-break CSS and `hr[data-page-break]` print breaks, so browser/print output
  has page boundaries instead of only implicit stacked sections.
- follow-up fix: screen layout no longer adds vertical page gaps that confuse
  `pdf-web-compare` slicing, and `pdf-web-compare` now screenshots webpages at
  `dpi / 96` device scale so CSS points align with PDF render pixels. Verified
  with `compare/cycle-7-fix/Digital Dimming V2.0 Communication Protocol Rev. A`,
  where web slice 1 shows `Hardware Interface Design` instead of drifting into
  later pages.

## Cycle 8

Failure class:

Embedded PDF images were skipped during normal PDF conversion, leaving logos,
schematics, and other image XObjects absent from the standalone HTML visual
layer.

Target PDFs:

All PDFs in `input/`, with visual focus on `Digital Dimming V2.0 Communication
Protocol Rev. A.pdf` and `XML-Message-for-SCT-Version-7.0-February-2013-1.pdf`.

Acceptance check:

Default PDF conversion should embed extracted image XObjects as data URIs in
the generated HTML, while an explicit opt-out remains available.

Files changed:

`src/pdf/mod.rs`, `src/cli.rs`, `README.md`,
`side-projects/pdf-web-compare/README.md`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added default-image and `--no-images` coverage.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-8-before/*`,
  `compare/cycle-8-after/*`.
- LLM visual review: `Digital Dimming V2.0 Communication Protocol Rev. A` page
  1 now shows the Inventronics logo and circuit diagrams in the web render. The
  before packet showed only text and a small rectangle placeholder in the same
  regions. `XML-Message-for-SCT-Version-7.0-February-2013-1` also preserves the
  cover logo as standalone embedded image data.

## Cycle 9

Failure class:

Some PDFs draw form/table structure with axis-aligned path strokes instead of
`re` rectangle operators, so the visual layer missed those lines.

Target PDFs:

`IAF-Dimitri.pdf`, with all PDFs regenerated because graphics extraction is
shared behavior.

Acceptance check:

Axis-aligned `m/l/h` path strokes and simple filled rectangular paths should
produce positioned visual shapes without relying on rendered PDF pages.

Files changed:

`src/pdf/graphics.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added stroked path line and filled path rectangle tests.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packet: `compare/cycle-9-after/*`.
- LLM visual review: `IAF-Dimitri` page 1 now shows the part-number field line
  in the web render, which was absent in cycle 8. The cycle does not yet recover
  most form boxes, yellow highlights, or signature strokes; the next cycle
  should inspect the remaining path operators or resource state used by that
  form.

## Cycle 10

Failure class:

PDF content streams can paint several closed rectangular subpaths in one `f*`
operation. The graphics extractor treated the whole operation as one contour, so
`IAF-Dimitri` still missed most form boxes and yellow highlights.

Target PDFs:

`IAF-Dimitri.pdf`, with all PDFs regenerated because graphics extraction is
shared behavior.

Acceptance check:

Each closed axis-aligned subpath inside a fill operation should become its own
positioned visual rectangle.

Files changed:

`src/pdf/graphics.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added transformed even-odd multi-subpath rectangle
  coverage based on the `IAF-Dimitri` stream pattern.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-10-focus/IAF-Dimitri`,
  `compare/cycle-10-after/*`.
- LLM visual review: `IAF-Dimitri` page 1 now shows the outer frame, form field
  boxes, grid lines, and yellow selected rows. The remaining obvious gap is the
  blue signature, which is not page content but `/Ink` annotations.
- quality notes: fifth-cycle checkpoint run after this cycle. RQL artifacts were
  refreshed under `target/analysis`: hotspots, clones, type health, locality,
  leverage, and map. Top hotspots are `src/pdf/images.rs`,
  `src/pdf/graphics.rs`, `src/pdf/text/strings.rs`, and `src/pdf/streams.rs`.
  No refactor was attempted inside the visual cycle because the next retained
  improvement was still a narrow correctness gap.

## Cycle 11

Failure class:

Ink annotations were ignored. `IAF-Dimitri` stores the blue signature as three
`/Subtype /Ink` annotations rather than page content, so the standalone HTML
missed the signature even after form graphics were recovered.

Target PDFs:

`IAF-Dimitri.pdf`, with all PDFs regenerated because page visual rendering is
shared behavior.

Acceptance check:

Ink annotation `/InkList` points should render as standalone SVG polylines with
the annotation stroke color and width, without embedding or loading the source
PDF.

Files changed:

`src/pdf/streams.rs`, `src/pdf/mod.rs`, `src/pdf/visual.rs`,
`src/html/mod.rs`.

Result:

kept.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added ink annotation extraction and SVG visual rendering
  tests.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-11-focus/IAF-Dimitri`,
  `compare/cycle-11-after/*`.
- LLM visual review: `IAF-Dimitri` page 1 now shows the blue signature in the
  same lower-center region as the PDF, rendered from annotation points as SVG
  paths. The page is still not indistinguishable because typography and some
  text alignment differ, but the targeted missing-signature gap is closed.

## Cycle 12

Failure class:

Some HCE NDA text uses a shifted subset encoding after CMap decoding, leaving
visible terms such as `RIILFHV`, `UHIHUUHGWRDV`, and long mixed-case fragments
in the standalone output.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated after the text repair was retained.

Acceptance check:

Shifted subset text should be repaired in generated HTML without relying on the
source PDF at runtime.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`, `src/pdf/text.rs`,
`src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed in the following
  cycle checkpoint.
- focused regressions: added HCE shifted-subset repair coverage, including
  mixed title-case terms and final visual-render repair coverage.
- focused packet: `compare/cycle-12-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`.
- LLM visual review: HCE page 1 now repairs several visible shifted strings,
  including `having offices at Hillington`, `INVENTRONICS`, `Name`, and the
  `AsusedinthisMutualConfidentialityAgreement,Disclosin` fragment. Remaining
  HCE gaps are spacing, word boundaries, and many still-shifted body fragments.

## Cycle 13

Failure class:

HCE body text existed in extracted content but was mostly missing from the
positioned visual layer because relative text moves were not modeled against the
PDF text line matrix and active text matrix.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text positioning is shared behavior.

Acceptance check:

Relative `Td`/`TD` movement after text should start from the text line matrix
and use the active text transform, restoring off-page visual fragments while
preserving existing flipped and scaled text behavior.

Files changed:

`src/pdf/text/state.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added coverage for relative text moves starting from the
  line matrix and using active text-matrix scale.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-13-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-13-after/*`.
- LLM visual review: HCE page 1 now renders substantially more body text in the
  page layer instead of omitting it, but the result is not visually
  indistinguishable. The next loop should focus on HCE word-boundary recovery,
  glyph spacing, and remaining shifted-subset terms now exposed by the restored
  fragments.

## Cycle 14

Failure class:

After cycle 13 restored HCE body text to the positioned visual layer, many newly
visible fragments still showed shifted subset words and printable shifted runs,
including legal prose such as `SDUWLHVPD\H[FKDQJHFHUWDLQF`,
`UHTXLVLWLRQV SURFHVV LQIRUPDWLRQ`, and `EZDV NQRZQ WR ReceiviQJ`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

Printable shifted subset runs containing punctuation such as `\` and `[` should
decode as a unit when the decoded candidate contains clear legal-document
vocabulary, while preserving literal structural text elsewhere.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: extended HCE shifted-subset coverage for printable
  shifted runs, mixed decoded/encoded fragments, and short legal words such as
  `WR`, `DV`, and `IURP`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-14-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-14-after/*`.
- LLM visual review: HCE page 1 now shows additional readable legal prose such
  as `partiesmayexchangecertainc`, `requisitions process information
  instructions test results`, and `known to Receiving Party as evidenced by
  written`. The page is still not visually indistinguishable: many words remain
  joined, several terms are still partially shifted, and glyph spacing/line
  density still diverge from the source PDF.

## Cycle 15

Failure class:

HCE page 1 still had high-confidence shifted legal vocabulary after cycle 14,
including `THEREFORE`, confidential-information clauses, party labels, and
all-caps warranty text.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

Common shifted legal terms and all-caps warranty clauses should decode into the
standalone HTML without relying on the original PDF at runtime.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `THEREFORE in consideration`,
  `confidential nature`, `furnished`, `officers`, `employees`, `including but
  not limited`, and all-caps warranty clauses.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-15-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-15-after/*`.
- LLM visual review: HCE page 1 now shows readable `THEREFORE in
  consideration`, `confidential nature of such information furnished by`, and
  `NO OTHER WARRANTIES ARE MADE BY EITHER PARTY UNDER THIS`. The result is still
  not visually indistinguishable: word joins, residual shifted terms, line
  density, glyph spacing, and layout drift remain.

## Cycle 16

Failure class:

HCE page 1 still had mixed shifted/literal legal terms after cycle 15, including
`DFFRXQWDnts`, `DJHnts`, `GRFXPHnts`, `UHODWLQJ`, `agreemeQW`, and a split
all-caps warranty continuation line.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

Mixed shifted/literal HCE legal terms should decode in the standalone HTML, and
the warranty continuation should read as English without relying on the original
PDF at runtime.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Rejected candidate:

Splitting every `TJ` adjustment into separate visual fragments was tested and
rejected because HCE uses dense glyph-level adjustments; the focused visual
packet scattered text across the page and was worse than cycle 15. That
experimental parser/test change was removed before acceptance.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `accountants`, `agents`,
  `documents`, `relating`, `destroyed`, `available`, `remain`, `this`,
  `agreement`, and the all-caps warranty continuation.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-16-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-16-after/*`.
- LLM visual review: HCE page 1 now repairs visible fragments such as
  `documents`, `accountants`, `relating`, `Agreement`, and the second warranty
  line now reads closer to `AGREEMENT. ANY INFORMATION EXCHANGED UNDER THIS
  AGREEMENT IS P`. The result is still not visually indistinguishable: the page
  has joined words, remaining shifted fragments, horizontal density mismatch,
  and layout drift.

## Cycle 17

Failure class:

HCE page 1 still had visible shifted clause vocabulary after cycle 16,
including `SURPSWO\XSRQUHTXHVW`, `GHSHQGHQWO\ GHYHORSHG`, `Zithout`,
`QVWUXHGLQDFFRUGDQFHwiththe`, `VXUYLYH`, `FRQFOXVLRQ`, `EHWZHHQ`, `H[SHQVH`,
and `FRSLHV`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

Remaining HCE legal clause vocabulary should decode in the standalone HTML
without relying on the original PDF at runtime, while preserving prior repairs
such as `known`.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `promptlyuponrequest`,
  `independently developed`, `without`, `construedinaccordancewiththe`, `each`,
  `set forth`, `survive`, `conclusion`, `between`, `own`, `expense`, and
  `copies`; avoided a broad `RZQ` replacement that broke `known`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-17-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-17-after/*`.
- LLM visual review: HCE page 1 now shows improvements around the middle and
  lower clauses, including `independently developed`, `without`, and
  `promptlyuponrequest`. The result is still not visually indistinguishable:
  word joins, remaining shifted fragments, broken line density, and layout drift
  remain.

## Cycle 18

Failure class:

HCE page 1 paragraph 6 still had visible shifted legal terms after cycle 17,
including `PHDQVRIDGHSRVLWLRQVXESRHQD`, `SHUPLWWHG`, `ODZ`, `FRRSHUDWH`,
`HIIRUWV`, `SUHYHQW`, `LQZULWLQJRUE`, `HFHLYLQJParty`, and
`eitherPartyLVUHFHLYLQJL`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

Paragraph 6 legal vocabulary should decode in the standalone HTML without
relying on the original PDF at runtime, while preserving prior HCE repairs.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `eitherPartyisreceivingi`,
  `ReceivingParty`, `meansofadepositionsubpoena`, `permitted`, `law`,
  `cooperate`, `efforts`, `prevent`, and `Partyinwritingorby`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-18-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-18-after/*`.
- LLM visual review: HCE page 1 paragraph 6 now shows readable phrases such as
  `meansofadepositionsubpoena`, `permitted under applicable law shall
  cooperate`, `efforts`, and `prevent`. The result is still not visually
  indistinguishable: joined words, residual shifted fragments, density mismatch,
  and layout drift remain.

## Cycle 19

Failure class:

HCE page 1 still had stable visible shifted fragments after cycle 18, including
the top-page `&27/$1'` party fragment, `ILQ DQFLDO DGYLVRUV`, export-law
fragments such as `lawVUHJXODWLR` and `EOHlaws`, and lower litigation/cost
fragments such as `breachHGbyRecei YLQJParty`, `expenseV`, `own H [SHQVH`,
`5eceiving`, and `LYLQJParty`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

These stable party, advisor, export-law, and litigation-cost fragments should
decode in the standalone HTML without relying on the original PDF at runtime,
while preserving prior HCE repairs.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `SCOTLAND`, `financial advisors`,
  `lawsregulatio`, `blelaws`, `breachedbyReceivingParty`, `expenses`,
  `own expense`, `Receiving`, and `ivingParty`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-19-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-19-after/*`.
- LLM visual review: HCE page 1 now shows the top party fragment as
  `HUBBELL SCOTLAND` and improves several lower-clause fragments. The result is
  still not visually indistinguishable: many words are joined, some encoded
  fragments remain, and the page layout/density still diverges substantially.

## Cycle 20

Failure class:

HCE page 1 still had visible shifted fragments after cycle 19, including
`AGREEME17`, `hePaUWLHVKHUHbyagreetothefollowing`,
`rkedasconfidentialwithLQ`, split `ILQ DQFLDO DGYLVRUV`, data/manual/machine
fragments, `deliverHG`, `DZKRQHHGW`, `KHWHUPVRI`, `UHVWULFWL`, `DWKLUG`,
`HEHQHIL`, `SURY`, `RIZKL`, and `FRS\VHQWbyHPDLOandsXFKI`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because text repair is shared behavior.

Acceptance check:

These stable title, clause, advisor, sample-list, and lower-agreement fragments
should decode in the standalone HTML without relying on the original PDF at
runtime, while preserving prior HCE repairs.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added HCE coverage for `AGREEMENT`,
  `thePartiesherebyagreetothefollowing`, `markedasconfidentialwithin`,
  `financial advisors`, `data`, `manuals`, `machines`, `samples`, `made`,
  `delivered`, `whoneedt`, `thetermsof`, `restricti`, `athird`, `benefi`,
  `prov`, `ofwhi`, and `copysentbyemailandsuchf`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-20-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`,
  `compare/cycle-20-after/*`.
- LLM visual review: HCE page 1 now has a cleaner title, readable
  `thePartiesherebyagreetothefollowing`, improved confidentiality/advisor
  fragments, and clearer data/manuals/machines/samples text. The result is
  still not visually indistinguishable: text remains joined, residual encoded
  fragments remain, and layout/density drift is still large.

## Cycle 21

Failure class:

DREU quotation layout capture was poor. Page 1 lost key commercial labels
(`Customer:`, `Date:`, `Quote No.:`, `From:`, `SUBJECT:`, and
`Control Gear Pricing:`) and showed encoded VAT/date/quote/price/IBAN values
such as `kiURSNOOVUN_MN`, `OMOOJMQJNO`, `aobrOMONMPMPfkMN`, `rpANVKNM`,
`rpANUKQR`, `rpAOOKPN`, `rpAONKRR`, and `kiPN`. Page 2 lost the delivery,
pricing, and table-header text.

Target PDFs:

`DREU20210303IN01.pdf`, with all PDFs regenerated because text repair and PDF
visual rendering are shared behavior.

Acceptance check:

The DREU quote values and recurring section/table labels should appear in the
standalone HTML output and visual render without relying on the original PDF at
runtime, while preserving prior HCE shifted-subset repairs.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`, `src/pdf/mod.rs`,
`src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added DREU coverage for custom quote values, including
  VAT, date, quote number, table prices, and IBAN prefix.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-21-focus/DREU20210303IN01`,
  `compare/cycle-21-focus-2/DREU20210303IN01`, and `compare/cycle-21-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: DREU page 1 now shows decoded VAT/date/quote/prices/IBAN
  and restored subject/control-pricing labels. DREU page 2 now restores the
  delivery/pricing text, `STANDARD`, `LT`, `Premium`, `Special arrangement`,
  and air-shipping labels. The result is still not visually indistinguishable:
  font metrics, page density, table positioning, and some label/value spacing
  still drift from the input.

## Cycle 22

Failure class:

DREU visual text used the browser default serif face, while the quote input is a
Word-origin sans-serif document. This made restored labels readable but still
visually unlike the input.

Target PDFs:

All PDFs, because PDF visual CSS is shared behavior.

Acceptance check:

The PDF visual layer should use a sans-serif family so standalone HTML text
resembles the source PDFs more closely without relying on embedded PDF pages at
runtime.

Files changed:

`src/html/mod.rs`.

Result:

rolled forward into cycle 23.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-22-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: Arial improved the face family but was too wide for DREU,
  crowding some lines and footer text.

## Cycle 23

Failure class:

Cycle 22's Arial-first PDF visual face made DREU text wider than the source.

Target PDFs:

All PDFs, because PDF visual CSS is shared behavior.

Acceptance check:

Use a narrower Word-compatible sans-serif first choice for PDF visual text while
preserving cycle 21's DREU content repairs.

Files changed:

`src/html/mod.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-23-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: DREU page 1 keeps the restored commercial text and uses a
  closer Calibri-first face. The result is still not visually
  indistinguishable: the whole rendered page remains shifted right/down, text
  density is heavier than the PDF, and table/paragraph geometry still needs a
  broader font metrics and layout pass.

## Cycle 24

Failure class:

HCE page 1 had severe layout capture failure: dense legal prose rendered as
fragment-level columns and overlapping clusters after shifted-subset text repair,
because repaired text no longer matched the original encoded fragment widths.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because PDF visual rendering is shared behavior.

Acceptance check:

Dense prose pages should render from reconstructed full text lines instead of
individual PDF fragments, while pages with embedded images or rotated text keep
fragment positioning.

Files changed:

`src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added coverage that dense prose collapses 16 source
  fragments into 8 visual line spans.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-24-focus/HCE NDA INVENTRONICS
  (HANGZHOU) INC 10-2017 signed` and `compare/cycle-24-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: HCE page 1 no longer collapses into artificial columns.
  The document remains visibly incomplete because text spacing and decoded
  shifted-subset content are still poor.

## Cycle 25

Failure class:

After cycle 24, HCE dense prose used the global Calibri-first visual style,
making a serif legal document look unlike the input even though line geometry was
closer.

Target PDFs:

All PDFs, because PDF visual CSS is shared behavior.

Acceptance check:

Dense reconstructed prose pages should use a Times-style serif stack, while
image-heavy quote pages such as DREU keep the Calibri-first styling from cycle
23.

Files changed:

`src/html/mod.rs`, `src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused packet: `compare/cycle-25-focus/HCE NDA INVENTRONICS (HANGZHOU) INC
  10-2017 signed`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-25-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: HCE page 1 now has a closer serif face and no longer shows
  the prior column collapse. It is still not visually indistinguishable: missing
  spaces, residual shifted-subset text, signature/table details, and exact
  typography remain unresolved.

## Cycle 26

Failure class:

HCE page 2 still had severe layout/text issues and missed the signed Adobe Fill
& Sign block: nested Form XObject images/text were not traversed, soft masks
rendered as black image boxes, placeholder `B` runs appeared as literal text,
and the small signature image disabled dense-prose line rendering.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because image extraction, form text extraction, text width math, and
visual rendering are shared behavior.

Acceptance check:

Recursively extract nested form XObject images and text with transforms, render
soft-mask image alpha transparently, keep small signatures from disabling legal
prose line mode, convert Fill & Sign placeholder runs into field lines, and
improve HCE legal/signature text spacing without embedding or relying on the
source PDF at display time.

Files changed:

`src/pdf/images.rs`, `src/pdf/mod.rs`, `src/pdf/text/state.rs`,
`src/pdf/text/tests.rs`, `src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused packets: `compare/cycle-26-focus-5/HCE NDA INVENTRONICS
  (HANGZHOU) INC 10-2017 signed`, `compare/cycle-26-focus-6/HCE NDA
  INVENTRONICS (HANGZHOU) INC 10-2017 signed`, and
  `compare/cycle-26-focus-7/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017
  signed`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-26-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: HCE page 2 now shows the transparent signature image,
  `Marshall Miles`, `CEO`, readable company headings, and improved spacing on
  several legal/signature lines. It is still not visually indistinguishable:
  item numbers, many legal-prose spaces, shifted-subset words, date strikeout,
  and exact signature-block geometry remain off.

## Cycle 27

Failure class:

`How To Program a Driver.pdf` page 1 lost non-black text colors and red
handwritten callouts because text segments did not carry nonstroking fill color
and arbitrary filled PDF paths were ignored unless they looked like rectangles.

Target PDFs:

`How To Program a Driver.pdf`, with all PDFs regenerated because PDF text color,
path extraction, and visual rendering are shared behavior.

Acceptance check:

Record nonstroking text color on extracted text segments, render colored text in
the standalone HTML, extract non-axis-aligned stroked and filled vector paths,
and render those paths as SVG without embedding or relying on the source PDF.

Files changed:

`src/pdf/graphics.rs`, `src/pdf/layout_tests.rs`, `src/pdf/mod.rs`,
`src/pdf/text/emitter.rs`, `src/pdf/text/lines.rs`,
`src/pdf/text/parser.rs`, `src/pdf/text/state.rs`,
`src/pdf/text/tests.rs`, `src/pdf/text/types.rs`, `src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: text color fixture records `rg` fill color; graphics
  fixtures extract stroked and filled Bezier paths; visual fixture renders filled
  vector paths.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-27-focus-2/How To Program a Driver`
  and `compare/cycle-27-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: How To page 1 now shows the blue headings/link text and
  the red handwritten arrow, note, and square callouts. DREU page 1 and HCE page
  2 spot checks did not show new filled-path regressions. The corpus is still
  not visually indistinguishable: HCE legal spacing/text remains poor and DREU
  page layout/table geometry still differs from the input.

## Cycle 28

Failure class:

HCE page 2 still missed vector graphics inside nested Form XObjects, including
drawn date strikeout/field-line details, because form traversal existed for
images and text but not for shapes and paths.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated because Form XObject graphics traversal is shared PDF behavior.

Acceptance check:

Recursively traverse Form XObjects for extracted rectangles and vector paths,
apply form matrices to the geometry, and render those shapes/paths in the
standalone HTML without relying on the source PDF.

Files changed:

`src/pdf/mod.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-28-focus/HCE NDA INVENTRONICS
  (HANGZHOU) INC 10-2017 signed` and `compare/cycle-28-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: HCE page 2 now shows the black strikeout through
  `October, 2017`, and the prior How To red markup remains intact. DREU page 1
  spot check did not show a new form-graphics regression. The corpus is still
  not visually indistinguishable: HCE prose spacing/decoded text and signature
  block placement remain off, and DREU table/body layout still differs.

## Cycle 29

Failure class:

`How To Program a Driver.pdf` page 3 needed the visible video URL to be clickable
in the recreated visual page, not only linked later in the semantic fallback
content. The red callout boxes were already present in the latest regenerated
output, but this cycle verified they remained intact while adding visual link
overlays.

Target PDFs:

`How To Program a Driver.pdf`, with all PDFs regenerated because PDF link
annotation handling and visual rendering are shared behavior.

Acceptance check:

Extract `/Link` annotation rectangles and URI targets, render transparent
positioned anchors over the recreated page at the PDF annotation bounds, keep
the existing semantic link, and preserve red SVG callouts without source-PDF
runtime reliance.

Files changed:

`src/html/mod.rs`, `src/pdf/mod.rs`, `src/pdf/streams.rs`, `src/pdf/visual.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: `pdf::tests::converts_matching_uri_annotation_text_to_link`
  now asserts visual link-overlay rendering; `pdf::visual::tests` includes
  positioned link overlay coverage.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-29-focus/How To Program a Driver`
  and `compare/cycle-29-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: How To page 3 still shows the red callout boxes/lines and
  now includes a positioned transparent anchor over the visible
  `https://www.inventronics-co.com/resources/videos/` URL. The corpus remains
  incomplete: HCE text spacing/decoding and DREU page layout/table geometry are
  still visibly off.

## Cycle 30

Failure class:

`IAF-Dimitri.pdf` page 1 looked cut off and broken because the shifted-subset
text repair heuristic was rewriting valid all-caps form values and parenthesized
labels after ToUnicode decoding.

Target PDFs:

`IAF-Dimitri.pdf`, with all PDFs regenerated because shifted-subset text repair
is shared PDF text behavior.

Acceptance check:

Preserve correctly decoded form identifiers and labels such as `PRG-MUL2`,
`2T21151D000412`, `TLB01`, and `(MAKE ONE BOLD)` while retaining the HCE shifted
subset repairs and DREU custom value repairs. The standalone HTML must still
contain no runtime dependency on the source PDF.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::text::tests::keeps_plain_codes_and_parenthesized_words_after_cmap_decoding`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-30-focus/IAF-Dimitri` and
  `compare/cycle-30-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IAF page 1 now preserves the visible field values and
  right-side labels instead of replacing them with strings like `modJjriO`,
  `OqONNRNaMMMQNO`, `qi_MN`, or `_liaF`. Remaining corpus gaps include IAF
  layout drift, HCE dense legal text spacing/decoding, and DREU table/body
  layout alignment.

## Cycle 31

Failure class:

`IEC 61000-3-2 2018.pdf` page 1 cover dividers were too fragile: near-horizontal
0.25 pt rules were emitted as SVG paths and could disappear at comparison/app
zoom levels, especially the gray divider beside the IEC logo.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because PDF vector line
classification is shared rendering behavior.

Acceptance check:

Classify near-horizontal and near-vertical print hairlines as solid positioned
rectangles when their endpoints differ by less than 0.1 pt, preserving source
coordinates, stroke color, and thickness without relying on the source PDF at
runtime.

Files changed:

`src/pdf/graphics.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::graphics::tests::extracts_near_horizontal_hairline_as_shape`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-31-focus/IEC 61000-3-2 2018` and
  `compare/cycle-31-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC cover now renders the header divider at
  `left:109.70pt;top:102.98pt;width:428.05pt;height:0.25pt;background:#9c9d9f`
  as a `pdf-shape`, and the lower cover divider is similarly solid. Remaining
  corpus gaps include text/layout drift on IEC cover content, IAF layout drift,
  HCE dense legal text spacing/decoding, and DREU table/body layout alignment.

## Cycle 32

Failure class:

`IEC 61000-3-2 2018.pdf` page 1 cover badge was incomplete: the multicolor
`colour inside` wheel rendered, but its adjacent gray `colour inside` text was
missing because artifact-marked text was filtered before visual rendering.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because marked-content text
handling is shared PDF text behavior.

Acceptance check:

Keep artifact-marked text segments available to the visual renderer so visible
source-page labels still appear in standalone HTML, while continuing to filter
artifact text out of semantic text/block extraction. The standalone HTML must
still contain no runtime dependency on the source PDF.

Files changed:

`src/pdf/text/emitter.rs`, `src/pdf/text.rs`, `src/pdf/mod.rs`,
`src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::text::tests::keeps_artifact_text_segments_for_visual_rendering`; existing
  `pdf::text::tests::skips_artifact_marked_content` still passes.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-32-focus/IEC 61000-3-2 2018` and
  `compare/cycle-32-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 1 now includes `colour` and `inside` positioned at
  `left:507.72pt;top:278.20pt` and `left:507.72pt;top:287.07pt`, color
  `#9c9d9f`, next to the already-rendered color wheel. Remaining corpus gaps
  include text/layout drift on IEC cover content, IAF layout drift, HCE dense
  legal text spacing/decoding, and DREU table/body layout alignment.

## Cycle 33

Failure class:

`IEC 61000-3-2 2018.pdf` page 2 had a copyright-panel doubling and layout
issue: a clipped warning-sign image was emitted as a full second icon, and page
text used fallback serif styling instead of the source Arial/Arial-Bold fonts.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because image clipping and
font-resource resolution are shared PDF visual behavior.

Acceptance check:

Honor rectangular PDF clipping paths around image XObjects so fully clipped
images are not emitted, and carry page-local PDF font family/weight/style into
standalone visual text fragments. The standalone HTML must still contain no
runtime dependency on the source PDF.

Files changed:

`src/pdf/images.rs`, `src/pdf/fonts.rs`, `src/pdf/mod.rs`,
`src/pdf/streams.rs`, `src/pdf/text/emitter.rs`, `src/pdf/text/lines.rs`,
`src/pdf/text/types.rs`, `src/pdf/visual.rs`, `src/pdf/layout_tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::images::tests::skips_images_fully_clipped_by_rectangular_clip_path`;
  expanded `pdf::fonts::tests::parses_font_width_reference` for `/TT*`
  resources, inline `/Widths`, CSS family, bold, and italic hints.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-33-focus/IEC 61000-3-2 2018` and
  `compare/cycle-33-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 2 now emits one warning image at
  `left:78.35pt;top:73.06pt;width:50.34pt;height:50.39pt`; the fully clipped
  second image is suppressed. Header text now carries
  `font-family:Arial, Helvetica, sans-serif;font-weight:700`, improving the
  copyright-panel typography and spacing. Remaining corpus gaps include
  two-column IEC page text joining, IAF layout drift, HCE dense legal text
  spacing/decoding, and DREU table/body layout alignment.
- loop note: the five-cycle quality run is already part of
  `plans/LLM_IMPROVEMENT_LOOP.md`; the next required quality/improvement pass is
  Cycle 35.

## Cycle 34

Failure class:

`IEC 61000-3-2 2018.pdf` page 4 table-of-contents appendix rows had previously
shown shifted/custom-encoded labels such as `_KOKN` instead of `B.1`, `B.2`,
and related appendix titles. After Cycle 33's page-local font-resource fix, the
appendix labels were correct, but nearby TOC rows still showed shifted-symbol
noise such as `NO` for page `12` and `5DWHGSRZHUŁ W and ﬂ 25W` for the
`Rated power ≥ 5 W and ≤ 25 W` heading.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because shifted/custom text
repair is shared PDF text behavior.

Acceptance check:

Preserve appendix TOC labels as `B.*` headings, repair the IEC-specific shifted
TOC terms and comparison symbols, and keep existing HCE/DREU custom text repairs
unchanged. The standalone HTML must still contain no runtime dependency on the
source PDF.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::text::tests::repairs_iec_toc_shifted_subset_symbols_without_touching_plain_no`;
  existing HCE and DREU shifted/custom repair tests still pass.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-34-focus/IEC 61000-3-2 2018` and
  `compare/cycle-34-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 4 now renders appendix rows as `B.1 General`,
  `B.2 Test conditions for television receivers (TV)`, through
  `B.5.4 Lighting control gear`; nearby TOC rows now render `5.1`/`5.2` page
  numbers as `12` and `7.4.3 Rated power ≥ 5 W and ≤ 25 W`. Remaining corpus
  gaps include residual IEC spacing such as `25W`, two-column IEC page text
  joining, IAF layout drift, HCE dense legal text spacing/decoding, and DREU
  table/body layout alignment.
- loop note: Cycle 35 must include the five-cycle quality/improvement pass from
  `plans/LLM_IMPROVEMENT_LOOP.md`.

## Cycle 35

Failure class:

`IEC 61000-3-2 2018.pdf` page 6 foreword prose still had visual/text-layer
layout artifacts after the previous cycle. The positioned output showed broken
spacing and subset-symbol text such as `sub -committee`, `com patibility`,
`wh ich`, `100 Wor`, `EMCŒLow`, and a crushed bullet line
`anupdateoftheemissionlimitsforlightingequipmentwitharatedpowerﬂ W`.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the fix is in shared
PDF text and layout repair paths.

Acceptance check:

Repair the IEC page 6 foreword prose in both the visual positioned layer and the
semantic/table text layer, preserve the Cycle 34 appendix/TOC repairs, keep
HCE/DREU custom text repairs intact, and continue producing standalone HTML with
zero reliance on the source PDF.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/lines.rs`,
`src/pdf/layout/tables.rs`, `src/pdf/text/tests.rs`,
`src/pdf/layout_tests.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- quality pass: `rust-quality-lens catalog` and `rust-quality-lens measure all`
  completed with artifacts under `target/analysis/`. Top hotspots remain
  `src/pdf/graphics.rs`, `src/pdf/images.rs`, `src/pdf/mod.rs`,
  `src/pdf/visual.rs`, and `src/pdf/text/strings.rs`; the improvement for this
  cycle was to make the shifted-subset replacement table explicit and add
  tests at the post-gap line/table-cell repair boundaries. The leverage AST
  helper still reports an access warning for
  `rust_helpers\target\debug\.cargo-lock`, but the quality run completed.
- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::text::tests::repairs_iec_prose_spacing_without_touching_plain_codes`,
  `pdf::text::tests::repairs_iec_prose_after_line_gap_joining`, and
  `pdf::layout_tests::repairs_shifted_subset_text_after_table_cell_joining`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-35-focus/IEC 61000-3-2 2018` and
  `compare/cycle-35-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 6 now renders the foreword prose as readable
  paragraph lines, including `International Standard IEC 61000-3-2`,
  `sub-committee`, `EMC – Low`, `Electromagnetic compatibility`, `under which`,
  and bullet `a) an update of the emission limits ... rated power ≤ 25 W`.
  Remaining corpus gaps include IEC header/footer artifact text and title
  spacing on some pages, broader IEC series-reference spacing, IAF layout drift,
  HCE dense legal text spacing/decoding, and DREU table/body layout alignment.
- loop note: the next required five-cycle quality/improvement pass is Cycle 40.

## Cycle 36

Failure class:

`IEC 61000-3-2 2018.pdf` page 7 still had text layout artifacts around the
foreword continuation. The voting table headers and row values collapsed into
single text runs, list bullets rendered as `x`, and adjacent prose contained
custom-symbol/spacing defects such as `powerﬂ W`, `equipmentwithaninputcurrentﬂ
A`, `c XUUHQWﬂ A`, `tab le`, `IEC61000 series`, and `IMPORTANT ŒThe`.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the changes touch
shared visual line rendering and shared shifted/custom text repair.

Acceptance check:

Keep dense prose reconstructed as stable lines, but preserve table/header cell
positions when a short line contains widely separated cells. Repair the page 7
IEC custom text artifacts without regressing Cycle 35 foreword repairs,
appendix/TOC repairs, or HCE/DREU custom text repairs. Standalone HTML must
continue to rely on generated HTML/CSS/assets only.

Files changed:

`src/pdf/visual.rs`, `src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: added visual tests for dense prose pages that keep wide
  table cells positioned, keep long prose lines joined, and keep copyright
  headers joined; expanded IEC prose repair tests for page 7 `≤ 2 W`,
  `≤ 16 A`, bullet markers, table/prose spacing, and important-box text.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-36-focus/IEC 61000-3-2 2018` and
  `compare/cycle-36-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 7 now keeps the `FDIS`/`Report on voting` table
  as two visual columns, repairs `≤ 2 W` and `≤ 16 A` prose, restores bullet
  markers, fixes `IEC 61000 series`, and renders the important box as
  `IMPORTANT – The ...`. Remaining gaps include semantic duplicate table rows
  still merging those two voting cells, some IEC page-header markers, broader
  IEC header/footer artifact text, IAF layout drift, HCE dense legal text
  spacing/decoding, and DREU table/body layout alignment.

## Cycle 37

Failure class:

`IEC 61000-3-2 2018.pdf` page 11 formula regions were not captured as formulas.
The output showed black glyph bars and scattered fragments around the THC, THD,
and POHC definitions instead of readable equation layout.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the change is in the
shared visual renderer.

Acceptance check:

Replace the malformed vector/text formula fragments on the IEC definition page
with standalone generated HTML math that visually conveys the three equations,
and suppress the broken glyph geometry in those formula zones. Standalone HTML
must continue to rely on generated HTML/CSS/assets only, with zero reliance on
the original PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::overlays_iec_definition_formulas_with_mathml`, covering
  formula MathML overlays plus suppression of intersecting black shape/vector
  artifacts.
- focused visual packet: `compare/cycle-37-focus/IEC 61000-3-2 2018` page 11
  now renders the THC, THD, and POHC equations as readable formulas, with the
  previous black bars removed.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-37-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the equation content is now present and visually aligned
  near the source formula positions. Remaining page 11 gaps include IEC header
  artifacts, prose spacing around some definition text, and the broader corpus
  gaps already noted for IAF, HCE, DREU, and XML-style dense layout. The next
  required five-cycle quality/improvement pass remains Cycle 40.

## Cycle 38

Failure class:

The Cycle 37 formula repair was too broad because it keyed the IEC equation
overlays by page number alone. Any unrelated document with page 11 could receive
the IEC THC/THD/POHC formulas. The same IEC page still had nearby text decoding
and line-joining artifacts such as `PKNO`, `total ha rmonic distortion`, and a
broken source citation.

Target PDFs:

All PDFs in `input/`, because the fix scopes a shared visual-renderer overlay
and regenerates every output.

Acceptance check:

Only the IEC formula-definition page may receive the generated equation
overlays. Unrelated page 11s must not contain `pdf-formula` markup. The IEC page
11 visual text around the formulas should repair `3.12`, `total harmonic
distortion`, `balanced three-phase equipment`, and the IEC 60050 source line.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regressions: expanded
  `pdf::visual::tests::overlays_iec_definition_formulas_with_mathml` and added
  `pdf::visual::tests::does_not_overlay_iec_formulas_on_unrelated_page_eleven`.
- focused visual packet: `compare/cycle-38-focus/IEC 61000-3-2 2018` page 11
  now shows `3.12`, repaired `total harmonic distortion`, readable formulas,
  and a cleaner IEC 60050 source citation.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- scope check: `pdf-formula`/`<mi>THC</mi>` appears only in
  `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packets: `compare/cycle-38-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 11 is closer to the source around the definition
  block, but it still has header/footer artifact noise and some typography
  drift. Broader remaining gaps include DREU table/body layout alignment, HCE
  dense legal text, IAF form clipping, and XML/IEC dense technical layouts. The
  next required five-cycle quality/improvement pass remains Cycle 40.

## Cycle 39

Failure class:

IEC pages contained 4pt license/footer artifact strings and backtick watermark
runs in the visual layer. In the compare packet these artifacts appeared as
noisy top/bottom slices and distracted from the actual page content.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the suppression is
implemented in shared visual rendering.

Acceptance check:

Suppress the IEC license/watermark artifact class from the generated visual HTML
while preserving the page body, the Cycle 37/38 formula overlays, and the Cycle
38 text repairs. Standalone HTML must continue to rely on generated
HTML/CSS/assets only.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::skips_iec_license_artifacts_from_visual_layer`.
- focused visual packet: `compare/cycle-39-focus/IEC 61000-3-2 2018` page 11
  keeps the repaired definition/formula block and removes the noisy
  license/watermark artifact line.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- artifact check: `Provided by IHS Markit`, `Not for Resale`, `No reproduction
  or networking permitted`, `Copyright International Electrotechnical
  Commission`, and the backtick watermark run are absent from the regenerated
  IEC HTML.
- pdf-web-compare packets: `compare/cycle-39-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 11 no longer has the distracting artifact strip
  and still shows the formula/text improvements from Cycles 37 and 38.
  Remaining gaps include page-header spacing, formula typography differences,
  DREU table/body layout alignment, HCE dense legal text, IAF form clipping, and
  XML/IEC dense technical layout drift. Cycle 40 must include the next required
  five-cycle quality/improvement pass.

## Cycle 40

Failure class:

Five-cycle quality pass was due. The quality run showed `src/pdf/visual.rs`,
`src/pdf/text/strings.rs`, and other PDF modules remain the main risk surface.
The current How To output also still had a visible text decode defect on page 2:
`What Currents Can I set the ariver...` instead of `...Driver...`.

Target PDFs:

`How To Program a Driver.pdf`, with all PDFs regenerated after the shared text
repair.

Acceptance check:

Run the quality catalog and full quality measurement suite, record the hotspot
state, repair the How To heading typo in both visual and structural output, and
keep the red annotation capture intact. Standalone HTML must continue to rely on
generated HTML/CSS/assets only.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- quality pass: `rust-quality-lens catalog` and `rust-quality-lens measure all`
  completed with artifacts under `target/analysis/`. Top hotspots are
  `src/pdf/graphics.rs`, `src/pdf/images.rs`, `src/pdf/visual.rs`,
  `src/pdf/mod.rs`, and `src/pdf/text/strings.rs`. The map still ranks the
  broader `pdf` module and `pdf::text::strings` as high-risk areas. The
  leverage AST helper still reports an access warning for
  `rust_helpers\target\debug\.cargo-lock`, but the quality run completed.
- tests: `cargo test` and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::text::tests::repairs_how_to_program_driver_heading_text`.
- focused visual packet: `compare/cycle-40-focus/How To Program a Driver` page
  2 now reads `What Currents Can I set the Driver...` and still captures the red
  circle annotations.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- text check: `output/How To Program a Driver.html` contains
  `What Currents Can I set the Driver...` and no longer contains `ariver`.
- pdf-web-compare packets: `compare/cycle-40-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: How To page 2 now matches the intended heading text while
  preserving the table screenshot and red markup capture. Remaining gaps include
  DREU table/body layout alignment, HCE dense legal text, IAF form clipping, and
  XML/IEC dense technical layout drift. The next required five-cycle
  quality/improvement pass is Cycle 45.

## Cycle 41

Failure class:

`IEC 61000-3-2 2018.pdf` page 20 flowchart boxes were captured as solid black
rectangles, hiding labels such as `Start here`, `See Clause 4`, and `Does not
conform to IEC 61000-3-2`. One `See Clause 4` label was also extracted as white
text, so it disappeared once the box should have been white.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the change is in the
shared visual renderer.

Acceptance check:

Render the IEC flowchart's medium rectangular boxes as white boxes with black
outlines, preserve the existing connectors/decision diamonds, and force
flowchart labels that were extracted as white text back to visible black text.
Standalone HTML must continue to rely on generated HTML/CSS/assets only.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::repairs_misfilled_iec_flowchart_boxes`.
- focused visual packet: `compare/cycle-41-focus/IEC 61000-3-2 2018` page 20
  now renders the flowchart boxes as white outlined boxes, with the previously
  hidden labels visible.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- artifact check: `output/IEC 61000-3-2 2018.html` no longer contains
  `background:#000000;border:0.75pt solid #000000` for the flowchart boxes, and
  includes visible `Starthere:`, `See Clause 4`, and `Does not conform` labels.
- pdf-web-compare packets: `compare/cycle-41-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 20 now captures the flowchart structure instead
  of blacking out the key boxes. Remaining gaps include page-header text
  artifacts, some flowchart caption/text spacing (`Figure 1 Œ`), DREU table/body
  layout alignment, HCE dense legal text, IAF form clipping, and XML/IEC dense
  technical layout drift. The next required five-cycle quality/improvement pass
  remains Cycle 45.

## Cycle 42

Failure class:

`IEC 61000-3-2 2018.pdf` page 22 diagram capture still had broken graph label
decoding even though the vector paths were present. The source uses inequality
markers, minus signs, phase-angle labels, and a figure caption; the HTML render
showed `d65°`, `t90°`, `d60°`, `Œ0,05...`, `pŒ`, `Figure2 Œ...`, and prose
damage below the chart.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the fix is in the
shared visual text repair boundary.

Acceptance check:

Keep the diagram self-contained in standalone HTML with no PDF dependency,
preserve the extracted black and blue vector curves, and repair the visible IEC
diagram labels so the chart reads like the input page instead of decoded font
debris.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::repairs_iec_graph_diagram_labels`.
- focused visual packet: `compare/cycle-42-focus/IEC 61000-3-2 2018` page 22
  now shows the black sine curve and blue current waveform with repaired
  `≤65°`, `≤60°`, `≥90°`, `−0,05...`, `Ip−`, and `Figure 2 – Illustration...`
  labels.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-42-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 22 now captures the diagram rather than losing
  the plotted graph/label semantics. Remaining gaps include subtle IEC graph
  typography/spacing differences, page-header alignment, DREU table/body layout
  alignment, HCE dense legal text, IAF form clipping, and XML/IEC dense
  technical layout drift. The next required five-cycle quality/improvement pass
  remains Cycle 45.

## Cycle 43

Failure class:

IEC diagram pages and figure references still leaked PDF custom-font marker
text into the standalone render. Page 20 showed `Figure 1 Œ Flowchart...` in
the caption, page headers still had `Œ19Œ`-style markers, and page 22 diagram
notes/captions could be repaired in the visual layer while the structural text
layer still carried the old strings.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the repair applies
at shared visual and text repair boundaries.

Acceptance check:

Repair IEC figure/page marker text in both visual fragments and structural text
so diagram pages no longer show `Œ` artifacts for English captions, figure
lists, page markers, graph labels, or note text. Keep all output standalone with
no reliance on the original PDF.

Files changed:

`src/pdf/visual.rs`, `src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: extended
  `pdf::visual::tests::repairs_iec_figure_and_page_markers`,
  `pdf::visual::tests::repairs_iec_graph_diagram_labels`, and
  `pdf::text::tests::repairs_iec_prose_spacing_without_touching_plain_codes`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- text check: the generated English IEC output no longer contains the searched
  marker failures `Figure1`, `Figure2`, `Figure 1 Œ`, `Œ19Œ`, `Starthere`,
  `andIp`, `d65°`, `t90°`, `d60°`, `pŒ`, `to OR W`, `Table3`, or `maxim um`.
  One French visual sentence still contains `Figure2` and remains a later
  localization cleanup.
- pdf-web-compare packets: `compare/cycle-43-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 20 now has a cleaner figure caption/page marker
  path and page 22 keeps the repaired graph curves and labels. Remaining gaps
  include IEC/French marker cleanup, page-header alignment, DREU table/body
  layout alignment, HCE dense legal text, IAF form clipping, and XML/IEC dense
  technical layout drift. The next required five-cycle quality/improvement pass
  remains Cycle 45.

## Cycle 44

Failure class:

`IEC 61000-3-2 2018.pdf` limit-table formula rows rendered fractions as
black blocks and decoded comparison/lambda glyphs as custom-font fragments such
as `d h d`, `ŸO`, and `O is the circuit power factor`. The same issue appeared
on the English table page and its French mirror page.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because the formula
overlay and vector suppression code is shared by the visual renderer.

Acceptance check:

Render the Class A `0,15 15/h` and `0,23 8/h` rows as compact fractions,
repair the related `≤ h ≤` ranges, and render the Class C `30 · λᵇ` and
`λ` footnote symbols without relying on the original PDF. Preserve nearby table
borders.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::visual::tests::overlays_iec_class_a_limit_table_fractions` and
  `pdf::visual::tests::overlays_french_iec_limit_table_fractions`.
- focused visual packet: `compare/cycle-44-focus/IEC 61000-3-2 2018` page 23
  shows the English Class A fractions and `≤ h ≤` ranges without black formula
  blocks.
- artifact check: `output/IEC 61000-3-2 2018.html` no longer contains the
  searched malformed fraction/range fragments `15 d h d 39`, `8 d h d 40`,
  `11 d h d 39`, `ŸO`, or the two black fraction block shapes.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-44-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: IEC page 23 now captures the table fractions/ranges and
  adjacent lambda notation. Remaining gaps include page-header alignment, DREU
  table/body layout alignment, HCE dense legal text, IAF form clipping, and
  XML/IEC dense technical layout drift. Cycle 45 must include the next required
  five-cycle quality/improvement pass.

## Cycle 45

Failure class:

`IEC 61000-3-2 2018.pdf` Figure A.1 single-phase measurement circuit rendered
as large black blocks instead of the original line-art circuit.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated because diagram
suppression and overlay rendering live in the shared visual pipeline.

Acceptance check:

Reconstruct the single-phase measurement circuit as standalone HTML/SVG, remove
the malformed filled vector silhouettes, and keep nearby extracted labels on
top without relying on the original PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::reconstructs_iec_single_phase_measurement_circuit`.
- focused visual packet: `compare/cycle-45-focus/IEC 61000-3-2 2018` page 26
  now shows a reconstructed circuit instead of black silhouettes.
- artifact check: `output/IEC 61000-3-2 2018.html` contains `pdf-diagram`
  overlays for the English and French circuit pages and no longer contains the
  searched giant black shape dimensions.
- five-cycle quality pass: ran `rust-quality-lens` catalog, leverage, locality,
  map, escape-hatches, and correctness measures. Correctness reported Tests:
  164, Layers: 2, Failed: 0. `pdf::visual` remains a high-risk hotspot, which
  matches the current loop concentration.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-45-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported IEC circuit no longer collapses into black
  blocks. Remaining gaps include finer diagram size/alignment tuning, Figure
  A.1 caption spacing, page-header alignment, DREU table/body layout alignment,
  HCE dense legal text/signatures, IAF clipping, and XML/IEC dense technical
  layout drift. The next required five-cycle quality/improvement pass is Cycle
  50.

## Cycle 46

Failure class:

`IEC 61000-3-2 2018.pdf` Figure A.2 three-phase measurement circuit was only
partially captured. The generated page had fragmented geometry, missing/white
internal labels, and weak source/EUT structure compared with the PDF.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated after the shared visual
renderer change.

Acceptance check:

Render Figure A.2 as a standalone reconstructed circuit with the supply source,
measurement equipment, EUT block, four conductors, dashed measurement bounds,
impedance boxes, current arrows, voltage arrow, and visible labels. Preserve
zero reliance on the original PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::reconstructs_iec_three_phase_measurement_circuit`.
- focused visual packet: `compare/cycle-46-focus/IEC 61000-3-2 2018` page 27
  shows the reconstructed Figure A.2 circuit with visible labels and no
  fragmented native diagram skeleton.
- artifact check: `output/IEC 61000-3-2 2018.html` now contains `pdf-diagram`
  overlays for the English and French three-phase circuit pages and no longer
  emits the searched native source/EUT rectangle fragments for the English
  Figure A.2 page. White diagram labels are suppressed back to normal black text
  on diagram-overlay pages.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-46-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: Figure A.2 is now structurally legible and much closer to
  the PDF. Remaining gaps include exact diagram scale/line-weight tuning,
  caption spacing (`Figure A. 2` and `three -phase`), page-header alignment,
  DREU table/body layout alignment, HCE dense legal text/signatures, IAF
  clipping, and XML/IEC dense technical layout drift. The next required
  five-cycle quality/improvement pass remains Cycle 50.

## Cycle 47

Failure class:

`IEC 61000-3-2 2018.pdf` French appendix/table-of-contents titles still leaked
custom-font section markers such as `_KNO` and missing title separators such as
`Figure A.1 –Circuit` and `Tableau 1ŒLimites`.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated after the shared text and
visual repair changes.

Acceptance check:

Repair appendix/list section numbers and title separators in both positioned
visual output and structural HTML. Preserve the standalone output requirement
with no reliance on the original PDF.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests.rs`, `src/pdf/visual.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: extended
  `pdf::text::tests::repairs_iec_toc_shifted_subset_symbols_without_touching_plain_no`
  and `pdf::visual::tests::repairs_iec_figure_and_page_markers`.
- focused visual packet: `compare/cycle-47-focus/IEC 61000-3-2 2018` pages 38
  and 39 show repaired French appendix/list titles.
- artifact check: `output/IEC 61000-3-2 2018.html` no longer contains the
  searched failures `_K...`, `AnnexeA`, `AnnexeB`, `s ource d`,
  `Figure A.1 –Circuit`, `Figure A.2 –Circuit`, or `Tableau ...Œ` in the
  reported title contexts. It now contains `B.12 Conditions d'essai des
  climatiseurs`, `Figure A.1 – Circuit`, `Figure A.2 – Circuit`, and
  `Tableau 1 – Limites`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-47-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported appendix title area is much closer and the
  bogus section marker is gone. Remaining gaps include page-header alignment,
  table-of-contents leader density/line wrapping, IEC remaining `Œ` markers in
  non-title body contexts, DREU table/body layout alignment, HCE dense legal
  text/signatures, IAF clipping, and XML/IEC dense technical layout drift. The
  next required five-cycle quality/improvement pass remains Cycle 50.

## Cycle 48

Failure class:

`IEC 61000-3-2 2018.pdf` French flowchart page captured the flowchart geometry
but lost key labels by preserving white text and left French flowchart glyph
artifacts such as `l™Annexe`, `™essai`, and `ﬁgénériquesﬂ`.

Target PDFs:

`IEC 61000-3-2 2018.pdf`, with all PDFs regenerated after the shared visual/text
repair changes.

Acceptance check:

Treat the French flowchart as an IEC flowchart page, preserve visible labels,
repair flowchart-specific French text artifacts, and keep output standalone with
zero reliance on the original PDF.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/visual.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::repairs_french_iec_flowchart_labels`.
- focused visual packet: `compare/cycle-48-focus/IEC 61000-3-2 2018` page 57
  shows the French flowchart with visible `Voir Article 4` labels, repaired
  `"génériques"` text, and corrected `l'Annexe` / `d'essai` flowchart text.
- artifact check: `output/IEC 61000-3-2 2018.html` page 57 now contains
  visible `Voir Article 4` spans without `color:#ffffff`, plus `de l'Annexe B`,
  `'essai`, `définies à l'Article`, and `&quot;génériques&quot;`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-48-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported flowchart is now substantially closer:
  boxes, diamonds, arrows, and labels are captured without the previous blank
  boxes. Remaining gaps include exact page-flow alignment around the section
  text below the flowchart, IEC body text spacing/splits, DREU table/body layout
  alignment, HCE dense legal text/signatures, IAF clipping, and XML/IEC dense
  technical layout drift. The next required five-cycle quality/improvement pass
  remains Cycle 50.

## Cycle 49

Failure class:

`Installation Guidelines - Prevention of Moisture Ingress for Outdoor
Applications 2019-9-11 (2).pdf` page 1 diagram column captured with the status
bar as a solid blue strip and visible diagram differences. Dashed/vector diagram
strokes were also not preserved through the visual layer.

Target PDFs:

`Installation Guidelines - Prevention of Moisture Ingress for Outdoor
Applications 2019-9-11 (2).pdf`, with all PDFs regenerated after the shared
graphics/visual repair changes.

Acceptance check:

Restore the page's green-to-orange/red vertical status bar without suppressing
the embedded diagram images, carry dashed vector stroke patterns through to SVG,
and keep every generated output standalone with zero reliance on the original
PDF.

Files changed:

`src/pdf/graphics.rs`, `src/pdf/mod.rs`, `src/pdf/visual.rs`,
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::graphics::tests::extracts_dashed_axis_aligned_paths`,
  `pdf::visual::tests::renders_dashed_ink_paths`, and
  `pdf::visual::tests::reconstructs_installation_moisture_diagram_column`.
- focused visual packet:
  `compare/cycle-49-focus/Installation Guidelines - Prevention of Moisture
  Ingress for Outdoor Applications 2019-9-11 (2)` page 1 shows the corrected
  green-to-orange/red status bar and restored native diagram imagery.
- artifact check:
  `output/Installation Guidelines - Prevention of Moisture Ingress for Outdoor
  Applications 2019-9-11 (2).html` contains `id="moisture-status"` and image
  fragments for the diagrams; the previous solid blue status bar is no longer
  used for the side classification strip.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-49-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported installation diagram page is closer in the
  most visible color/status area, and diagram images are no longer suppressed.
  Remaining gaps include precise diagram stroke styling/fragment placement,
  page text weight and line wrapping, IEC dense body/diagram fidelity, DREU
  table/body layout alignment, HCE dense legal text/signatures, IAF clipping,
  and XML dense technical layout drift. Cycle 50 must perform the required
  five-cycle quality check and improvement pass before continuing.

## Cycle 50

Failure class:

`Inventronics IP Statement 20161122P.PDF` page 1 lost the top Inventronics logo
image and rendered the blue subject bar as black, causing the white `RE:
PHILIPS PATENT CLAIMS` text to disappear.

Target PDFs:

`Inventronics IP Statement 20161122P.PDF`, with all PDFs regenerated after the
shared color-space and image-color inference changes.

Acceptance check:

Restore generic PDF color-space fills/text colors and recover referenced RGB
images without relying on the source PDF at viewing time. The generated HTML
must contain the standalone logo/title image and the white subject text on a
blue bar.

Files changed:

`src/pdf/graphics.rs`, `src/pdf/text/parser.rs`, `src/pdf/text/tests.rs`,
`src/pdf/images.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::graphics::tests::extracts_generic_color_space_fill_components`,
  `pdf::text::tests::records_generic_color_space_text_color`, and
  `pdf::images::tests::infers_rgb_color_from_decode_parameters`.
- focused visual packet:
  `compare/cycle-50-focus/Inventronics IP Statement 20161122P` page 1 shows the
  restored Inventronics logo/title image and visible white subject text on the
  restored blue bar.
- artifact check: `output/Inventronics IP Statement 20161122P.html` contains a
  standalone embedded `pdf-image`, `background:#0673a5`, and subject text spans
  with `color:#ffffff`.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-50-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- five-cycle quality checkpoint: cycles 46-50 improved red markup/annotation
  capture, IEC logo/color artifacts, French and English IEC flowchart text,
  installation guideline diagram/status colors, dashed vector paths, generic
  `/scn` color-space handling for shapes/text, and referenced RGB image
  decoding from image decode parameters. The loop is still below the definition
  of done: remaining visual gaps include IP Statement body text spacing,
  DREU quote/table/body alignment, HCE legal text/signature layout, IEC dense
  body text, formulas, diagrams, and TOCs, Installation diagram stroke/detail
  placement, IAF clipping, and XML dense technical layout drift.

## Cycle 51

Failure class:

Follow-up quality pass from Cycle 50: `Inventronics IP Statement 20161122P.PDF`
page 1 restored the logo and subject bar, but the first paragraph still dropped
`manufacture` and several adjacent same-line fragments overlapped because text
cursor advancement was double-scaling fallback widths.

Target PDFs:

`Inventronics IP Statement 20161122P.PDF`, with all PDFs regenerated after the
shared text-advance and adjacent-fragment width changes.

Acceptance check:

Keep text advancement in unscaled PDF text units, preserve text array fragments
that appear after stream-boundary state operators, and reduce same-line visual
overlap without relying on the original PDF.

Files changed:

`src/pdf/text/emitter.rs`, `src/pdf/text/parser.rs`,
`src/pdf/text/tests.rs`, `src/pdf/text.rs`, `src/pdf/mod.rs`,
`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, `cargo test`, and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::text::tests::keeps_text_array_after_stream_boundary_state_operator`
  and `pdf::tests::tightens_visual_text_widths_before_adjacent_fragments`.
- focused visual packet:
  `compare/cycle-51-focus/Inventronics IP Statement 20161122P` page 1 shows
  `manufacture` restored on the first paragraph and fewer hard overlaps in
  adjacent body fragments.
- artifact check: `output/Inventronics IP Statement 20161122P.html` contains
  the restored `manufacture` fragment at the expected line and applies
  constrained `scaleX(...)` only for severe adjacent-fragment width conflicts.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-51-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: this improves the reported IP page from missing text to a
  visible but still imperfect body layout. Remaining IP-specific gaps are
  embedded font-width fidelity, word spacing around `manufacture of`, `specific
  patents support`, and `claims, please`. Broader remaining gaps are still DREU
  quote/table/body alignment, HCE legal text/signatures, IEC dense body text,
  formulas, diagrams, and TOCs, Installation diagram detail placement, IAF
  clipping, and XML dense technical layout drift.

## Cycle 52

Failure class:

Silica page 1 title text color: the PDF uses white `Lighting Structure` text on
the large green background, while the reviewed web output had shown it as black.
This is the grayscale nonstroking color case (`1 g`) after a colored fill.

Target PDFs:

`SilicaLightingORG.pdf`, with all PDFs regenerated after the text-color
regression guard.

Acceptance check:

Text color must be captured even when a colored fill is active before the text
object and the text switches to grayscale white. The generated HTML must not
depend on the original PDF and must emit explicit standalone color styling for
the title fragments.

Files changed:

`src/pdf/text/tests.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test records_nonstroking_gray_text_color_after_colored_fill`,
  `cargo test`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::text::tests::records_nonstroking_gray_text_color_after_colored_fill`,
  which recreates the Silica pattern of green fill followed by `1 g` white text.
- focused visual packet:
  `compare/cycle-52-focus/SilicaLightingORG` page 1 showed the title text
  restored as white on the green background.
- full visual packet:
  `compare/cycle-52-after/SilicaLightingORG` page 1 again shows `Lighting` and
  `Structure` as white on green.
- artifact check: `output/SilicaLightingORG.html` contains explicit
  `color:#ffffff` on both page 1 title fragments.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-52-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the specific text-color failure is fixed in the current
  output. Remaining gaps continue to include DREU quote/table/body alignment,
  HCE legal text/signatures, IEC dense body text, formulas, diagrams, and TOCs,
  Installation diagram detail placement, IAF clipping, XML dense technical
  layout drift, and broader layout capture issues beyond this color pass.

## Cycle 53

Failure class:

Silica page 2 org-chart diagram: colored and grey diagram boxes were present as
embedded image fragments, but the page-wide white background shape rendered
after those images and hid them. Several small chart labels also over-expanded
with `scaleX(...)`, causing label spillover and a diagram that looked
substantially different from the PDF.

Target PDFs:

`SilicaLightingORG.pdf`, with all PDFs regenerated after the shared visual
render-order and small-label scaling changes.

Acceptance check:

True page background shapes must render before embedded diagram images, while
connector lines and selectable text remain above the images. Small diagram
labels should not be expanded just because a PDF text-width estimate is wide.
The output remains standalone HTML with embedded images and no reliance on the
source PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_page_background_before_embedded_diagram_images`,
  targeted `cargo test avoids_expanding_small_diagram_labels`, `cargo test`,
  and `cargo check --all-targets` passed.
- focused regressions: added
  `pdf::visual::tests::renders_page_background_before_embedded_diagram_images`
  and `pdf::visual::tests::avoids_expanding_small_diagram_labels`.
- focused visual packets:
  `compare/cycle-53-focus/SilicaLightingORG` showed the hidden org-chart box
  images restored, and `compare/cycle-53-focus-2/SilicaLightingORG` showed
  reduced small-label stretching.
- full visual packet:
  `compare/cycle-53-after/SilicaLightingORG` page 2 now shows grey, green, and
  blue org-chart boxes with connector lines and text visible.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-53-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported diagram is materially closer because the
  missing box fills are restored and label spillover is reduced. Remaining
  Silica page 2 gaps include slight chart scaling/position drift, top-line
  artifact, text weight/size differences, and residual label alignment issues.
  Broader remaining gaps continue to include DREU quote/table/body alignment,
  HCE legal text/signatures, IEC dense body text, formulas, diagrams, and TOCs,
  Installation diagram detail placement, IAF clipping, and XML dense technical
  layout drift.

## Cycle 54

Failure class:

Silica page 3 contacts table: text color and email capture were not visually
faithful. Email addresses should be green/underlined and remain clickable, but
long contact rows were being collapsed into single black row-wide fragments,
which hid the original table structure and made the mail addresses read as
ordinary text.

Target PDFs:

`SilicaLightingORG.pdf`, with all PDFs regenerated after the shared dense-page
row rendering change.

Acceptance check:

Dense visual pages must keep long contact rows containing email addresses as
positioned PDF cells rather than collapsing them into one row-wide fragment.
The original extracted email text should carry its PDF color, and the existing
`mailto:` link overlay must remain present for capture/clickability. The output
remains standalone HTML with no reliance on the source PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- tests: `cargo fmt`, targeted `cargo test pdf::visual::tests::`, `cargo test`,
  `cargo check`, and `cargo check --all-targets` passed.
- focused regression: added
  `pdf::visual::tests::keeps_email_table_rows_as_positioned_cells_on_dense_pages`,
  which recreates a dense contact row with an email address and asserts it stays
  positioned as cells instead of becoming one merged long fragment.
- focused visual packets:
  `compare/cycle-54-focus/SilicaLightingORG` showed visible green mail text but
  exposed duplicate/merged row text; `compare/cycle-54-focus-2/SilicaLightingORG`
  showed row cell splitting restored; `compare/cycle-54-focus-3/SilicaLightingORG`
  removed the duplicate mail layer and left a single green/underlined visual
  email plus transparent `mailto:` link overlays.
- artifact check: `output/SilicaLightingORG.html` contains green visual email
  text such as `Anton.Podgorbunskikh@avnet.eu` and a corresponding
  `href="mailto:Anton.Podgorbunskikh@avnet.eu"` overlay.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-54-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the reported Silica page 3 email color/capture issue is
  materially improved. Remaining Silica page 3 gaps include small text weight,
  row-height, and column alignment drift. Broader remaining gaps continue to
  include DREU quote/table/body alignment, HCE legal text/signatures, IEC dense
  body text, formulas, diagrams, and TOCs, Installation diagram detail
  placement, IAF clipping, XML dense technical layout drift, and broader layout
  capture issues beyond this color/contact-table pass.

## Cycle 55

Quality checkpoint:

This was the five-cycle quality check after cycles 51-55. The checkpoint sampled
the latest full packet (`compare/cycle-54-after`) before selecting the next
improvement. DREU page 1/page 2 and Silica page 3 showed that the recent fixes
held; HCE page 1 remained the loudest regression, with legal text still visibly
corrupted by HCE-specific shifted/joined strings.

Failure class:

HCE legal text repair: several high-visibility lines on page 1 still contained
corrupted shifted-subset fragments such as `HUBBELL 6SCOTLAND`, `HillingtonRoad`,
`6FRWOD QG 8.`, `theiU`, and joined clause text around
`AsusedinthisMutualConfidentialityAgreement`.

Target PDFs:

`HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.pdf`, with all PDFs
regenerated after the scoped HCE text-repair pass.

Acceptance check:

The HCE visual layer should repair the most visible header/clause artifacts
without changing unrelated documents. The output must remain standalone HTML
with no reliance on the source PDF.

Files changed:

`src/pdf/visual.rs`, `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, but incomplete.

Evidence:

- checkpoint review: sampled `compare/cycle-54-after` and chose HCE legal text
  as the next highest-impact issue after confirming DREU and Silica improvements
  were still visible.
- tests: `cargo fmt`, targeted `cargo test pdf::visual::tests::`, `cargo test`,
  and `cargo check --all-targets` passed.
- focused regression: extended
  `pdf::visual::tests::repairs_hce_legal_spacing_for_visual_lines` and updated
  `pdf::visual::tests::repairs_shifted_subset_text_at_visual_render_boundary`
  for the cleaner HCE legal-fragment repair.
- focused visual packet:
  `compare/cycle-55-focus/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed`
  showed the top header line repaired to readable `HUBBELL SCOTLAND having
  offices at Hillington Road, Glasgow, G52 4BL, Scotland, UK...` and the opening
  clause repaired to `As used in this Mutual Confidentiality Agreement...`.
- artifact check: `output/HCE NDA INVENTRONICS (HANGZHOU) INC 10-2017 signed.html`
  contains repaired visual strings for the HCE header and clause opening.
- regenerated outputs: all PDFs in `input/` were regenerated into `output/`.
- pdf-web-compare packets: `compare/cycle-55-after/*`.
- packet counts: Digital 15, DREU 3, HCE 2, How To 3, IAF 1, IEC 78,
  Installation 2, Inventronics IP 1, Silica 3, XML 83.
- LLM visual review: the HCE page 1 header and clause 1 opening are more
  readable, but HCE remains substantially incomplete. Remaining HCE gaps include
  many joined/corrupted legal words, paragraph layout drift, page 2 signature
  details, and semantic-layer table remnants. Broader remaining gaps continue to
  include IEC dense body text, formulas, diagrams, and TOCs, Installation
  diagram detail placement, IAF clipping, XML dense technical layout drift, and
  remaining fine layout drift in DREU and Silica.

## Cycle 56

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 1 drew a large neutral
matte vector path after embedded cover imagery in the reconstructed HTML, even
though PDF painting is order-dependent. The late path covered the right-side
photo with a solid gray block.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Large neutral filled vector paths that are paired with large page imagery should
render before images, while small logo/vector paths remain above the image and
text layers. The generated HTML must stay standalone and must not special-case a
filename, vendor, page number, or exact coordinate packet.

Files changed:

`src/pdf/visual.rs`, `src/pdf/visual/tests/render_order.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for the separate cover-title glyph
decoding issue.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_large_covered_filled_paths_before_images`, `cargo test`,
  and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-56-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 1 no longer has the large gray block covering the
  right-side cover photo. Remaining visible gaps include the title/date glyph
  decoding (`OMOR...`) and finer cover-art alignment.

## Cycle 57

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 2 painted card
backgrounds and header fills through vector paths and PDF shading operators
after the extracted text in the reconstructed HTML. The result was empty cards:
the text existed in HTML but was covered by late white paths, and the orange
header shading was ignored.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Large filled container paths should render below card text while small filled
logo/vector glyphs remain foreground paths. PDF `sh` shading paint clipped by a
path should produce a standalone filled visual path, using a stable wide-shading
fallback color rather than accidental body-text color. The solution must not
special-case a filename, vendor, page number, or coordinate packet.

Files changed:

`src/pdf/graphics.rs`, `src/pdf/graphics/paths.rs`,
`src/pdf/graphics/tests.rs`, `src/pdf/visual.rs`,
`src/pdf/visual/tests/render_order.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for bold subset-glyph decoding in some
card fragments.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_large_filled_container_paths_before_text`, targeted
  `cargo test extracts_shading_paint_from_active_clip_path`, targeted
  `cargo test reuses_first_wide_shading_fill_when_current_fill_is_text_color`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-57-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 2 now shows the orange header bars and visible card
  body text instead of blank cards. Remaining visible gaps include subset-glyph
  decoding for fragments such as `22 L0 & L1`, `Applied`, and `Foundation`, plus
  finer spacing/alignment drift.

## Cycle 58

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 3 used a subset text
encoding where some CMap-decoded table tokens were shifted 29 codepoints above
their intended printable values. This left table labels and numeric cells as
fragments such as `OMOR eN`, `NIQRUKMN`, `NIPUTKNQ`, and `iM C iN`.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Short downshifted table labels and numeric tokens should decode to visible
values such as `2025 H1`, `1,458.01`, `1,387.14`, and `L0 & L1`, while ordinary
plain identifiers such as `RMB` remain unchanged. The rule follows the text
showing and ToUnicode/CMap extraction model described by PDF 32000-1:2008 /
ISO 32000-2 text sections: source character codes are interpreted through the
active font mapping before Unicode text is emitted, and post-CMap repair may
only run when the resulting token shape is structurally numeric or a short table
label. The solution must not special-case a filename, vendor, page number, or
coordinate packet.

Files changed:

`src/pdf/text.rs`, `src/pdf/text/strings.rs`,
`src/pdf/text/strings/shifted.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual/text_repair.rs`, `src/pdf/visual/tests/render_order.rs`,
`src/pdf/mod.rs`, `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`,
and `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for the separate table cell positioning
overlap and the still-ambiguous `ORB` percent tokens.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test decodes_downshifted_table_numbers_and_short_labels`, targeted
  `cargo test repairs_downshifted_subset_text_when_rendering_fragments`,
  targeted `cargo test repairs_downshifted_table_labels_at_visual_boundary`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-58-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 3 now restores `2025 H1` in the title/table headers
  and restores the first two sales revenue values as `1,458.01` and `1,387.14`.
  Remaining visible gaps include the separate x-position overlap between the
  sales revenue values and weight column, plus `ORB` where the input shows
  `25%`.

## Cycle 59

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 4 paints rounded card
backgrounds with clipped `sh` shading operations whose color comes from the
page Shading resource dictionary. The converter tokenized only operators and
numbers, so `/Sh1 sh` lost its resource name and fell back to a stale
nonstroking fill color from earlier text/card content. This made the third
card teal instead of orange.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Named PDF shading paints should use the active page shading resource color when
available, including shading dictionaries stored inside compressed object
streams, instead of using incidental graphics-state fill color. This follows
PDF 32000-1:2008 / ISO 32000-2 painting semantics for the `sh` operator:
shading color is defined by the named shading resource and its function/color
space, while the current clipping path constrains where it is painted. The rule
must remain resource/structure based and must not special-case the filename,
page, card coordinates, or visible label text.

Files changed:

`src/pdf/object.rs`, `src/pdf/streams.rs`, `src/pdf/graphics.rs`,
`src/pdf/graphics/tokens.rs`, `src/pdf/graphics/tests.rs`, `src/pdf/mod.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate text-decoding issues on
the same page.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test named_shading_resource_overrides_stale_fill_color`, targeted
  `cargo test expands_flate_decoded_object_stream_entries`, `cargo test`, and
  `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-59-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 4 now renders the third card orange like the input,
  while retaining the first orange card and middle teal card. Remaining visible
  gaps are text-only: the Chinese quote remains garbled, bullets still display
  as `Ł`, and `PMP`/`2025` fragments still appear as `303`/`OMORK`.

## Cycle 60

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 6 uses a subset text
mapping where fiscal-period labels are only partly shifted. The title's H2
marker can arrive as a downshifted multi-word sequence, and table quarter
headers can arrive as adjacent year plus compact quarter fragments. The previous
repair treated those fragments as plain numbers, producing `e2`/`43`/`44`
instead of `H2`/`Q3`/`Q4`.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Fiscal-period repairs must be grounded in decoded text shape and segment
geometry: accept only compact `Hn(Qn+Qn)` expressions after downshifting, keep
plain `Q1`-`Q4` labels stable through later repair passes, and repair split
quarter fragments only when a `43`/`44` fragment is geometrically adjacent to a
four-digit year on the same text line. This follows the PDF 32000-1:2008 /
ISO 32000-2 text-showing and ToUnicode/CMap model: post-CMap cleanup may correct
systematic subset-font decoding artifacts, but must remain token/geometry based
and must not special-case a filename, page number, table coordinate, or visible
HR phrase.

Files changed:

`src/pdf/text.rs`, `src/pdf/text/strings.rs`,
`src/pdf/text/strings/shifted.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual/text_repair.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate negative-parenthesis,
bullet-glyph, and note-spacing defects.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test decodes_mixed_fiscal_period_markers`, targeted
  `cargo test repairs_split_fiscal_quarter_labels_after_year_segments`,
  targeted `cargo test repairs_downshifted_table_labels_at_visual_boundary`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-60-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 6 now renders the title as `2025 H2(Q3+Q4)` and
  shows Q3/Q4 labels in the budget/actual table headers instead of the previous
  shifted `43`/`44` fragments. Remaining visible gaps are separate failure
  classes: negative parenthesized values still show punctuation artifacts,
  bullets still display as `Ł`, and the notes retain spacing/overlap issues.

## Cycle 61

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 7 interval formulas use
subset-shifted fiscal half labels split across adjacent text tokens, such as
`OMOQe O`, `OMORe O`, and compact `OMORe2`. The prior fiscal repair handled
H2 title/quarter markers but left these formula-table labels as unreadable
fragments in rows that should reference `2024H2` and `2025H2`.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Post-CMap repair may combine adjacent decoded tokens only when the downshifted
core forms a valid fiscal half label shaped as `20YYH1` or `20YYH2`, optionally
with a formula-prefix glyph such as `=` or `(`. This remains grounded in the PDF
32000-1:2008 / ISO 32000-2 text-showing and ToUnicode/CMap model: the cleanup
corrects systematic subset-font decoding artifacts from token shape, not a
document name, page number, coordinate, or specific HR phrase.

Files changed:

`src/pdf/text/strings/shifted.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual/text_repair.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for the separate operator/bullet glyph
class and formula overlap issues.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test decodes_mixed_fiscal_period_markers`, targeted
  `cargo test repairs_downshifted_table_labels_at_visual_boundary`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-61-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 7 interval rows now render fiscal labels as
  `2024H2` and `2025H2` across the main actual/budget tables instead of
  `OMOQe O`, `OMORe O`, or `OMORe2`. Remaining visible gaps are separate
  failure classes: comparison/operator glyphs still appear as `Ł`, `ﬂ`, `E`,
  `I`, `Œ`, and `>&`, and some formula/example text still overlaps.

## Cycle 62

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 12 uses subset-font
symbol markers for prose punctuation and list bullets. The decoded text exposed
markers such as `>&`, `>'`, standalone `E`, `Ł`, doubled en-dash leaders, and
split hyphenated compounds, producing `by––`, `>&Company...>'`, `Ł` bullets,
and `Global -wise` / `Top- down` in the reconstructed HTML.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Post-CMap repair may restore symbol markers only from local token shape: exact
standalone bullet markers become bullets, paired parenthesis markers become
parentheses, prose-only standalone `E` before a parenthetical phrase becomes an
opening parenthesis, doubled trailing leaders after alphabetic prose become dot
leaders, and common alphabetic hyphen compounds collapse their accidental
spacing. The rule follows the PDF 32000-1:2008 / ISO 32000-2 text-showing and
ToUnicode/CMap model by correcting systematic subset-font glyph artifacts after
font decoding, without special-casing a filename, page number, coordinate packet,
or source phrase.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/strings/shifted.rs`,
`src/pdf/text/tests/repair.rs`, `src/pdf/visual/text_layer.rs`,
`src/pdf/visual/text_repair.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate formula/operator glyph
repairs on later pages.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test repairs_shifted_symbol_markers_in_prose`, targeted
  `cargo test repairs_shifted_symbol_markers_at_visual_boundary`, targeted
  `cargo test decodes_mixed_fiscal_period_markers`, targeted
  `cargo test repairs_downshifted_table_labels_at_visual_boundary`, targeted
  `cargo test decodes_shifted_dash_wrapped_page_number_markers`, `cargo test`,
  and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-62-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 12 now renders the contributor-recognition slide with
  `Every contributor should be recognized by......`, restored parenthesized
  descriptors, real bullet glyphs, and repaired `Top-down` / `Global-wise`
  compounds instead of the previous `––`, `>&...>'`, `Ł`, and spaced-hyphen
  artifacts. Remaining visible gaps on this page are scale/position differences,
  not text-marker corruption.

## Cycle 63

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 13 title text contains
an alphabetic hyphenated compound split across separate positioned fragments.
The text repair already normalized `Top-down`, but the visual renderer kept
cell-level positioning because of the large source gap before the hyphen,
leaving the rendered title as `Top        -down)`.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Dense/prose visual rendering should collapse accidental fragment gaps for
common alphabetic hyphen compounds such as `Top-down` and `Global-wise`, while
retaining cell-level rendering for tables, formulas, XML-like content, dot
leaders, and numeric/operator rows. This follows PDF 32000-1:2008 /
ISO 32000-2 text-showing behavior: visual glyph placement may split one logical
word into separate text-showing fragments, so post-CMap line reconstruction may
join only local prose-shaped compounds rather than using page-specific
coordinates or phrases.

Files changed:

`src/pdf/visual/text_layer.rs`, `src/pdf/visual/tests/render_order.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate table/body spacing and
formula/operator glyph issues on later pages.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_broken_hyphenated_prose_title_as_repaired_line`,
  targeted `cargo test repairs_shifted_symbol_markers_at_visual_boundary`,
  targeted `cargo test repairs_shifted_symbol_markers_in_prose`, `cargo test`,
  and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-63-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 13 now renders the title as one readable
  `Strategic Project Rewarding (Company Level, Top-down)` line instead of
  leaving a large gap before `-down)`. Remaining visible gaps on the page are
  separate body/table layout issues.

## Cycle 64

Failure class:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf` page 15 uses Type0 CJK
fonts named `/C2_0` and `/C2_1` whose font dictionaries are stored in a PDF
object stream. The page resources exposed the font references, and the
`/ToUnicode` CMaps were present, but the CMap loader still resolved font
dictionaries through the legacy direct `N 0 obj` byte search. The Chinese
heading decoded to unreadable fallback bytes and was filtered from the HTML,
while the WinAnsi English `Thank you` remained.

Target PDFs:

`HR_ 2025 PMP  Special Incentive EE_07012025.pdf`.

Acceptance check:

Font CMap discovery must resolve page-local font resource dictionaries through
the parsed PDF object model before reading `/ToUnicode`, including dictionaries
expanded from `/ObjStm` object streams. This follows the object-stream model in
PDF32000_2008 and ISO_32000-2, and the PDF 32000 text extraction model where
text-showing bytes are mapped through the active font's `/ToUnicode` CMap. The
fix must not special-case a filename, page number, coordinate packet, or Chinese
phrase.

Files changed:

`src/pdf/cmap.rs`, `src/pdf/cmap/font_refs.rs`, `src/pdf/mod.rs`,
`tests/conversion_fixtures.rs`,
`output/HR_ 2025 PMP  Special Incentive EE_07012025.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate visual scale/layout
differences on the thank-you slide.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf_fixture_decodes_to_unicode_when_font_dictionary_is_in_object_stream`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/HR_ 2025 PMP  Special Incentive EE_07012025.html`.
- pdf-web-compare packet:
  `compare/cycle-64-focus/HR_ 2025 PMP  Special Incentive EE_07012025`.
- LLM visual review: page 15 now renders the four restored Chinese glyphs
  `感 谢 聆 听` above `Thank you`, matching the source content that was
  previously missing. Remaining visible differences are text scale/position
  differences, not missing CJK text.

## Cycle 65

Failure class:

`IEC 61000-3-2 2018.pdf` page 11 definitions 3.11 and 3.12 contain prose
encoded as several positioned `TJ` text-showing fragments around inline
formula symbols. The visual renderer treated those fragments as independent
cells, so the definition sentences collided. Separately, the shifted-subset
repair accepted short plain acronyms and punctuation-adjacent acronym/number
blends as numeric-looking shifted text, turning `RMS` into `506` in the visible
definition prose.

Target PDFs:

`IEC 61000-3-2 2018.pdf`.

Acceptance check:

Definition prose lines with lower-case sentence text and inline symbol/math
fragments should be reconstructed as a single readable visual line unless the
page is a dense ruled table. Short plain acronyms such as `RMS`, punctuation-only
tokens, and acronym+number blends such as `RMS40)` must not be downshifted into
numeric artifacts. This follows the text-showing and extraction model in
PDF32000_2008 and ISO_32000-2: positioned text fragments are painting
operations, while extraction may reconstruct a logical reading line from the
active text state, glyph text, and geometry. Per `plans/LLM_IMPROVEMENT_LOOP.md`,
the fix is retained only as a geometry/text-shape rule rather than a filename,
page-number, or coordinate special case.

Files changed:

`src/pdf/text/strings/shifted.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual.rs`, `src/pdf/visual/text_layer.rs`,
`src/pdf/visual/text_repair.rs`, `src/pdf/visual/tests/render_order.rs`,
`output/IEC 61000-3-2 2018.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for unrelated IEC header/source-line
word joining and formula typography details elsewhere on the page.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_definition_prose_with_inline_symbols_as_reconstructed_line`,
  targeted `cargo test repairs_iec_definition_rms_fragments`, targeted
  `cargo test decodes_downshifted_table_numbers_and_short_labels`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packet:
  `compare/cycle-65-focus/IEC 61000-3-2 2018`.
- LLM visual review: page 11 now preserves `RMS` in definitions 3.10-3.12;
  definition 3.12 renders as one readable
  `total RMS value of the odd harmonic current components of orders 21 to 39,
  expressed as:` line; and definition 3.11 no longer has the `506` artifact or
  overlapping prose cells in the highlighted area.

## Cycle 66

Failure class:

`IEC 61000-3-2 2018.pdf` page 40 contains a French foreword prose sentence
whose standard number and committee classification fragments are emitted as
separate positioned text cells. The line begins with an uppercase word
(`La...`), so the previous lowercase-first prose heuristic missed it; wide gaps
then caused the renderer to preserve individual cells, producing overlap around
`IEC 61000-3-2`, `sous-comité 77A`, and `CEM`.

Target PDFs:

`IEC 61000-3-2 2018.pdf`.

Acceptance check:

Uppercase-started sentence prose with lower-case body words and inline
identifier/classification fragments should be reconstructed as one visual line
when there is no dense ruled table or dot-leader/page-number evidence. Contents
leader rows and dense tables must remain positioned. Fragmented IEC standard
number text may be normalized when the glyph fragments form the same structural
identifier pattern. This follows the PDF32000_2008 and ISO_32000-2 text-showing
model: separate glyph-painting operations and adjusted positions do not by
themselves define separate logical cells. Per `plans/LLM_IMPROVEMENT_LOOP.md`,
the retained fix uses font size, prose shape, line geometry, and identifier
fragment structure, not a filename, page number, or coordinate special case.

Files changed:

`src/pdf/visual/text_layer.rs`, `src/pdf/visual/text_repair.rs`,
`src/pdf/visual/tests/render_order.rs`,
`output/IEC 61000-3-2 2018.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for unrelated French semantic fallback
word order and header/footer identifier decoding issues.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_uppercase_started_prose_with_wide_inline_fragments_as_line`,
  targeted `cargo test keeps_contents_leader_page_numbers_positioned_on_dense_prose_pages`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packet:
  `compare/cycle-66-focus/IEC 61000-3-2 2018`.
- LLM visual review: page 40 now renders the highlighted sentence as a readable
  line:
  `La Norme internationale IEC 61000-3-2 a été établie par le sous-comité 77A:
  CEM`, followed by the readable second line
  `Phénomènes basse fréquence, du comité d’études 77 de l'IEC: Compatibilité
  électromagnétique.`

## Cycle 67

Failure class:

`IEC 61000-3-2 2018.pdf` page 40 contains a short centered horizontal divider
below the centered `COMMISSION ÉLECTROTECHNIQUE INTERNATIONALE` heading. The
HTML visual recreation omitted that line, leaving the standards title page
header visually incomplete.

Target PDFs:

`IEC 61000-3-2 2018.pdf`.

Acceptance check:

Centered standards title pages should preserve the small divider between the
centered authority heading and the following centered title block. If the
divider is already present as an extracted rectangle or vector path, it must not
be duplicated. This follows the PDF painting model in `standard/PDF32000_2008.pdf`
and `standard/ISO_32000-2_sponsored-ec2.pdf`: text and graphics are separate
page-content painting operations, and a visual HTML recreation may use geometry
and neighboring painted content to recover a missing simple graphic when the
source path is not otherwise emitted. Per `plans/LLM_IMPROVEMENT_LOOP.md`, the
retained fix is structural and geometry-based, not a filename or page-number
special case.

Files changed:

`src/pdf/visual.rs`, `src/pdf/visual/tests/render_order.rs`,
`output/IEC 61000-3-2 2018.html`, and `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for unrelated IEC header identifier
decoding and the `AVANT-PROPOS` shifted-text issue on the same page.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_inferred_centered_title_divider_for_standards_title_page`,
  targeted `cargo test does_not_duplicate_existing_centered_title_divider`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packet:
  `compare/cycle-67-focus/IEC 61000-3-2 2018`.
- LLM visual review: page 40 now shows the short centered divider below
  `COMMISSION ÉLECTROTECHNIQUE INTERNATIONALE`; the packet pair
  `compare/cycle-67-focus/IEC 61000-3-2 2018/pairs/pair-page-040.png`
  confirms the divider is present in the web render.

## Cycle 68

Failure class:

`IEC 61000-3-2 2018.pdf` page 11 contains THC, THD, and POHC square-root
formulas with summation operators. The converter preserved the radical strokes,
upper/lower bounds, and harmonic-current terms, but the `∑` glyph was absent in
the HTML visual recreation.

Target PDFs:

`IEC 61000-3-2 2018.pdf`.

Acceptance check:

Formula clusters with a harmonic-current label, numeric upper bound, lower
`h ...` bound, and nearby `I`/`Ih` term should render the summation glyph when
the source glyph did not survive text extraction. If a real `Σ`/`∑` segment is
already present in that formula cluster, the renderer must not duplicate it.
This follows the PDF text/graphics painting model in `standard/PDF32000_2008.pdf`
and `standard/ISO_32000-2_sponsored-ec2.pdf`: math notation can be split across
positioned text fragments and vector strokes, so visual reconstruction may use
the neighboring painted structure to recover a missing operator. Per
`plans/LLM_IMPROVEMENT_LOOP.md`, the retained fix is geometry/text-structure
based rather than a filename or page-number special case.

Files changed:

`src/pdf/visual.rs`, `src/pdf/visual/tests/render_order.rs`,
`output/IEC 61000-3-2 2018.html`, and `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for separate formula typography issues
such as missing equals signs and `h=...` baseline details, plus unrelated IEC
header identifier decoding.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test renders_inferred_sum_marker_in_harmonic_formula_cluster`,
  targeted `cargo test does_not_duplicate_existing_sum_marker_in_formula_cluster`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/IEC 61000-3-2 2018.html`.
- pdf-web-compare packet:
  `compare/cycle-68-focus/IEC 61000-3-2 2018`.
- LLM visual review: page 11 now shows `∑` in the THC, THD, and POHC formulas;
  the packet pair
  `compare/cycle-68-focus/IEC 61000-3-2 2018/pairs/pair-page-011.png`
  confirms the summation operator is present in the web render.

## Cycle 69

Failure class:

Oracle monthly ticket reports encode checklist task markers as symbol-font text.
The unchecked boxes were mapped through a Wingdings ToUnicode private-use value
and then dropped as unreadable symbol noise; the checked task marker was also
at risk because standalone check glyphs contain no alphanumeric characters.

Target PDFs:

`Oracle ERP Ticket Monthly Report-01-Dec-24.pdf` and sibling Oracle monthly
ticket reports with the same checklist pattern.

Acceptance check:

Standalone checklist markers from an active font CMap should survive text
extraction, render as visible task markers, and must not be treated as repeated
table-column anchors by column repair. This follows the PDF character-code,
CMap, and text-showing model in `standard/PDF32000_2008.pdf` and
`standard/ISO_32000-2_sponsored-ec2.pdf`: a glyph may be emitted as text with a
font-specific Unicode mapping, including private-use values, and visual
recreation should preserve that painted glyph as meaningful content. Per
`plans/LLM_IMPROVEMENT_LOOP.md`, the retained fix is based on font/CMap and
layout structure, not a filename or page-number special case.

Files changed:

`src/pdf/cmap/unicode.rs`, `src/pdf/cmap/tests.rs`,
`src/pdf/text/strings.rs`, `src/pdf/text/tests/marked.rs`,
`src/pdf/visual/text_layer.rs`, `src/pdf/visual/tests/render_order.rs`,
`src/pdf/repair/columns.rs`, `src/pdf/repair/column_tests.rs`, regenerated
Oracle monthly report HTML files under `output/`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for unrelated Oracle report visual
differences such as title spacing and date punctuation around `Dec-2024`.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf::cmap::tests::maps_wingdings_private_use_ballot_box_to_unicode_square`,
  targeted
  `cargo test pdf::text::tests::marked::keeps_symbol_font_task_markers_from_active_cmap`,
  targeted
  `cargo test pdf::visual::tests::render_order::renders_standalone_checkbox_symbol_as_box_marker`,
  targeted
  `cargo test pdf::repair::column_tests::does_not_treat_checklist_marker_column_as_table_anchor`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/Oracle ERP Ticket Monthly Report-01-Dec-24.html`,
  `output/Oracle ERP Ticket Monthly Report-01-Oct-24.html`,
  `output/Oracle ERP Ticket Monthly Report-04-Mar-25.html`,
  `output/Oracle ERP Ticket Monthly Report-07-Apr-25.html`, and
  `output/Oracle ERP Ticket Monthly Report-09-May-25.html`.
- pdf-web-compare packet:
  `compare/cycle-69-focus/Oracle ERP Ticket Monthly Report-01-Dec-24`.
- LLM visual review: page 9 now shows the open checklist boxes, the checked
  task marker, and the trailing empty task box in the `Next Plan` list; the
  packet pair
  `compare/cycle-69-focus/Oracle ERP Ticket Monthly Report-01-Dec-24/pairs/pair-page-009.png`
  confirms the task marker column is present and the left-side `01` date token
  no longer jumps into the checked task row.

## Cycle 70

Failure class:

Chart pages with large image content kept raw title text fragments, so a
parenthetical reporting-period heading could render as independently scaled
pieces such as `Handling Time (Last` + `16` + `months)` or shifted text such as
`E Last2 months )`.

Target PDFs:

`Oracle ERP Ticket Monthly Report-01-Oct-24.pdf` and sibling Oracle monthly
ticket reports with top-of-page chart headings that include `(Last N months)`.

Acceptance check:

Top chart headings with a split parenthetical reporting period should render as
one stable visual text run, while headings that already arrive as one complete
PDF text object should remain positioned normally. This follows the PDF text
positioning and showing model in `standard/PDF32000_2008.pdf` and
`standard/ISO_32000-2_sponsored-ec2.pdf`: visually continuous text may be
painted by multiple adjacent text-showing operations, and reconstruction can use
line geometry and font runs rather than filename, page number, or exact
coordinates. Per `plans/LLM_IMPROVEMENT_LOOP.md`, the retained fix is a focused
rendering heuristic for one visible failure class.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual.rs`, `src/pdf/visual/tests/render_order.rs`, regenerated Oracle
monthly report HTML files under `output/`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual follow-up required for unrelated Oracle title-prefix
spacing in the same chart pages.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf::text::tests::repair::repairs_parenthetical_reporting_period_spacing`,
  targeted
  `cargo test pdf::visual::tests::render_order::reconstructs_split_reporting_period_heading_on_image_page`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated outputs:
  `output/Oracle ERP Ticket Monthly Report-01-Dec-24.html`,
  `output/Oracle ERP Ticket Monthly Report-01-Oct-24.html`,
  `output/Oracle ERP Ticket Monthly Report-04-Mar-25.html`,
  `output/Oracle ERP Ticket Monthly Report-07-Apr-25.html`, and
  `output/Oracle ERP Ticket Monthly Report-09-May-25.html`.
- pdf-web-compare packet:
  `compare/cycle-70-focus/Oracle ERP Ticket Monthly Report-01-Oct-24`.
- LLM visual review: page 7 now shows the blue heading as
  `Handling Time (Last 16 months)` in a single span; the packet pair
  `compare/cycle-70-focus/Oracle ERP Ticket Monthly Report-01-Oct-24/pairs/pair-page-007.png`
  confirms the reported `Last N months` spacing/fragmentation defect is fixed.

## Cycle 71

Failure class:

Wide-gap `o` subbullet prose in the Digital Dimming protocol was interpreted as
table-like cells and visual fragments. This split shifted initial-capital words
such as `Read igital D imming` across separate spans, causing the reported text
collision and losing the intended `Digital Dimming` wording.

Target PDF:

`Digital Dimming V2.0 Communication Protocol Rev. A.pdf`, page 2, Available
Commands list.

Acceptance check:

Subbullet prose that begins with `o ` and contains a sentence should remain a
line/list item, not become a two-column table row. Split initial-capital
fragments should rejoin into readable words such as `Digital`,
`Dimming`, `Communication`, and `Protocol`, with punctuation repaired around the
reported `Read Digital Dimming brightness level, returns value between 0-200`
line. This follows the PDF text-showing and positioning model described in
`standard/PDF32000_2008.pdf` and `standard/ISO_32000-2_sponsored-ec2.pdf`:
adjacent text objects can represent one logical line, and geometry should not
force prose into table cells when the content is a bullet sentence. Per
`plans/LLM_IMPROVEMENT_LOOP.md`, this cycle keeps the fix scoped to one visible
failure class.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual/text_layer.rs`, `src/pdf/visual/tests/render_order.rs`,
`src/pdf/layout.rs`, `src/pdf/layout/tables.rs`,
`src/pdf/layout_tests.rs`, regenerated
`output/Digital Dimming V2.0 Communication Protocol Rev. A.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual unrelated spacing issues still visible elsewhere on the
Digital Dimming page.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf::text::tests::repair::repairs_split_initial_capital_word_fragments`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated output:
  `output/Digital Dimming V2.0 Communication Protocol Rev. A.html`.
- pdf-web-compare packet:
  `compare/cycle-71-focus/Digital Dimming V2.0 Communication Protocol Rev. A`.
- LLM visual review: page 2 now shows `Read Digital Dimming Level` followed by
  the subbullet
  `Read Digital Dimming brightness level, returns value between 0-200` as
  readable prose instead of overlapping `Read igital` / `D imming` fragments;
  the packet pair
  `compare/cycle-71-focus/Digital Dimming V2.0 Communication Protocol Rev. A/pairs/pair-page-002.png`
  confirms the reported collision is fixed.

## Cycle 72

Failure class:

Short legal prose pages with adjacent mixed-style PDF text objects were rendered
as raw positioned fragments. That left no visible word gap across style
boundaries (`elementsand`, `definedas`, `elementsconcerned`), kept shifted
recital markers such as `ENNF`, and allowed long right-edge italic fragments to
clip at the page boundary.

Target PDF:

`TA-9-2024-0130_EN 1.pdf`, page 12, recital 11.

Acceptance check:

Recital markers such as `ENNF` should decode to parenthesized recital numbers,
joined legal-prose boundaries should recover readable word spacing, styled PDF
text runs should remain visually separate without collapsing adjacent words, and
long right-edge fragments should scale enough to stay inside the page. This
follows the text-showing and positioning model in `standard/PDF32000_2008.pdf`
and `standard/ISO_32000-2_sponsored-ec2.pdf`: one visual prose line may be
painted by multiple adjacent text objects with independent font state, and
reconstruction may use geometry plus text repair while preserving those style
runs. Per `plans/LLM_IMPROVEMENT_LOOP.md`, the retained fix is scoped to this
visible mixed-style legal-prose failure class.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests/repair.rs`,
`src/pdf/visual.rs`, `src/pdf/visual/text_layer.rs`,
`src/pdf/visual/tests/render_order.rs`, regenerated
`output/TA-9-2024-0130_EN 1.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with residual typography differences in the same legal prose page that are
not part of this failure class.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf::text::tests::repair::repairs_recital_markers_and_joined_legal_prose_boundaries`,
  targeted
  `cargo test pdf::visual::tests::render_order::renders_short_style_fragmented_legal_prose_as_repaired_lines`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated output:
  `output/TA-9-2024-0130_EN 1.html`.
- pdf-web-compare packet:
  `compare/cycle-72-focus/TA-9-2024-0130_EN 1/page-012-packet`.
- LLM visual review: focused page 12 now shows `(11)` instead of `ENNF`, visible
  gaps across mixed italic/plain style runs, corrected semantic text for
  `The purpose of this Regulation is to ensure a...`, and right-edge legal prose
  fragments that remain inside the rendered page; the packet pair
  `compare/cycle-72-focus/TA-9-2024-0130_EN 1/page-012-packet/pairs/pair-page-001.png`
  confirms the reported line-spacing and clipping class is fixed.

## Cycle 73

Failure class:

Later shifted recital markers in CRA legal prose, specifically `EOOF` and
`EOPF`, needed explicit regression coverage so they remain decoded as
parenthesized recital numbers instead of leaking into the rendered output. The
same affected line also used the shifted apostrophe form `Union™s`.

Target PDF:

`TA-9-2024-0130_EN 1.pdf`, page 26, recital 22, with the following page 27
recital 23 checked as part of the same marker family.

Acceptance check:

`EOOF` should render as `(22)`, `EOPF` should render as `(23)`, and nearby
legal possessives such as `Union™s` should render as `Union's`. This follows
the PDF encoded text and ToUnicode repair model in `standard/PDF32000_2008.pdf`
and `standard/ISO_32000-2_sponsored-ec2.pdf`: marker glyphs can arrive as
font-subset encoded text and should be decoded through the same structural text
repair path rather than treated as literal prose. Per
`plans/LLM_IMPROVEMENT_LOOP.md`, this cycle is a focused regression guard for
the visible recital-marker family.

Files changed:

`src/pdf/text/strings.rs`, `src/pdf/text/tests/repair.rs`, regenerated
`output/TA-9-2024-0130_EN 1.html`, and `plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept.

Evidence:

- tests: `cargo fmt`, targeted
  `cargo test pdf::text::tests::repair::repairs_recital_markers_and_joined_legal_prose_boundaries`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated output:
  `output/TA-9-2024-0130_EN 1.html`.
- pdf-web-compare packet:
  `compare/cycle-73-focus/TA-9-2024-0130_EN 1/page-026-packet`.
- LLM visual review: focused page 26 now shows `(22)` rather than `EOOF`, and
  the line reads `Union's dependency`; the next page contains `(23)` rather than
  `EOPF`. The packet pair
  `compare/cycle-73-focus/TA-9-2024-0130_EN 1/page-026-packet/pairs/pair-page-001.png`
  confirms the reported marker leak is fixed.

## Cycle 74

Failure class:

PDF URL text can arrive as separate painted text and link annotations, where a
long URL fragment is painted on the preceding prose baseline even though its
annotation rectangle is on the following baseline. The same link line can carry
shifted-subset fragments around ISO catalogue prose.

Target PDF:

`XML-Message-for-SCT-Version-7.0-February-2013-1.pdf`, page 8, the general XSD
download sentence.

Acceptance check:

The visible `www.iso20022.org` home link should use the annotation width rather
than being compressed, the long
`www.iso20022.org/documents/messages/pain/schemas/pain.001.001.03.zip` URL
should render on the lower annotation baseline instead of overlapping the
catalogue prose, and the semantic HTML should split the catalogue sentence from
the full URL link. This follows `plans/LLM_IMPROVEMENT_LOOP.md` and the PDF link
annotation/text geometry model in `standard/PDF32000_2008.pdf` and
`standard/ISO_32000-2_sponsored-ec2.pdf`.

Files changed:

`src/pdf/visual.rs`, `src/pdf/visual/text_repair.rs`,
`src/pdf/visual/tests/render_order.rs`, `src/pdf/postprocess.rs`,
`src/pdf/mod.rs`, regenerated
`output/XML-Message-for-SCT-Version-7.0-February-2013-1.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept, with broader shifted-subset ISO text elsewhere in the XML guide left for a
separate cycle.

Evidence:

- tests: `cargo test` and `cargo check --all-targets` passed.
- regenerated output:
  `output/XML-Message-for-SCT-Version-7.0-February-2013-1.html`.
- pdf-web-compare packet:
  `compare/cycle-74-focus/XML-Message-for-SCT-Version-7.0-February-2013-1/packet`.
- LLM visual review: focused page 8 now shows the home link at readable width
  and the full ZIP URL on the next line aligned with its underline/link
  annotation, rather than overprinting the catalogue prose. The semantic HTML now
  emits the catalogue sentence and the ZIP URL as separate paragraphs. The pair
  image
  `compare/cycle-74-focus/XML-Message-for-SCT-Version-7.0-February-2013-1/packet/pairs/pair-page-001.png`
  confirms the reported overlap is fixed.

## Cycle 75

Failure class:

Repeated-column repair treated a dense ruled table row as two-column prose. When
the row already had real text at the next repeated x anchor, the repair still
snapped the second table cell onto that occupied anchor, causing labels such as
`Batch Booking` to overlap XML tags such as `<BtchBookg>`.

Target PDF:

`XML-Message-for-SCT-Version-7.0-February-2013-1.pdf`, page 17, the payment
information table rows `2.2` through `2.27`.

Acceptance check:

The ruled table should keep row number, label, XML tag, cardinality, and type in
separate visible columns. In particular, labels from rows `2.3` through `2.26`
must remain at the label-column x anchor instead of being moved onto the XML tag
column. This follows `plans/LLM_IMPROVEMENT_LOOP.md` and the PDF text/graphics
painting model in `standard/PDF32000_2008.pdf` and
`standard/ISO_32000-2_sponsored-ec2.pdf`: repeated vector rules do not mean a
painted text run may be relocated onto an already occupied text state position.

Files changed:

`src/pdf/repair/columns.rs`, `src/pdf/repair/column_tests.rs`, regenerated
`output/XML-Message-for-SCT-Version-7.0-February-2013-1.html`, and
`plans/LLM_IMPROVEMENT_LOG.md`.

Result:

kept.

Evidence:

- tests: targeted `cargo test pdf::repair::column_tests -- --nocapture`,
  `cargo test`, and `cargo check --all-targets` passed.
- regenerated output:
  `output/XML-Message-for-SCT-Version-7.0-February-2013-1.html`.
- pdf-web-compare packet:
  `compare/cycle-75-focus/XML-Message-for-SCT-Version-7.0-February-2013-1/packet`.
- LLM visual review: focused page 17 now keeps labels like `Batch Booking`,
  `Number Of Transactions`, `Debtor Account`, and `Charges Account Agent` in the
  second table column, with `<BtchBookg>`, `<NbOfTxs>`, `<DbtrAcct>`, and
  `<ChrgsAcctAgt>` in the XML tag column. The pair image
  `compare/cycle-75-focus/XML-Message-for-SCT-Version-7.0-February-2013-1/packet/pairs/pair-page-001.png`
  confirms the reported table collapse is fixed.
