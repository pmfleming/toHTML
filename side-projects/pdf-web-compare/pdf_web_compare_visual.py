from __future__ import annotations

import html
import datetime as dt
import json
import os
from pathlib import Path
import sys
import urllib.parse

from pdf_web_compare_common import AppError, PLAYWRIGHT_BROWSERS_DIR, REPORTS_DIR, safe_run_name

def browser_target(source: str) -> str:
    parsed = urllib.parse.urlparse(source)
    if parsed.scheme in {"http", "https"}:
        return source

    path = Path(source).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    if not path.exists():
        raise AppError(f"Webpage file does not exist: {path}")
    return path.as_uri()


def render_pdf_images(pdf_path: Path, output_dir: Path, dpi: int) -> list[dict[str, object]]:
    try:
        import fitz  # type: ignore[import-not-found]
    except ImportError as exc:
        raise AppError(
            "PyMuPDF is required for PDF rendering. Install it with: pip install pymupdf"
        ) from exc

    output_dir.mkdir(parents=True, exist_ok=True)
    pages: list[dict[str, object]] = []
    try:
        with fitz.open(str(pdf_path)) as document:
            for index, page in enumerate(document, start=1):
                pixmap = page.get_pixmap(dpi=dpi, alpha=False)
                path = output_dir / f"pdf-page-{index:03}.png"
                pixmap.save(str(path))
                pages.append(
                    {
                        "page": index,
                        "path": path,
                        "width": pixmap.width,
                        "height": pixmap.height,
                    }
                )
    except Exception as exc:  # noqa: BLE001 - fitz raises several exception types.
        raise AppError(f"Could not render PDF {pdf_path}: {exc}") from exc

    if not pages:
        raise AppError("The PDF has no pages to render.")
    return pages


def render_web_fullpage(
    web_source: str,
    output_path: Path,
    *,
    viewport_width: int,
    viewport_height: int,
    device_scale_factor: float,
    wait_ms: int,
) -> dict[str, object]:
    os.environ.setdefault("PLAYWRIGHT_BROWSERS_PATH", str(PLAYWRIGHT_BROWSERS_DIR))
    try:
        from playwright.sync_api import sync_playwright  # type: ignore[import-not-found]
    except ImportError as exc:
        raise AppError(
            "Playwright is required for webpage screenshots. Install it with: pip install playwright"
        ) from exc

    target = browser_target(web_source)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            page = browser.new_page(
                viewport={"width": viewport_width, "height": viewport_height},
                device_scale_factor=device_scale_factor,
            )
            page.goto(target, wait_until="networkidle", timeout=45_000)
            if wait_ms:
                page.wait_for_timeout(wait_ms)
            page.screenshot(path=str(output_path), full_page=True)
            dimensions = page.evaluate(
                "() => ({ width: document.documentElement.scrollWidth, "
                "height: document.documentElement.scrollHeight, "
                "title: document.title })"
            )
            browser.close()
    except Exception as exc:  # noqa: BLE001 - Playwright errors are runtime-specific.
        raise AppError(
            f"Could not render webpage screenshot. If Chromium is missing, run: "
            f"{Path(sys.executable).name} -m playwright install chromium\n\nDetails: {exc}"
        ) from exc

    return {"path": output_path, "target": target, **dimensions}


def slice_webpage_to_pdf_pages(
    fullpage_path: Path,
    pdf_pages: list[dict[str, object]],
    output_dir: Path,
) -> list[dict[str, object]]:
    try:
        from PIL import Image
    except ImportError as exc:
        raise AppError("Pillow is required for webpage image slicing. Install it with: pip install pillow") from exc

    output_dir.mkdir(parents=True, exist_ok=True)
    source = Image.open(fullpage_path).convert("RGB")
    slices: list[dict[str, object]] = []
    y_offset = 0
    for page_info in pdf_pages:
        page_number = int(page_info["page"])
        width = int(page_info["width"])
        height = int(page_info["height"])
        path = output_dir / f"web-page-{page_number:03}.png"
        canvas = Image.new("RGB", (width, height), "white")
        if y_offset < source.height:
            crop = source.crop(
                (
                    0,
                    y_offset,
                    min(width, source.width),
                    min(y_offset + height, source.height),
                )
            )
            canvas.paste(crop, (0, 0))
        canvas.save(path)
        slices.append(
            {
                "page": page_number,
                "path": path,
                "width": width,
                "height": height,
                "source_y": y_offset,
            }
        )
        y_offset += height
    source.close()
    return slices


