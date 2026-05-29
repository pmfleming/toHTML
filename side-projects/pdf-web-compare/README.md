# PDF/Web Visual Compare

Small side-project app for a tool chain where an LLM compares images:

- render each PDF page to PNG
- render the webpage in Chromium to PNG
- slice the webpage screenshot into PDF-page-sized images
- create side-by-side page images
- write an LLM-ready manifest and prompt
- write a local HTML visual report for quick inspection

It also includes an interactive React mode for manually walking through local
files page by page:

- files in the main project's `C:\Code\toHTML\input` appear on the left
- files in the main project's `C:\Code\toHTML\output` appear on the right
- files are paired by matching base filename, such as `example.pdf` and
  `example.html`
- each matched pair is paged together, so page 1 on the left shows page 1 of the
  matching file on the right
- when one matched file ends, the next page is the first page of the next
  matched file
- the header shows total comparable pages across all files

The app code and dependencies are isolated in this side-project folder. The
interactive mode deliberately reads from the main `toHTML` project's `input/`
and `output/` folders. Generated Python visual packets are still written to this
folder's own `reports/` directory unless you pass a custom output directory.

## Setup

```powershell
cd C:\Code\toHTML\side-projects\pdf-web-compare
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install --cache-dir .pip-cache -r requirements.txt
$env:PLAYWRIGHT_BROWSERS_PATH = "$PWD\.playwright-browsers"
.\.venv\Scripts\python.exe -m playwright install chromium
npm install
```

For interactive mode, put PDFs, HTML files, or image files in the main project
folders before launching the UI:

```text
C:\Code\toHTML\input
C:\Code\toHTML\output
```

## Interactive React Mode

```powershell
npm run dev
```

Open:

```text
http://127.0.0.1:5177
```

The local server scans the main project folders recursively and serves the files
to the React UI. The UI renders PDFs with pdf.js, slices HTML documents into
page-sized viewport sections, and shows image files as one-page items.

Typical workflow:

1. Put PDFs in `C:\Code\toHTML\input`.
2. Run `npm run dev`.
3. Open `http://127.0.0.1:5177`.
4. Start reviewing immediately while background generation runs.
5. Page through the matched PDF/HTML pairs as generated files appear.

Use the `Input` folder field in the toolbar to compare a different source
folder. Enter a path and click `Apply`, or click `Choose` to open a native
folder picker on Windows. Relative paths are resolved from the main `toHTML`
project root. The `Output` folder stays fixed at `C:\Code\toHTML\output`.

On load, the app scans every PDF in the main `input` folder, regenerates matching
HTML into the main `output` folder, and shows progress without blocking the
viewer. This intentionally does not skip existing output because `toHTML` may be
improving during development. Use `Generate output` to start the background
generation again. PDF conversion embeds extractable PDF image XObjects by
default so the generated HTML remains standalone without loading the source PDF.

Build check:

```powershell
npm run build
```

## GUI

```powershell
.\.venv\Scripts\python.exe pdf_web_compare_app.py
```

Choose a PDF, enter a webpage URL or select a local HTML file, then click
`Compare`.

## Visual CLI

Visual mode is the default:

```powershell
.\.venv\Scripts\python.exe pdf_web_compare_app.py --pdf C:\Docs\source.pdf --web https://example.com/page
.\.venv\Scripts\python.exe pdf_web_compare_app.py --pdf C:\Docs\source.pdf --web C:\Docs\page.html --open
.\.venv\Scripts\python.exe pdf_web_compare_app.py --pdf C:\Docs\source.pdf --web https://example.com/page --output C:\Temp\packet
```

Each packet contains:

- `pdf/pdf-page-001.png`
- `web/web-page-001.png`
- `web/web-full-page.png`
- `pairs/pair-page-001.png`
- `llm-manifest.json`
- `llm-prompt.md`
- `visual-report.html`

The manifest is the machine-readable handoff point for an LLM tool. The prompt
lists absolute paths to each image pair.

## Text CLI

The earlier text-diff report is still available:

```powershell
.\.venv\Scripts\python.exe pdf_web_compare_app.py --mode text --pdf C:\Docs\source.pdf --web https://example.com/page
```

## Notes

- PDF rendering uses PyMuPDF.
- Webpage rendering uses Playwright Chromium.
- The webpage is captured from a browser after network idle, plus a short wait.
- The webpage image is sliced vertically to match the rendered PDF page heights.
- Highly dynamic pages may need a longer `--wait-ms` value.
