import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FileOutput, FolderSync, Images, PanelLeft, PanelRight } from "lucide-react";
import { fetchLibrary, renderJobMessage, startRenderOutput, waitForRenderJob } from "./api";
import { FolderHints, Navigator, PairStrip, StatusBar } from "./components/Layout";
import { Pane } from "./components/Pane";
import { RenderProgress } from "./components/RenderProgress";
import { pairFiles, pruneCounts, resolvePairPage, totalPages, totalPairPages } from "./pairing";
import type { LibraryResponse, PageCounts, RefreshOptions, RenderJob, Side } from "./types";

export function App() {
  const [library, setLibrary] = useState<LibraryResponse | null>(null);
  const [pageCounts, setPageCounts] = useState<PageCounts>({ input: {}, output: {} });
  const [globalPage, setGlobalPage] = useState(1);
  const [zoom, setZoom] = useState(0.7);
  const [includeImages, setIncludeImages] = useState(false);
  const [isRendering, setIsRendering] = useState(false);
  const [renderMessage, setRenderMessage] = useState<string | null>(null);
  const [renderJob, setRenderJob] = useState<RenderJob | null>(null);
  const [error, setError] = useState<string | null>(null);
  const backgroundSyncStarted = useRef(false);

  const refreshLibrary = useCallback(async (options: RefreshOptions = {}) => {
    setError(null);
    try {
      const data = await fetchLibrary();
      setLibrary(data);
      setPageCounts((current) => pruneCounts(current, data));
      if (options.resetPage ?? true) {
        setGlobalPage(1);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, []);

  const generateOutput = useCallback(async () => {
    setError(null);
    setRenderMessage("Starting background generation...");
    setIsRendering(true);
    try {
      const started = await startRenderOutput(includeImages);
      setRenderJob(started);
      setRenderMessage(renderJobMessage(started));
      if (started.status === "running") {
        const finished = await trackRenderJob(started, setRenderJob, setRenderMessage, refreshLibrary);
        if (finished.status === "error") {
          throw new Error(finished.error ?? "Render failed");
        }
      }
      await refreshLibrary({ resetPage: false });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsRendering(false);
    }
  }, [includeImages, refreshLibrary]);

  useEffect(() => {
    void refreshLibrary();
  }, [refreshLibrary]);

  useEffect(() => {
    if (backgroundSyncStarted.current || !library) {
      return;
    }
    backgroundSyncStarted.current = true;
    void generateOutput();
  }, [generateOutput, library]);

  const filePairs = useMemo(
    () => pairFiles(library?.input ?? [], library?.output ?? []),
    [library],
  );

  const totals = useMemo(() => {
    if (!library) {
      return { input: 0, output: 0, combined: 0 };
    }
    return {
      input: totalPages(library.input, pageCounts.input),
      output: totalPages(library.output, pageCounts.output),
      combined: totalPairPages(filePairs, pageCounts),
    };
  }, [filePairs, library, pageCounts]);

  const pairPage = useMemo(
    () => resolvePairPage(filePairs, pageCounts, globalPage),
    [filePairs, globalPage, pageCounts],
  );

  const updateCount = useCallback((side: Side, fileId: string, count: number) => {
    const nextCount = Math.max(1, Math.ceil(count));
    setPageCounts((current) =>
      current[side][fileId] === nextCount
        ? current
        : { ...current, [side]: { ...current[side], [fileId]: nextCount } },
    );
  }, []);

  useEffect(() => {
    if (totals.combined > 0 && globalPage > totals.combined) {
      setGlobalPage(totals.combined);
    }
  }, [globalPage, totals.combined]);

  const canMove = totals.combined > 0;
  const currentPage = canMove ? Math.min(globalPage, totals.combined) : 0;

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <h1>Interactive Compare</h1>
          <p>
            Matched files from <code>input</code> and <code>output</code>, paged together by
            filename.
          </p>
        </div>
        <div className="actions">
          <button
            type="button"
            disabled={isRendering}
            onClick={() => void generateOutput()}
            title="Regenerate HTML output for every input PDF in the background"
          >
            <FileOutput size={18} />
            {isRendering ? "Generating..." : "Generate output"}
          </button>
          <label className="inline-toggle" title="Include rendered PDF page images in generated HTML">
            <input
              type="checkbox"
              checked={includeImages}
              disabled={isRendering}
              onChange={(event) => setIncludeImages(event.currentTarget.checked)}
            />
            <Images size={17} />
            Include images
          </label>
          <button type="button" onClick={() => void refreshLibrary()} title="Refresh folder scan">
            <FolderSync size={18} />
            Refresh
          </button>
          <label className="zoom-control">
            Zoom
            <input
              type="range"
              min="0.35"
              max="1.2"
              step="0.05"
              value={zoom}
              onChange={(event) => setZoom(Number(event.currentTarget.value))}
            />
            <span>{Math.round(zoom * 100)}%</span>
          </label>
        </div>
      </header>

      <StatusBar currentPage={currentPage} totals={totals} />
      <Navigator
        canMove={canMove}
        currentPage={currentPage}
        totalPages={totals.combined}
        onPageChange={setGlobalPage}
      />
      <PairStrip pairPage={pairPage} pairIndex={filePairs.findIndex((pair) => pair.id === pairPage.pair?.id)} totalPairs={filePairs.length} />

      {error ? <div className="error">{error}</div> : null}
      {renderMessage && !error ? <RenderProgress message={renderMessage} job={renderJob} /> : null}

      <FolderHints library={library} />

      <section className="compare-grid">
        <Pane
          title="Input"
          icon={<PanelLeft size={18} />}
          side="input"
          resolved={pairPage.input}
          total={totals.input}
          zoom={zoom}
          onPageCount={updateCount}
        />
        <Pane
          title="Output"
          icon={<PanelRight size={18} />}
          side="output"
          resolved={pairPage.output}
          total={totals.output}
          zoom={zoom}
          onPageCount={updateCount}
        />
      </section>
    </main>
  );
}

async function trackRenderJob(
  started: RenderJob,
  setRenderJob: (job: RenderJob) => void,
  setRenderMessage: (message: string) => void,
  refreshLibrary: (options?: RefreshOptions) => Promise<void>,
) {
  let lastCompleted = started.completed;
  return waitForRenderJob(async (job) => {
    setRenderJob(job);
    setRenderMessage(renderJobMessage(job));
    if (job.completed !== lastCompleted) {
      lastCompleted = job.completed;
      await refreshLibrary({ resetPage: false });
    }
  });
}
