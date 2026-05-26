"""For each PDF in input/ produce a side-by-side PNG: PDF page | extracted article text.

Output goes to compare/<stem>/sxs-page-N.png. The article text is rendered as plain
multi-line text so I can read it next to the rasterised PDF page.
"""
from __future__ import annotations
import html
import pathlib
import re
import sys
import textwrap

import fitz  # PyMuPDF
from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent
INPUT = ROOT / "input"
OUTPUT = ROOT / "output"
COMPARE = ROOT / "compare"
COMPARE.mkdir(exist_ok=True)

PAGES = 4
DPI = 130
WRAP_COLS = 70
LINE_HEIGHT = 22


def stem(name: str) -> str:
    return re.sub(r"\.(?:pdf|PDF)$", "", name)


def load_article(html_path: pathlib.Path) -> str:
    text = html_path.read_text(encoding="utf-8", errors="replace")
    match = re.search(
        r'<details class="pdf-extracted-content">[\s\S]*?<article>([\s\S]*?)</article>',
        text,
    )
    if not match:
        return "(no article block)"
    body = match.group(1)
    return strip_tags(body)


def strip_tags(html_body: str) -> str:
    # Insert newlines around common block elements before stripping tags.
    body = re.sub(r"</(?:p|h\d|li|tr|hr)>", "\n", html_body)
    body = re.sub(r"<br\s*/?>", "\n", body)
    body = re.sub(r"<(?:h\d)[^>]*>", "\n# ", body)
    body = re.sub(r"<tr[^>]*>", "| ", body)
    body = re.sub(r"</t[dh]>", " | ", body)
    body = re.sub(r"<li[^>]*>", "  - ", body)
    body = re.sub(r"<table[^>]*>", "\n[table]\n", body)
    body = re.sub(r"</table>", "\n[/table]\n", body)
    body = re.sub(r"<[^>]+>", "", body)
    body = html.unescape(body)
    body = re.sub(r"\n{3,}", "\n\n", body)
    return body.strip()


def render_text(text: str, width_px: int, height_px: int) -> Image.Image:
    img = Image.new("RGB", (width_px, height_px), (255, 255, 255))
    draw = ImageDraw.Draw(img)
    try:
        font = ImageFont.truetype("consola.ttf", 16)
    except OSError:
        font = ImageFont.load_default()
    wrapper = textwrap.TextWrapper(width=WRAP_COLS, break_long_words=False, replace_whitespace=False)
    y = 8
    for raw_line in text.splitlines():
        if not raw_line.strip():
            y += LINE_HEIGHT // 2
            continue
        for line in wrapper.wrap(raw_line) or [""]:
            if y + LINE_HEIGHT > height_px - 8:
                draw.text((8, y), "… [truncated]", fill=(160, 0, 0), font=font)
                return img
            draw.text((8, y), line, fill=(20, 20, 20), font=font)
            y += LINE_HEIGHT
    return img


def render_pdf_page(pdf_doc: fitz.Document, page_index: int) -> Image.Image:
    page = pdf_doc.load_page(page_index)
    pix = page.get_pixmap(dpi=DPI)
    return Image.frombytes("RGB", (pix.width, pix.height), pix.samples)


def compose(pdf_img: Image.Image, text: str, label: str) -> Image.Image:
    height = pdf_img.height
    text_img = render_text(text, 950, height)
    width = pdf_img.width + text_img.width + 4
    canvas = Image.new("RGB", (width, height + 28), (40, 40, 40))
    canvas.paste(pdf_img, (0, 28))
    canvas.paste(text_img, (pdf_img.width + 4, 28))
    draw = ImageDraw.Draw(canvas)
    try:
        font = ImageFont.truetype("consola.ttf", 14)
    except OSError:
        font = ImageFont.load_default()
    draw.text((8, 6), f"PDF — {label}", fill=(220, 220, 220), font=font)
    draw.text((pdf_img.width + 12, 6), f"HTML article — {label}", fill=(220, 220, 220), font=font)
    return canvas


def main() -> int:
    for pdf_path in sorted(INPUT.iterdir()):
        if pdf_path.suffix.lower() != ".pdf":
            continue
        s = stem(pdf_path.name)
        target = COMPARE / s
        target.mkdir(parents=True, exist_ok=True)
        html_path = OUTPUT / f"{s}.html"
        article = load_article(html_path) if html_path.exists() else "(no HTML output)"
        pdf_doc = fitz.open(pdf_path)
        pages = min(len(pdf_doc), PAGES)
        for i in range(pages):
            pdf_img = render_pdf_page(pdf_doc, i)
            sxs = compose(pdf_img, article, f"page {i + 1}/{len(pdf_doc)}")
            sxs.save(str(target / f"sxs-page-{i + 1}.png"), optimize=True)
        pdf_doc.close()
        print(f"{pdf_path.name}: composed {pages}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
