"""Compare a PDF against a webpage and write an HTML difference report.

Run without arguments to open a small Tkinter app:

    python pdf_web_compare_app.py

Run from the command line for repeatable comparisons:

    python pdf_web_compare_app.py --pdf input/report.pdf --web https://example.com/page
    python pdf_web_compare_app.py --pdf input/report.pdf --web output/report.html --open
"""

from __future__ import annotations

import argparse
import datetime as dt
import difflib
import html
from html.parser import HTMLParser
from pathlib import Path
import re
import sys
import textwrap
import urllib.parse
import urllib.request
import webbrowser

from pdf_web_compare_common import AppError, REPORTS_DIR, USER_AGENT, safe_report_name
from pdf_web_compare_visual import visual_compare


class VisibleTextParser(HTMLParser):
    """Extract readable text from HTML while ignoring non-content elements."""

    BLOCK_TAGS = {
        "address",
        "article",
        "aside",
        "blockquote",
        "br",
        "dd",
        "div",
        "dl",
        "dt",
        "figcaption",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hr",
        "li",
        "main",
        "nav",
        "ol",
        "p",
        "pre",
        "section",
        "table",
        "td",
        "th",
        "tr",
        "ul",
    }
    SKIP_TAGS = {"script", "style", "noscript", "svg", "canvas"}

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []
        self.skip_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        del attrs
        tag = tag.lower()
        if tag in self.SKIP_TAGS:
            self.skip_depth += 1
            return
        if self.skip_depth == 0 and tag in self.BLOCK_TAGS:
            self.parts.append("\n")

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in self.SKIP_TAGS and self.skip_depth:
            self.skip_depth -= 1
            return
        if self.skip_depth == 0 and tag in self.BLOCK_TAGS:
            self.parts.append("\n")

    def handle_data(self, data: str) -> None:
        if self.skip_depth:
            return
        self.parts.append(data)

    def text(self) -> str:
        return clean_text(" ".join(self.parts))


def clean_text(value: str) -> str:
    value = html.unescape(value)
    value = value.replace("\xa0", " ")
    value = re.sub(r"[ \t\r\f\v]+", " ", value)
    value = re.sub(r"\s+([,.;:!?%)\]])", r"\1", value)
    value = re.sub(r"([(])\s+", r"\1", value)
    value = re.sub(r" *\n *", "\n", value)
    value = re.sub(r"\n{3,}", "\n\n", value)
    return value.strip()


def normalize_line(value: str) -> str:
    value = value.casefold()
    value = re.sub(r"\s+", " ", value)
    value = re.sub(r"[^\w\s]+", "", value)
    return value.strip()


def readable_lines(value: str) -> list[str]:
    lines: list[str] = []
    for raw_line in value.splitlines():
        line = re.sub(r"\s+", " ", raw_line).strip()
        if line:
            lines.append(line)
    return lines


def normalized_lines(value: str) -> list[str]:
    return [line for line in (normalize_line(line) for line in readable_lines(value)) if line]


def tokenize(value: str) -> list[str]:
    return re.findall(r"\b[\w'-]+\b", value.casefold())


def extract_pdf_text(pdf_path: Path) -> str:
    try:
        import fitz  # type: ignore[import-not-found]
    except ImportError as exc:
        raise AppError(
            "PyMuPDF is required for PDF text extraction. Install it with: pip install pymupdf"
        ) from exc

    if not pdf_path.exists():
        raise AppError(f"PDF does not exist: {pdf_path}")
    if pdf_path.suffix.casefold() != ".pdf":
        raise AppError(f"Expected a .pdf file: {pdf_path}")

    chunks: list[str] = []
    try:
        with fitz.open(str(pdf_path)) as document:
            for index, page in enumerate(document, start=1):
                text = clean_text(page.get_text("text"))
                if text:
                    chunks.append(f"[Page {index}]\n{text}")
                else:
                    chunks.append(f"[Page {index}]\n[No selectable text found]")
    except Exception as exc:  # noqa: BLE001 - fitz raises several exception types.
        raise AppError(f"Could not read PDF {pdf_path}: {exc}") from exc

    return "\n\n".join(chunks)


