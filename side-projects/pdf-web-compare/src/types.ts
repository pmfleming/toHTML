import type React from "react";

export type FileKind = "pdf" | "html" | "image";
export type Side = "input" | "output";
export type RenderJobState = "idle" | "running" | "done" | "error";

export interface LibraryFile {
  id: string;
  side: Side;
  name: string;
  relativePath: string;
  extension: string;
  kind: FileKind;
  url: string;
}

export interface RefreshOptions {
  resetPage?: boolean;
}

export interface LibraryResponse {
  inputDir: string;
  outputDir: string;
  input: LibraryFile[];
  output: LibraryFile[];
}

export interface RenderJob {
  status: RenderJobState;
  includeImages: boolean;
  scanned: number;
  total: number;
  completed: number;
  failed: number;
  current: string | null;
  error: string | null;
}

export interface PageCounts {
  input: Record<string, number>;
  output: Record<string, number>;
}

export interface FilePair {
  id: string;
  key: string;
  name: string;
  input: LibraryFile | null;
  output: LibraryFile | null;
}

export interface FileCoverage {
  input: number;
  output: number;
  matched: number;
  missingOutput: number;
  extraOutput: number;
}

export interface ResolvedPage {
  file: LibraryFile | null;
  localPage: number;
  totalPagesInFile: number;
  globalPage: number;
}

export interface ResolvedPairPage {
  pair: FilePair | null;
  localPage: number;
  pairPageCount: number;
  input: ResolvedPage;
  output: ResolvedPage;
}

export interface HtmlPageSlice {
  width: number;
  height: number;
  offsetX: number;
  offsetY: number;
  documentWidth: number;
  documentHeight: number;
}

export interface HtmlPageLayout {
  pages: HtmlPageSlice[];
}

export interface PaneProps {
  title: string;
  icon: React.ReactNode;
  side: Side;
  resolved: ResolvedPage;
  total: number;
  zoom: number;
  onPageCount: (side: Side, fileId: string, count: number) => void;
}
