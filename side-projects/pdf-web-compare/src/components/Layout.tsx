import { ChevronLeft, ChevronRight, RotateCcw } from "lucide-react";
import type { FileCoverage, LibraryResponse, ResolvedPairPage } from "../types";
import { clamp } from "../utils";

export function StatusBar({
  currentPage,
  totals,
  coverage,
}: {
  currentPage: number;
  totals: { input: number; output: number; combined: number };
  coverage: FileCoverage;
}) {
  return (
    <section className="statusbar" aria-label="Comparison status">
      <Metric value={currentPage} label="current page" />
      <Metric value={totals.combined} label="total comparable pages" />
      <Metric value={`${coverage.matched}/${coverage.input}`} label="input files matched" />
      <Metric value={totals.input} label="input pages" />
      <Metric value={totals.output} label="output pages" />
    </section>
  );
}

export function Navigator({
  canMove,
  currentPage,
  totalPages,
  onPageChange,
}: {
  canMove: boolean;
  currentPage: number;
  totalPages: number;
  onPageChange: (value: number | ((page: number) => number)) => void;
}) {
  return (
    <section className="navigator" aria-label="Page navigation">
      <button
        type="button"
        disabled={!canMove || currentPage <= 1}
        onClick={() => onPageChange((page) => Math.max(1, page - 1))}
        title="Previous page"
      >
        <ChevronLeft size={18} />
        Previous
      </button>
      <input
        type="number"
        min={canMove ? 1 : 0}
        max={totalPages || 0}
        value={currentPage}
        disabled={!canMove}
        onChange={(event) => {
          const next = Number(event.currentTarget.value);
          if (Number.isFinite(next)) {
            onPageChange(clamp(Math.round(next), 1, Math.max(1, totalPages)));
          }
        }}
      />
      <span>of {totalPages || 0}</span>
      <button
        type="button"
        disabled={!canMove || currentPage >= totalPages}
        onClick={() => onPageChange((page) => Math.min(totalPages, page + 1))}
        title="Next page"
      >
        Next
        <ChevronRight size={18} />
      </button>
      <button type="button" disabled={!canMove} onClick={() => onPageChange(1)} title="Return to first page">
        <RotateCcw size={18} />
        Reset
      </button>
    </section>
  );
}

export function PairStrip({
  pairPage,
  pairIndex,
  totalPairs,
}: {
  pairPage: ResolvedPairPage;
  pairIndex: number;
  totalPairs: number;
}) {
  if (!pairPage.pair) {
    return null;
  }
  return (
    <section className="pair-strip" aria-label="Current matched file">
      <strong>{pairPage.pair.name}</strong>
      <span>
        matched file {pairIndex + 1} of {totalPairs}, page {pairPage.localPage} of{" "}
        {pairPage.pairPageCount}
      </span>
    </section>
  );
}

export function FolderHints({ library }: { library: LibraryResponse | null }) {
  return (
    <section className="folder-hints">
      <div>
        <strong>Input</strong>
        <span>{library?.inputDir ?? "loading..."}</span>
      </div>
      <div>
        <strong>Output</strong>
        <span>{library?.outputDir ?? "loading..."}</span>
      </div>
    </section>
  );
}

function Metric({ value, label }: { value: number | string; label: string }) {
  return (
    <div>
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}
