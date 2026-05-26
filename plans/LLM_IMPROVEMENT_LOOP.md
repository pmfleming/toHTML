# LLM Improvement Loop

This loop is the operating protocol for finishing PDF recreation in HTML. It is
deliberately evidence-driven: each cycle chooses one visible gap, implements one
targeted improvement, regenerates outputs, and keeps the change only when a
visual comparison plus LLM review shows a real improvement.

## Definition of Done

The project is done when every PDF in `input/` has a freshly generated,
same-stem HTML file in `output/` with limited CSS, and every output is visually
indistinguishable from its input PDF when reviewed through `pdf-web-compare`.
The retained HTML must stand on its own: once generated, it must not rely on the
original PDF, rendered PDF page images, JavaScript, or hidden source-PDF access
to appear correct.

Limited CSS means CSS is allowed only when it makes the recreated document more
readable or inspectable and the same result cannot be expressed with semantic
HTML alone. CSS must stay narrowly scoped to the generated PDF output, must not
require JavaScript, and must not hide content-loss or extraction uncertainty.

For each PDF, done requires:

1. The HTML preserves the meaningful text content.
2. Reading order is understandable without opening the source PDF.
3. Headings, paragraphs, lists, tables, links, figures, page breaks, and warnings
   are represented when the source contains reliable evidence for them.
4. Non-extractable or uncertain content is called out explicitly.
5. `pdf-web-compare` visual packets plus LLM review show the output is visually
   indistinguishable from the input PDF, with no known regression against the
   previous retained output.

## Cycle Loop

Each improvement cycle works on exactly one area.

1. Review input and output for differences.
   - Compare the source PDF in `input/` with the generated HTML in `output/`.
   - Use `side-projects/pdf-web-compare` as the primary comparison tool.
   - Review the generated side-by-side page images and LLM prompt.
   - Prefer high-impact gaps that affect multiple PDFs or make one PDF
     unreadable.

2. Isolate one area for improvement.
   - Name the failure class in one sentence.
   - Identify the smallest source modules likely responsible.
   - Define an acceptance check before editing.

3. Implement the improvement.
   - Keep the change narrow.
   - Add or update focused tests when the behavior can be reduced to a fixture.
   - Avoid unrelated cleanup inside the same cycle.

4. Regenerate output and comparison artifacts.
   - Recreate affected HTML files in `output/`.
   - Recreate relevant `pdf-web-compare` visual packets.
   - If the change touches shared PDF behavior, regenerate all PDF outputs.

5. Compare before and after with visual evidence and an LLM review.
   - Keep the change only if it improves the targeted difference and does not
     regress other sampled PDFs.
   - Use the generated `llm-manifest.json` and `llm-prompt.md` as the LLM
     handoff. The LLM review must inspect the side-by-side images, not only text.
   - Rework or revert changes that only move the defect around.
   - Record the result in the cycle log.

6. Repeat steps 1 through 5 as often as necessary. Continue cycling until every
   PDF in `input/` has a same-stem output in `output/` and all generated outputs
   are visually indistinguishable from their corresponding input PDFs without
   relying on the original PDFs.

## Five-Cycle Quality Run

After every five retained or attempted cycles, pause feature work and run a
quality pass. The quality pass must improve the project against:

1. Cognitive, cyclomatic, and effort metrics.
2. Leverage and locality.
3. Clone count and total lines of code.

Use the current `rqlens.toml` contract:

```powershell
$env:PYTHONPATH = "C:\Code\rust-quality-lens\src"
python -m rust_quality_lens.cli catalog --config rqlens.toml
python -m rust_quality_lens.cli measure all --config rqlens.toml
```

Review these artifacts first:

1. `target/analysis/hotspots.json`
2. `target/analysis/clones.json`
3. `target/analysis/leverage_metrics.json`
4. `target/analysis/locality_metrics.json`
5. `target/analysis/type_health.json`

Quality work follows the same keep-only-improvements rule. Refactors are kept
only when tests still pass, output comparisons do not regress, and at least one
quality signal improves or a clear risk is removed.

## Standard Checks

Run these before keeping a cycle that changes Rust behavior:

```powershell
cargo test
cargo check --all-targets
```

Run the converter for the affected PDFs. For a single file:

```powershell
cargo run -- "input\How To Program a Driver.pdf" --output "output\How To Program a Driver.html"
```

For broad PDF behavior, regenerate all current PDF outputs:

```powershell
Get-ChildItem input -Filter *.pdf | ForEach-Object {
  $out = Join-Path "output" ($_.BaseName + ".html")
  cargo run -- $_.FullName --output $out
}
```

Then rebuild comparison artifacts:

```powershell
.\side-projects\pdf-web-compare\.venv\Scripts\python.exe `
  side-projects\pdf-web-compare\pdf_web_compare_app.py `
  --pdf "input\How To Program a Driver.pdf" `
  --web "output\How To Program a Driver.html" `
  --output "compare\cycle-N\How To Program a Driver"
```

For broad PDF behavior or a final completion check, generate one fresh visual
packet per PDF/HTML pair:

```powershell
Get-ChildItem input -Filter *.pdf | ForEach-Object {
  $html = Join-Path "output" ($_.BaseName + ".html")
  $packet = Join-Path "compare\cycle-N" $_.BaseName
  .\side-projects\pdf-web-compare\.venv\Scripts\python.exe `
    side-projects\pdf-web-compare\pdf_web_compare_app.py `
    --pdf $_.FullName `
    --web $html `
    --output $packet
}
```

Each packet must contain:

1. `pdf/pdf-page-001.png` and following PDF page renders.
2. `web/web-page-001.png` and following browser-rendered HTML slices.
3. `pairs/pair-page-001.png` and following side-by-side images.
4. `llm-manifest.json`.
5. `llm-prompt.md`.
6. `visual-report.html`.

Use `python render_pdfs.py` and `python compare_render.py` only as fallback
manual aids when `pdf-web-compare` cannot run.

## Cycle Log Template

Create one entry per cycle. Keep it short and factual.

```markdown
## Cycle N

Failure class:

Target PDFs:

Acceptance check:

Files changed:

Result:
- kept / reverted / narrowed and retried

Evidence:
- tests:
- regenerated outputs:
- pdf-web-compare packets:
- LLM visual review:
- quality notes, if this was a fifth-cycle quality run:
```

## Stop Rules

Stop the current cycle and narrow the attempt when any of these happen:

1. The targeted PDF improves but another representative PDF gets worse.
2. The change needs broad CSS, JavaScript, or hidden content to appear correct.
3. Text content is lost without an explicit warning.
4. Complexity climbs in a parser or layout module without reducing a larger
   failure class.
5. The improvement cannot be described as one failure class anymore.
6. The LLM visual review cannot identify a clear before/after improvement from
   the `pdf-web-compare` packet.

When a stop rule fires, record the failed attempt and restart the cycle with a
smaller hypothesis.
