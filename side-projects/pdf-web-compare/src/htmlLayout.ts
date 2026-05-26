import { PAGE_HEIGHT, PAGE_WIDTH } from "./constants";
import type { HtmlPageLayout, HtmlPageSlice } from "./types";

export function measureHtmlLayout(doc: Document): HtmlPageLayout {
  const documentWidth = measureDocumentWidth(doc);
  const documentHeight = measureDocumentHeight(doc);
  const recreatedPages = Array.from(
    doc.querySelectorAll<HTMLElement>(".pdf-recreated-page[data-page]"),
  );

  if (recreatedPages.length > 0) {
    return {
      pages: recreatedPages.map((element) => measureRecreatedPage(element, documentWidth, documentHeight)),
    };
  }

  const pageCount = Math.max(1, Math.ceil(documentHeight / PAGE_HEIGHT));
  return {
    pages: Array.from({ length: pageCount }, (_, index) =>
      fallbackHtmlPageSlice(index, pageCount, documentWidth, documentHeight),
    ),
  };
}

export function fallbackHtmlLayout(pageCount: number): HtmlPageLayout {
  return {
    pages: Array.from({ length: Math.max(1, pageCount) }, (_, index) =>
      fallbackHtmlPageSlice(index, pageCount),
    ),
  };
}

export function fallbackHtmlPageSlice(
  index: number,
  pageCount: number,
  documentWidth = PAGE_WIDTH,
  documentHeight = Math.max(PAGE_HEIGHT, pageCount * PAGE_HEIGHT),
): HtmlPageSlice {
  return {
    width: PAGE_WIDTH,
    height: PAGE_HEIGHT,
    offsetX: 0,
    offsetY: index * PAGE_HEIGHT,
    documentWidth,
    documentHeight,
  };
}

function measureRecreatedPage(
  element: HTMLElement,
  documentWidth: number,
  documentHeight: number,
): HtmlPageSlice {
  const rect = element.getBoundingClientRect();
  return {
    width: Math.max(1, Math.ceil(element.offsetWidth || rect.width || PAGE_WIDTH)),
    height: Math.max(1, Math.ceil(element.offsetHeight || rect.height || PAGE_HEIGHT)),
    offsetX: Math.max(0, Math.floor(element.offsetLeft)),
    offsetY: Math.max(0, Math.floor(element.offsetTop)),
    documentWidth,
    documentHeight,
  };
}

function measureDocumentWidth(doc: Document) {
  const body = doc.body;
  const root = doc.documentElement;
  return Math.max(
    PAGE_WIDTH,
    body?.scrollWidth ?? 0,
    root?.scrollWidth ?? 0,
    body?.offsetWidth ?? 0,
    root?.offsetWidth ?? 0,
  );
}

function measureDocumentHeight(doc: Document) {
  const body = doc.body;
  const root = doc.documentElement;
  return Math.max(
    PAGE_HEIGHT,
    body?.scrollHeight ?? 0,
    root?.scrollHeight ?? 0,
    body?.offsetHeight ?? 0,
    root?.offsetHeight ?? 0,
  );
}
