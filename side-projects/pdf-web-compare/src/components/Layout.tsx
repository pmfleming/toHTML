import { Check, ChevronLeft, ChevronRight, Eye, EyeOff, FolderOpen, RotateCcw } from "lucide-react";
import type { FileCoverage, FilePair, LibraryResponse, PaneKey, ResolvedPairPage } from "../types";
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

export function ViewToggles({
  visiblePanes,
  onToggle,
}: {
  visiblePanes: Record<PaneKey, boolean>;
  onToggle: (pane: PaneKey) => void;
}) {
  return (
    <section className="view-toggles" aria-label="Visible comparison panes">
      {paneToggleItems.map((item) => {
        const active = visiblePanes[item.key];
        return (
          <button
            key={item.key}
            className={`toggle-button ${active ? "active" : ""}`}
            type="button"
            aria-pressed={active}
            title={`${active ? "Hide" : "Show"} ${item.label} pane`}
            onClick={() => onToggle(item.key)}
          >
            {active ? <Eye size={16} /> : <EyeOff size={16} />}
            {item.label}
          </button>
        );
      })}
    </section>
  );
}

export function PairStrip({
  pairPage,
  pairs,
  pairIndex,
  totalPairs,
  onPairChange,
}: {
  pairPage: ResolvedPairPage;
  pairs: FilePair[];
  pairIndex: number;
  totalPairs: number;
  onPairChange: (pairId: string) => void;
}) {
  if (!pairPage.pair) {
    return null;
  }
  return (
    <section className="pair-strip" aria-label="Current matched file">
      <select
        aria-label="Jump to matched file"
        value={pairPage.pair.id}
        onChange={(event) => onPairChange(event.currentTarget.value)}
      >
        {pairs.map((pair, index) => (
          <option key={pair.id} value={pair.id}>
            {pairOptionLabel(pair, index)}
          </option>
        ))}
      </select>
      <span>
        matched file {pairIndex + 1} of {totalPairs}, page {pairPage.localPage} of{" "}
        {pairPage.pairPageCount}
      </span>
    </section>
  );
}

const paneToggleItems: Array<{ key: PaneKey; label: string }> = [
  { key: "input", label: "Input" },
  { key: "output", label: "Output" },
  { key: "editor", label: "Editor" },
];

function pairOptionLabel(pair: FilePair, index: number) {
  const fileName = pair.input?.relativePath ?? pair.output?.relativePath ?? pair.name;
  const matchState = pair.input && pair.output
    ? "matched"
    : pair.input
      ? "missing output"
      : "output only";
  return `${index + 1}. ${fileName} (${matchState})`;
}

export function FolderHints({
  library,
  inputFolderDraft,
  inputFolderDisabled,
  onInputFolderDraftChange,
  onApplyInputFolder,
  onPickInputFolder,
}: {
  library: LibraryResponse | null;
  inputFolderDraft: string;
  inputFolderDisabled: boolean;
  onInputFolderDraftChange: (value: string) => void;
  onApplyInputFolder: (value: string) => void;
  onPickInputFolder: () => void;
}) {
  return (
    <section className="folder-hints">
      <div>
        <strong>Input</strong>
        <form
          className="folder-picker"
          onSubmit={(event) => {
            event.preventDefault();
            const form = event.currentTarget;
            const input = form.elements.namedItem("inputFolder");
            onApplyInputFolder(input instanceof HTMLInputElement ? input.value : inputFolderDraft);
          }}
        >
          <input
            name="inputFolder"
            aria-label="Input folder"
            value={inputFolderDraft}
            disabled={inputFolderDisabled}
            placeholder={library ? "Choose an input folder" : "Loading..."}
            onChange={(event) => onInputFolderDraftChange(event.currentTarget.value)}
          />
          <button type="submit" disabled={inputFolderDisabled} title="Use this input folder">
            <Check size={16} />
            Apply
          </button>
          <button type="button" disabled={inputFolderDisabled} title="Choose input folder" onClick={onPickInputFolder}>
            <FolderOpen size={16} />
            Choose
          </button>
        </form>
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
