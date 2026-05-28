import { useCallback, useEffect, useRef, useState } from "react";
import * as pdfjs from "pdfjs-dist";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import { fallbackHtmlLayout, fallbackHtmlPageSlice, measureHtmlLayout } from "../htmlLayout";
import type { HtmlPageLayout, LibraryFile, Side } from "../types";
import { clamp } from "../utils";

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

export function DocumentPage({
  file,
  localPage,
  zoom,
  onPageCount,
}: {
  file: LibraryFile;
  localPage: number;
  zoom: number;
  onPageCount: (count: number) => void;
}) {
  if (file.kind === "pdf") {
    return <PdfPage file={file} pageNumber={localPage} zoom={zoom} onPageCount={onPageCount} />;
  }
  if (file.kind === "html") {
    return <HtmlPage file={file} pageNumber={localPage} zoom={zoom} onPageCount={onPageCount} />;
  }
  return <ImagePage file={file} zoom={zoom} onPageCount={onPageCount} />;
}

export function EmptySide({ side }: { side: Side }) {
  return (
    <div className="empty-side">
      <strong>No {side} page for this position</strong>
      <span>The matched file may be missing on this side or have fewer pages.</span>
    </div>
  );
}

function PdfPage({
  file,
  pageNumber,
  zoom,
  onPageCount,
}: {
  file: LibraryFile;
  pageNumber: number;
  zoom: number;
  onPageCount: (count: number) => void;
}) {
  const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
  const [message, setMessage] = useState("Loading PDF...");
  const hasRendered = useRef(false);

  useEffect(() => {
    if (!canvas) {
      return;
    }
    let cancelled = false;
    if (!hasRendered.current) {
      setMessage("Loading PDF...");
    }
    const task = pdfjs.getDocument(file.url);
    task.promise
      .then(async (pdf) => {
        onPageCount(pdf.numPages);
        const page = await pdf.getPage(clamp(pageNumber, 1, pdf.numPages));
        const viewport = page.getViewport({ scale: zoom * 1.45 });
        const nextCanvas = document.createElement("canvas");
        const nextContext = nextCanvas.getContext("2d");
        if (!nextContext || cancelled) {
          return;
        }
        nextCanvas.width = Math.floor(viewport.width);
        nextCanvas.height = Math.floor(viewport.height);
        await page.render({ canvas: nextCanvas, canvasContext: nextContext, viewport }).promise;
        if (!cancelled) {
          const context = canvas.getContext("2d");
          if (!context) {
            return;
          }
          canvas.width = nextCanvas.width;
          canvas.height = nextCanvas.height;
          context.drawImage(nextCanvas, 0, 0);
          hasRendered.current = true;
          setMessage("");
        }
      })
      .catch((reason) => {
        if (!cancelled) {
          setMessage(reason instanceof Error ? reason.message : String(reason));
        }
      });
    return () => {
      cancelled = true;
      task.destroy();
    };
  }, [canvas, file.url, onPageCount, pageNumber, zoom]);

  return (
    <div className="pdf-view">
      {message ? <div className="viewer-message">{message}</div> : null}
      <canvas ref={setCanvas} />
    </div>
  );
}

function HtmlPage({
  file,
  pageNumber,
  zoom,
  onPageCount,
}: {
  file: LibraryFile;
  pageNumber: number;
  zoom: number;
  onPageCount: (count: number) => void;
}) {
  const [layout, setLayout] = useState<HtmlPageLayout>(() => fallbackHtmlLayout(1));
  const currentPageIndex = clamp(pageNumber, 1, layout.pages.length) - 1;
  const page = layout.pages[currentPageIndex] ?? fallbackHtmlPageSlice(0, 1);
  const scaledWidth = Math.round(page.width * zoom);
  const scaledHeight = Math.round(page.height * zoom);
  const frameWidth = Math.max(page.documentWidth, page.offsetX + page.width);
  const frameHeight = Math.max(page.documentHeight, page.offsetY + page.height);

  useEffect(() => {
    setLayout(fallbackHtmlLayout(1));
  }, [file.url]);

  const handleLoad = useCallback(
    (event: React.SyntheticEvent<HTMLIFrameElement>) => {
      const iframe = event.currentTarget;
      try {
        const doc = iframe.contentDocument;
        if (!doc) {
          throw new Error("Unable to inspect HTML document");
        }
        const nextLayout = measureHtmlLayout(doc);
        doc.documentElement.style.overflow = "hidden";
        doc.body.style.overflow = "hidden";
        setLayout(nextLayout);
        onPageCount(nextLayout.pages.length);
      } catch {
        setLayout(fallbackHtmlLayout(1));
        onPageCount(1);
      }
    },
    [onPageCount],
  );

  return (
    <div className="html-frame-window" style={{ width: scaledWidth, height: scaledHeight }}>
      <iframe
        title={`${file.name} page ${pageNumber}`}
        src={file.url}
        onLoad={handleLoad}
        scrolling="no"
        style={{
          width: frameWidth,
          height: frameHeight,
          transform: `scale(${zoom}) translate(${-page.offsetX}px, ${-page.offsetY}px)`,
          transformOrigin: "top left",
        }}
      />
    </div>
  );
}

function ImagePage({
  file,
  zoom,
  onPageCount,
}: {
  file: LibraryFile;
  zoom: number;
  onPageCount: (count: number) => void;
}) {
  useEffect(() => {
    onPageCount(1);
  }, [onPageCount]);
  return (
    <div className="image-view">
      <img src={file.url} alt={file.name} style={{ transform: `scale(${zoom})` }} />
    </div>
  );
}