def fetch_webpage(source: str) -> tuple[str, str]:
    parsed = urllib.parse.urlparse(source)
    if parsed.scheme in {"http", "https"}:
        request = urllib.request.Request(source, headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                content_type = response.headers.get_content_charset() or "utf-8"
                body = response.read().decode(content_type, errors="replace")
        except Exception as exc:  # noqa: BLE001 - urllib wraps network errors broadly.
            raise AppError(f"Could not fetch webpage {source}: {exc}") from exc
        return body, source

    path = Path(source).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    if not path.exists():
        raise AppError(f"Webpage file does not exist: {path}")
    return path.read_text(encoding="utf-8", errors="replace"), str(path)


def extract_web_text(source: str) -> tuple[str, str]:
    body, resolved_source = fetch_webpage(source)
    parser = VisibleTextParser()
    parser.feed(body)
    return parser.text(), resolved_source


def ratio_percent(left: str, right: str) -> float:
    return difflib.SequenceMatcher(None, left, right).ratio() * 100


def line_stats(pdf_lines: list[str], web_lines: list[str]) -> dict[str, int]:
    matcher = difflib.SequenceMatcher(None, pdf_lines, web_lines, autojunk=False)
    equal = 0
    inserted = 0
    deleted = 0
    replaced = 0
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag == "equal":
            equal += i2 - i1
        elif tag == "insert":
            inserted += j2 - j1
        elif tag == "delete":
            deleted += i2 - i1
        elif tag == "replace":
            replaced += max(i2 - i1, j2 - j1)
    return {
        "matching_lines": equal,
        "pdf_only_lines": deleted,
        "web_only_lines": inserted,
        "changed_line_blocks": replaced,
    }


def word_stats(pdf_text: str, web_text: str) -> dict[str, int]:
    pdf_words = tokenize(pdf_text)
    web_words = tokenize(web_text)
    return {
        "pdf_words": len(pdf_words),
        "web_words": len(web_words),
        "shared_unique_words": len(set(pdf_words) & set(web_words)),
        "pdf_unique_words": len(set(pdf_words) - set(web_words)),
        "web_unique_words": len(set(web_words) - set(pdf_words)),
    }


def build_text_diff(pdf_text: str, web_text: str) -> str:
    pdf_lines = readable_lines(pdf_text)
    web_lines = readable_lines(web_text)
    diff = difflib.HtmlDiff(wrapcolumn=100).make_table(
        pdf_lines,
        web_lines,
        fromdesc="PDF",
        todesc="Webpage",
        context=True,
        numlines=3,
    )
    return diff


def classify_score(score: float) -> str:
    if score >= 95:
        return "Very close"
    if score >= 80:
        return "Mostly similar"
    if score >= 60:
        return "Partly similar"
    return "Substantially different"


def html_doc(
    *,
    pdf_path: Path,
    web_source: str,
    resolved_web_source: str,
    pdf_text: str,
    web_text: str,
) -> str:
    pdf_norm = "\n".join(normalized_lines(pdf_text))
    web_norm = "\n".join(normalized_lines(web_text))
    score = ratio_percent(pdf_norm, web_norm) if pdf_norm or web_norm else 0.0
    stats = {**line_stats(normalized_lines(pdf_text), normalized_lines(web_text)), **word_stats(pdf_text, web_text)}
    generated = dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    diff_table = build_text_diff(pdf_text, web_text)

    def esc(value: object) -> str:
        return html.escape(str(value), quote=True)

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>PDF/Web Comparison Report</title>
  <style>
    :root {{
      color-scheme: light;
      --ink: #19212a;
      --muted: #667085;
      --line: #d8dee8;
      --panel: #f7f9fc;
      --accent: #1f7a8c;
      --add: #e6f6ec;
      --del: #fde8e8;
      --change: #fff4cc;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      color: var(--ink);
      font: 15px/1.5 "Segoe UI", system-ui, -apple-system, sans-serif;
      background: #ffffff;
    }}
    header {{
      padding: 28px clamp(18px, 4vw, 48px);
      border-bottom: 1px solid var(--line);
      background: linear-gradient(180deg, #f8fbfd, #ffffff);
    }}
    h1 {{
      margin: 0 0 6px;
      font-size: clamp(24px, 3vw, 36px);
      line-height: 1.15;
      letter-spacing: 0;
    }}
    h2 {{
      margin: 30px 0 12px;
      font-size: 20px;
      letter-spacing: 0;
    }}
    main {{
      padding: 0 clamp(18px, 4vw, 48px) 48px;
    }}
    .meta {{
      color: var(--muted);
      overflow-wrap: anywhere;
    }}
    .summary {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
      gap: 12px;
      margin-top: 20px;
      max-width: 1040px;
    }}
    .metric {{
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
      background: var(--panel);
      min-height: 88px;
    }}
    .metric strong {{
      display: block;
      font-size: 24px;
      line-height: 1.15;
      margin-bottom: 4px;
    }}
    .metric span {{
      color: var(--muted);
      font-size: 13px;
    }}
    .sources {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 16px;
      margin-top: 22px;
      max-width: 1040px;
    }}
    .source {{
      border-left: 4px solid var(--accent);
      padding-left: 12px;
      overflow-wrap: anywhere;
    }}
    .samples {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 16px;
    }}
    pre {{
      max-height: 360px;
      overflow: auto;
      padding: 14px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: #fbfcfe;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }}
    .diff-wrapper {{
      overflow: auto;
      border: 1px solid var(--line);
      border-radius: 8px;
      max-width: 100%;
    }}
    table.diff {{
      width: 100%;
      border-collapse: collapse;
      font-family: Consolas, "Courier New", monospace;
      font-size: 13px;
    }}
    .diff_header {{
      background: #edf2f7;
      padding: 6px 8px;
      text-align: left;
    }}
    td {{
      vertical-align: top;
      padding: 3px 6px;
      border-top: 1px solid #eef1f5;
    }}
    .diff_next {{ color: var(--muted); }}
    .diff_add {{ background: var(--add); }}
    .diff_sub {{ background: var(--del); }}
    .diff_chg {{ background: var(--change); }}
  </style>
</head>
<body>
  <header>
    <h1>PDF/Web Comparison Report</h1>
    <div class="meta">Generated {esc(generated)}</div>
    <section class="summary" aria-label="Summary">
      <div class="metric"><strong>{score:.1f}%</strong><span>normalized text similarity</span></div>
      <div class="metric"><strong>{esc(classify_score(score))}</strong><span>comparison result</span></div>
      <div class="metric"><strong>{stats["pdf_words"]}</strong><span>PDF words</span></div>
      <div class="metric"><strong>{stats["web_words"]}</strong><span>webpage words</span></div>
      <div class="metric"><strong>{stats["pdf_only_lines"]}</strong><span>PDF-only lines</span></div>
      <div class="metric"><strong>{stats["web_only_lines"]}</strong><span>webpage-only lines</span></div>
      <div class="metric"><strong>{stats["changed_line_blocks"]}</strong><span>changed line blocks</span></div>
      <div class="metric"><strong>{stats["shared_unique_words"]}</strong><span>shared unique words</span></div>
    </section>
    <section class="sources" aria-label="Sources">
      <div class="source"><strong>PDF</strong><br>{esc(pdf_path)}</div>
      <div class="source"><strong>Webpage</strong><br>{esc(web_source)}<br><span class="meta">{esc(resolved_web_source)}</span></div>
    </section>
  </header>
  <main>
    <h2>Text Samples</h2>
    <section class="samples">
      <div>
        <strong>PDF text</strong>
        <pre>{esc(textwrap.shorten(pdf_text, width=5000, placeholder=" ..."))}</pre>
      </div>
      <div>
        <strong>Webpage text</strong>
        <pre>{esc(textwrap.shorten(web_text, width=5000, placeholder=" ..."))}</pre>
      </div>
    </section>
    <h2>Line Difference</h2>
    <div class="diff-wrapper">{diff_table}</div>
  </main>
</body>
</html>
"""

def compare(pdf_path: Path, web_source: str, output: Path | None = None) -> Path:
    pdf_path = pdf_path.expanduser()
    if not pdf_path.is_absolute():
        pdf_path = (Path.cwd() / pdf_path).resolve()

    pdf_text = extract_pdf_text(pdf_path)
    web_text, resolved_web_source = extract_web_text(web_source)
    if not pdf_text.strip():
        raise AppError("No selectable text was extracted from the PDF.")
    if not web_text.strip():
        raise AppError("No visible text was extracted from the webpage.")

    if output is None:
        REPORTS_DIR.mkdir(exist_ok=True)
        output = REPORTS_DIR / safe_report_name(pdf_path, web_source)
    elif output.suffix.casefold() != ".html":
        output = output.with_suffix(".html")

    output = output.expanduser()
    if not output.is_absolute():
        output = (Path.cwd() / output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        html_doc(
            pdf_path=pdf_path,
            web_source=web_source,
            resolved_web_source=resolved_web_source,
            pdf_text=pdf_text,
            web_text=web_text,
        ),
        encoding="utf-8",
    )
    return output


def run_gui() -> int:
    try:
        import tkinter as tk
        from tkinter import filedialog, messagebox
    except ImportError as exc:
        raise AppError("Tkinter is not available. Use the command-line mode instead.") from exc

    root = tk.Tk()
    root.title("PDF/Web Compare")
    root.geometry("720x260")
    root.minsize(620, 240)

    pdf_var = tk.StringVar()
    web_var = tk.StringVar()
    output_var = tk.StringVar()
    status_var = tk.StringVar(
        value="Choose a PDF and enter a webpage URL or local HTML file. The app will create PNGs and an LLM packet."
    )

    def browse_pdf() -> None:
        selected = filedialog.askopenfilename(
            title="Choose PDF",
            filetypes=[("PDF files", "*.pdf"), ("All files", "*.*")],
        )
        if selected:
            pdf_var.set(selected)

    def browse_web() -> None:
        selected = filedialog.askopenfilename(
            title="Choose webpage HTML file",
            filetypes=[("HTML files", "*.html;*.htm"), ("All files", "*.*")],
        )
        if selected:
            web_var.set(selected)

    def browse_output() -> None:
        selected = filedialog.askdirectory(
            title="Choose packet directory",
            mustexist=False,
        )
        if selected:
            output_var.set(selected)

    def run_compare() -> None:
        pdf = pdf_var.get().strip()
        web = web_var.get().strip()
        output = output_var.get().strip()
        if not pdf or not web:
            messagebox.showerror("Missing input", "Choose a PDF and enter a webpage URL or file.")
            return
        status_var.set("Comparing...")
        root.update_idletasks()
        try:
            result = visual_compare(Path(pdf), web, Path(output) if output else None)
        except AppError as exc:
            status_var.set("Comparison failed.")
            messagebox.showerror("Comparison failed", str(exc))
            return
        status_var.set(f"LLM packet created: {result['directory']}")
        if messagebox.askyesno("LLM packet created", f"Open visual report now?\n\n{result['report']}"):
            webbrowser.open(result["report"].as_uri())

    root.columnconfigure(1, weight=1)
    pad = {"padx": 10, "pady": 8}

    tk.Label(root, text="PDF").grid(row=0, column=0, sticky="w", **pad)
    tk.Entry(root, textvariable=pdf_var).grid(row=0, column=1, sticky="ew", **pad)
    tk.Button(root, text="Browse", command=browse_pdf).grid(row=0, column=2, **pad)

    tk.Label(root, text="Webpage").grid(row=1, column=0, sticky="w", **pad)
    tk.Entry(root, textvariable=web_var).grid(row=1, column=1, sticky="ew", **pad)
    tk.Button(root, text="File", command=browse_web).grid(row=1, column=2, **pad)

    tk.Label(root, text="Packet folder").grid(row=2, column=0, sticky="w", **pad)
    tk.Entry(root, textvariable=output_var).grid(row=2, column=1, sticky="ew", **pad)
    tk.Button(root, text="Choose", command=browse_output).grid(row=2, column=2, **pad)

    tk.Button(root, text="Compare", command=run_compare, width=18).grid(
        row=3, column=1, sticky="e", **pad
    )
    tk.Label(root, textvariable=status_var, anchor="w", justify="left", wraplength=660).grid(
        row=4, column=0, columnspan=3, sticky="ew", padx=10, pady=(18, 8)
    )

    root.mainloop()
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create image packets for PDF/web visual comparison, with optional text diff mode."
    )
    parser.add_argument("--pdf", type=Path, help="Path to the PDF file.")
    parser.add_argument(
        "--web",
        help="Webpage URL, or a local .html/.htm file path for offline comparison.",
    )
    parser.add_argument(
        "--mode",
        choices=["visual", "text"],
        default="visual",
        help="visual creates PNGs and an LLM packet; text creates a text-diff HTML report.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Output directory for visual mode, or output HTML report path for text mode.",
    )
    parser.add_argument("--dpi", type=int, default=144, help="PDF render DPI for visual mode.")
    parser.add_argument(
        "--wait-ms",
        type=int,
        default=500,
        help="Extra webpage wait time before screenshotting in visual mode.",
    )
    parser.add_argument("--open", action="store_true", help="Open the generated report.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        return run_gui()

    args = parse_args(argv)
    if not args.pdf or not args.web:
        raise AppError("Command-line mode requires --pdf and --web.")
    if args.mode == "text":
        report = compare(args.pdf, args.web, args.output)
        print(f"Text report written: {report}")
        if args.open:
            webbrowser.open(report.as_uri())
        return 0

    result = visual_compare(args.pdf, args.web, args.output, dpi=args.dpi, wait_ms=args.wait_ms)
    print(f"Visual packet directory: {result['directory']}")
    print(f"Visual report: {result['report']}")
    print(f"LLM manifest: {result['manifest']}")
    print(f"LLM prompt: {result['prompt']}")
    if args.open:
        webbrowser.open(result["report"].as_uri())
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AppError as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
