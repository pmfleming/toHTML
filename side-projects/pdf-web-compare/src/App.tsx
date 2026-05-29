import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { CodeXml, FileOutput, FolderSync, Images, PanelLeft, PanelRight } from "lucide-react";
import {
  fetchLibrary,
  pickInputFolder,
  renderJobMessage,
  startRenderOutput,
  updateInputFolder,
  waitForRenderJob,
} from "./api";
import { FolderHints, Navigator, PairStrip, StatusBar, ViewToggles } from "./components/Layout";
import { Pane } from "./components/Pane";
import { RenderProgress } from "./components/RenderProgress";
import {
  fileCoverage,
  pairPageCount,
  pairFiles,
  pruneCounts,
  resolvePairPage,
  totalPages,
  totalPairPages,
} from "./pairing";
import type { LibraryResponse, PageCounts, PaneKey, RefreshOptions, RenderJob, ResolvedPage, Side } from "./types";

export function App() {
  const [library, setLibrary] = useState<LibraryResponse | null>(null);
  const [pageCounts, setPageCounts] = useState<PageCounts>({ input: {}, output: {} });
  const [globalPage, setGlobalPage] = useState(1);
  const [zoom, setZoom] = useState(0.7);
  const [includeImages, setIncludeImages] = useState(true);
  const [visiblePanes, setVisiblePanes] = useState<Record<PaneKey, boolean>>({
    input: true,
    output: true,
    editor: true,
  });
  const [isRendering, setIsRendering] = useState(false);
  const [isChangingInputFolder, setIsChangingInputFolder] = useState(false);
  const [inputFolderDraft, setInputFolderDraft] = useState("");
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

  useEffect(() => {
    if (library?.inputDir) {
      setInputFolderDraft(library.inputDir);
    }
  }, [library?.inputDir]);

  const acceptLibrary = useCallback((data: LibraryResponse) => {
    setLibrary(data);
    setPageCounts({ input: {}, output: {} });
    setGlobalPage(1);
    setInputFolderDraft(data.inputDir);
    setRenderMessage(null);
    setRenderJob(null);
  }, []);

  const applyInputFolder = useCallback(async (inputDir = inputFolderDraft) => {
    setError(null);
    setIsChangingInputFolder(true);
    try {
      acceptLibrary(await updateInputFolder(inputDir));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsChangingInputFolder(false);
    }
  }, [acceptLibrary, inputFolderDraft]);

  const chooseInputFolder = useCallback(async () => {
    setError(null);
    setIsChangingInputFolder(true);
    try {
      acceptLibrary(await pickInputFolder());
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsChangingInputFolder(false);
    }
  }, [acceptLibrary]);

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
        await refreshLibrary({ resetPage: false });
        if (finished.status === "error") {
          throw new Error(finished.error ?? "Render failed");
        }
      } else {
        await refreshLibrary({ resetPage: false });
      }
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

  const coverage = useMemo(() => fileCoverage(filePairs), [filePairs]);

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

  const pairStartPages = useMemo(() => {
    let nextPage = 1;
    const starts = new Map<string, number>();
    for (const pair of filePairs) {
      starts.set(pair.id, nextPage);
      nextPage += pairPageCount(pair, pageCounts);
    }
    return starts;
  }, [filePairs, pageCounts]);

  const jumpToPair = useCallback(
    (pairId: string) => {
      const firstPairPage = pairStartPages.get(pairId);
      if (firstPairPage) {
        setGlobalPage(firstPairPage);
      }
    },
    [pairStartPages],
  );

  const togglePane = useCallback((pane: PaneKey) => {
    setVisiblePanes((current) => ({ ...current, [pane]: !current[pane] }));
  }, []);

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
  const editorPage = useMemo(() => editorResolvedPage(pairPage.output), [pairPage.output]);
  const visiblePaneCount = Object.values(visiblePanes).filter(Boolean).length;

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="app-title">
          <h1>Interactive Compare</h1>
          <p>
            Matched files from <code>input</code> and <code>output</code>, paged together by
            filename.
          </p>
        </div>
        <StatusBar currentPage={currentPage} totals={totals} coverage={coverage} />
        <div className="actions">
          <button
            className="action-primary"
            type="button"
            disabled={isRendering || isChangingInputFolder}
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
          <button className="action-secondary" type="button" onClick={() => void refreshLibrary()} title="Refresh folder scan">
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
          <ViewToggles visiblePanes={visiblePanes} onToggle={togglePane} />
        </div>

        <section className="compare-toolbar" aria-label="Compare controls">
          <Navigator
            canMove={canMove}
            currentPage={currentPage}
            totalPages={totals.combined}
            onPageChange={setGlobalPage}
          />
          <PairStrip
            pairPage={pairPage}
            pairs={filePairs}
            pairIndex={filePairs.findIndex((pair) => pair.id === pairPage.pair?.id)}
            totalPairs={filePairs.length}
            onPairChange={jumpToPair}
          />
          <FolderHints
            library={library}
            inputFolderDraft={inputFolderDraft}
            inputFolderDisabled={isRendering || isChangingInputFolder}
            onInputFolderDraftChange={setInputFolderDraft}
            onApplyInputFolder={(inputDir) => void applyInputFolder(inputDir)}
            onPickInputFolder={() => void chooseInputFolder()}
          />
        </section>
      </header>

      {error ? <div className="error">{error}</div> : null}
      {renderMessage && !error ? <RenderProgress message={renderMessage} job={renderJob} /> : null}

      <section
        className={`compare-grid ${visiblePaneCount === 0 ? "empty" : ""}`}
        style={
          visiblePaneCount > 0
            ? ({ "--visible-pane-count": visiblePaneCount } as CSSProperties)
            : undefined
        }
      >
        {visiblePanes.input ? (
          <Pane
            title="Input"
            icon={<PanelLeft size={18} />}
            side="input"
            resolved={pairPage.input}
            total={totals.input}
            zoom={zoom}
            onPageCount={updateCount}
          />
        ) : null}
        {visiblePanes.output ? (
          <Pane
            title="Output"
            icon={<PanelRight size={18} />}
            side="output"
            resolved={pairPage.output}
            total={totals.output}
            zoom={zoom}
            onPageCount={updateCount}
          />
        ) : null}
        {visiblePanes.editor ? (
          <Pane
            title="Editor"
            icon={<CodeXml size={18} />}
            side="output"
            resolved={editorPage}
            total={totals.output}
            zoom={zoom}
            renderMode="editor"
            onPageCount={updateCount}
          />
        ) : null}
        {visiblePaneCount === 0 ? (
          <div className="empty-compare-grid">
            <strong>No panes visible</strong>
            <span>Use the Input, Output, or Editor toggles above to bring a view back.</span>
          </div>
        ) : null}
      </section>
    </main>
  );
}

function editorResolvedPage(output: ResolvedPage): ResolvedPage {
  if (output.file?.kind === "html") {
    return output;
  }
  return { ...output, file: null, localPage: 0 };
}

async function trackRenderJob(
  started: RenderJob,
  setRenderJob: (job: RenderJob) => void,
  setRenderMessage: (message: string) => void,
  refreshLibrary: (options?: RefreshOptions) => Promise<void>,
) {
  let lastProcessed = started.completed + (started.failed ?? 0);
  return waitForRenderJob(async (job) => {
    setRenderJob(job);
    setRenderMessage(renderJobMessage(job));
    const processed = job.completed + (job.failed ?? 0);
    if (processed !== lastProcessed) {
      lastProcessed = processed;
      await refreshLibrary({ resetPage: false });
    }
  });
}