def compose_pair_images(
    pdf_pages: list[dict[str, object]],
    web_pages: list[dict[str, object]],
    output_dir: Path,
) -> list[dict[str, object]]:
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError as exc:
        raise AppError("Pillow is required for pair image composition. Install it with: pip install pillow") from exc

    output_dir.mkdir(parents=True, exist_ok=True)
    pairs: list[dict[str, object]] = []
    try:
        font = ImageFont.truetype("arial.ttf", 18)
    except OSError:
        font = ImageFont.load_default()

    for pdf_info, web_info in zip(pdf_pages, web_pages, strict=False):
        page_number = int(pdf_info["page"])
        pdf_image = Image.open(Path(pdf_info["path"])).convert("RGB")
        web_image = Image.open(Path(web_info["path"])).convert("RGB")
        gutter = 16
        label_height = 36
        width = pdf_image.width + web_image.width + gutter
        height = max(pdf_image.height, web_image.height) + label_height
        canvas = Image.new("RGB", (width, height), (248, 250, 252))
        draw = ImageDraw.Draw(canvas)
        draw.text((10, 9), f"PDF page {page_number}", fill=(25, 33, 42), font=font)
        draw.text(
            (pdf_image.width + gutter + 10, 9),
            f"Web render slice {page_number}",
            fill=(25, 33, 42),
            font=font,
        )
        canvas.paste(pdf_image, (0, label_height))
        canvas.paste(web_image, (pdf_image.width + gutter, label_height))
        path = output_dir / f"pair-page-{page_number:03}.png"
        canvas.save(path)
        pdf_image.close()
        web_image.close()
        canvas.close()
        pairs.append(
            {
                "page": page_number,
                "path": path,
                "width": width,
                "height": height,
            }
        )
    return pairs


def rel(path: Path, base: Path) -> str:
    return path.resolve().relative_to(base.resolve()).as_posix()


def write_llm_manifest(
    *,
    output_dir: Path,
    pdf_path: Path,
    web_source: str,
    web_target: str,
    dpi: int,
    pdf_pages: list[dict[str, object]],
    web_pages: list[dict[str, object]],
    pair_images: list[dict[str, object]],
    fullpage: dict[str, object],
) -> Path:
    pairs = []
    for pdf_info, web_info, pair_info in zip(pdf_pages, web_pages, pair_images, strict=False):
        pairs.append(
            {
                "page": int(pdf_info["page"]),
                "pdf_page_image": rel(Path(pdf_info["path"]), output_dir),
                "web_page_image": rel(Path(web_info["path"]), output_dir),
                "side_by_side_image": rel(Path(pair_info["path"]), output_dir),
                "width": int(pdf_info["width"]),
                "height": int(pdf_info["height"]),
                "web_source_y": int(web_info["source_y"]),
            }
        )

    manifest = {
        "schema": "pdf-web-visual-compare/v1",
        "created_at": dt.datetime.now().isoformat(timespec="seconds"),
        "source": {
            "pdf": str(pdf_path),
            "web": web_source,
            "browser_target": web_target,
        },
        "rendering": {
            "pdf_dpi": dpi,
            "webpage_full_image": rel(Path(fullpage["path"]), output_dir),
            "webpage_scroll_width": fullpage.get("width"),
            "webpage_scroll_height": fullpage.get("height"),
            "webpage_title": fullpage.get("title"),
        },
        "pairs": pairs,
    }
    path = output_dir / "llm-manifest.json"
    path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    return path


