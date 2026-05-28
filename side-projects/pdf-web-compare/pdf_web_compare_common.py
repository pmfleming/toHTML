from __future__ import annotations

from pathlib import Path
import datetime as dt
import re
import urllib.parse


APP_DIR = Path(__file__).resolve().parent
REPORTS_DIR = APP_DIR / "reports"
PLAYWRIGHT_BROWSERS_DIR = APP_DIR / ".playwright-browsers"
USER_AGENT = "pdf-web-compare/1.0"


class AppError(Exception):
    """A user-facing application error."""


def safe_report_name(pdf_path: Path, web_source: str) -> str:
    source_name = urllib.parse.urlparse(web_source).netloc or Path(web_source).stem or "webpage"
    combined = f"{pdf_path.stem}-vs-{source_name}"
    combined = re.sub(r"[^A-Za-z0-9._-]+", "-", combined).strip("-")
    return f"{combined or 'comparison'}.html"


def safe_run_name(pdf_path: Path, web_source: str) -> str:
    stem = safe_report_name(pdf_path, web_source).removesuffix(".html")
    timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    return f"{stem}-visual-{timestamp}"
