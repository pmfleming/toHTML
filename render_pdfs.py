"""Rasterise every PDF in input/ to PNG pages in compare/<stem>/page-N.png."""
import os
import pathlib
import sys
import fitz  # PyMuPDF

ROOT = pathlib.Path(__file__).resolve().parent
INPUT = ROOT / "input"
OUT = ROOT / "compare"
OUT.mkdir(exist_ok=True)


def stem(path: pathlib.Path) -> str:
    name = path.name
    for ext in (".pdf", ".PDF"):
        if name.endswith(ext):
            return name[:-4]
    return path.stem


def rasterise(pdf_path: pathlib.Path, max_pages: int = 6) -> None:
    target = OUT / stem(pdf_path)
    target.mkdir(parents=True, exist_ok=True)
    doc = fitz.open(pdf_path)
    pages = min(len(doc), max_pages)
    for i in range(pages):
        page = doc.load_page(i)
        pix = page.get_pixmap(dpi=120)
        pix.save(str(target / f"page-{i + 1}.png"))
    doc.close()
    print(f"{pdf_path.name}: {pages} page(s)")


def main() -> int:
    pdfs = sorted([p for p in INPUT.iterdir() if p.suffix.lower() == ".pdf"])
    if not pdfs:
        print("no PDFs in input/")
        return 1
    for pdf in pdfs:
        try:
            rasterise(pdf)
        except Exception as exc:  # noqa: BLE001
            print(f"FAILED {pdf.name}: {exc}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