def write_llm_prompt(output_dir: Path, manifest_path: Path, pairs: list[dict[str, object]]) -> Path:
    lines = [
        "# Visual PDF/Web Comparison Task",
        "",
        "Compare each PDF page image with the corresponding webpage render image.",
        "Focus on visual differences: missing content, extra content, changed wording, table/layout drift, images, headers, footers, spacing, and page breaks.",
        "Return concise findings grouped by page, with severity and evidence from the images.",
        "",
        f"Manifest: {manifest_path.resolve()}",
        "",
        "Images:",
    ]
    for pair in pairs:
        page = int(pair["page"])
        lines.extend(
            [
                f"- Page {page}",
                f"  - PDF: {(output_dir / str(pair['pdf_page_image'])).resolve()}",
                f"  - Web: {(output_dir / str(pair['web_page_image'])).resolve()}",
                f"  - Side by side: {(output_dir / str(pair['side_by_side_image'])).resolve()}",
            ]
        )

    path = output_dir / "llm-prompt.md"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def write_visual_report(
    *,
    output_dir: Path,
    pdf_path: Path,
    web_source: str,
    manifest_path: Path,
    prompt_path: Path,
    pairs: list[dict[str, object]],
) -> Path:
    def esc(value: object) -> str:
        return html.escape(str(value), quote=True)

    pair_sections = []
    for pair in pairs:
        page = int(pair["page"])
        pdf_src = esc(pair["pdf_page_image"])
        web_src = esc(pair["web_page_image"])
        sxs_src = esc(pair["side_by_side_image"])
        pair_sections.append(
            f"""
      <section class="page-pair">
        <h2>Page {page}</h2>
        <div class="triple">
          <figure><img src="{pdf_src}" alt="PDF page {page}"><figcaption>PDF</figcaption></figure>
          <figure><img src="{web_src}" alt="Webpage render slice {page}"><figcaption>Web render</figcaption></figure>
          <figure><img src="{sxs_src}" alt="Side-by-side page {page}"><figcaption>Side by side for LLM</figcaption></figure>
        </div>
      </section>"""
        )

    report = output_dir / "visual-report.html"
    report.write_text(
        f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>PDF/Web Visual Packet</title>
  <style>
    * {{ box-sizing: border-box; }}
    body {{ margin: 0; color: #18212c; font: 15px/1.5 "Segoe UI", system-ui, sans-serif; background: #fff; }}
    header {{ padding: 28px clamp(18px, 4vw, 48px); border-bottom: 1px solid #d9e1ea; background: #f7fafc; }}
    main {{ padding: 24px clamp(18px, 4vw, 48px) 48px; }}
    h1 {{ margin: 0 0 8px; font-size: clamp(24px, 3vw, 34px); letter-spacing: 0; }}
    h2 {{ margin: 28px 0 12px; font-size: 20px; letter-spacing: 0; }}
    .meta {{ color: #5f6c7b; overflow-wrap: anywhere; }}
    .links {{ display: flex; flex-wrap: wrap; gap: 10px; margin-top: 16px; }}
    .links a {{ color: #0f6b7a; font-weight: 600; }}
    .page-pair {{ border-top: 1px solid #d9e1ea; padding-top: 8px; }}
    .triple {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 14px; align-items: start; }}
    figure {{ margin: 0; border: 1px solid #d9e1ea; border-radius: 8px; overflow: hidden; background: #f8fafc; }}
    img {{ display: block; width: 100%; height: auto; }}
    figcaption {{ padding: 8px 10px; color: #5f6c7b; border-top: 1px solid #d9e1ea; }}
  </style>
</head>
<body>
  <header>
    <h1>PDF/Web Visual Packet</h1>
    <div class="meta">PDF: {esc(pdf_path)}</div>
    <div class="meta">Webpage: {esc(web_source)}</div>
    <div class="links">
      <a href="{esc(rel(manifest_path, output_dir))}">LLM manifest</a>
      <a href="{esc(rel(prompt_path, output_dir))}">LLM prompt</a>
    </div>
  </header>
  <main>
    {''.join(pair_sections)}
  </main>
</body>
</html>
""",
        encoding="utf-8",
    )
    return report


def visual_compare(
    pdf_path: Path,
    web_source: str,
    output_dir: Path | None = None,
    *,
    dpi: int = 144,
    wait_ms: int = 500,
) -> dict[str, Path]:
    pdf_path = pdf_path.expanduser()
    if not pdf_path.is_absolute():
        pdf_path = (Path.cwd() / pdf_path).resolve()
    if not pdf_path.exists():
        raise AppError(f"PDF does not exist: {pdf_path}")
    if pdf_path.suffix.casefold() != ".pdf":
        raise AppError(f"Expected a .pdf file: {pdf_path}")

    if output_dir is None:
        output_dir = REPORTS_DIR / safe_run_name(pdf_path, web_source)
    else:
        output_dir = output_dir.expanduser()
        if not output_dir.is_absolute():
            output_dir = (Path.cwd() / output_dir).resolve()
        if output_dir.suffix.casefold() == ".html":
            output_dir = output_dir.with_suffix("")

    output_dir.mkdir(parents=True, exist_ok=True)
    pdf_pages = render_pdf_images(pdf_path, output_dir / "pdf", dpi)
    first_width = int(pdf_pages[0]["width"])
    first_height = int(pdf_pages[0]["height"])
    css_to_pdf_scale = dpi / 96
    fullpage = render_web_fullpage(
        web_source,
        output_dir / "web" / "web-full-page.png",
        viewport_width=round(first_width / css_to_pdf_scale),
        viewport_height=round(first_height / css_to_pdf_scale),
        device_scale_factor=css_to_pdf_scale,
        wait_ms=wait_ms,
    )
    web_pages = slice_webpage_to_pdf_pages(fullpage["path"], pdf_pages, output_dir / "web")
    pair_images = compose_pair_images(pdf_pages, web_pages, output_dir / "pairs")
    manifest_path = write_llm_manifest(
        output_dir=output_dir,
        pdf_path=pdf_path,
        web_source=web_source,
        web_target=str(fullpage["target"]),
        dpi=dpi,
        pdf_pages=pdf_pages,
        web_pages=web_pages,
        pair_images=pair_images,
        fullpage=fullpage,
    )
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    prompt_path = write_llm_prompt(output_dir, manifest_path, manifest["pairs"])
    report_path = write_visual_report(
        output_dir=output_dir,
        pdf_path=pdf_path,
        web_source=web_source,
        manifest_path=manifest_path,
        prompt_path=prompt_path,
        pairs=manifest["pairs"],
    )
    return {
        "directory": output_dir,
        "report": report_path,
        "manifest": manifest_path,
        "prompt": prompt_path,
    }
