import { useCallback } from "react";
import { DocumentPage, EmptySide } from "./DocumentPage";
import type { PaneProps } from "../types";

export function Pane({
  title,
  icon,
  side,
  resolved,
  total,
  zoom,
  renderMode = "source",
  onPageCount,
}: PaneProps) {
  const fileId = resolved.file?.id;
  const handlePageCount = useCallback(
    (count: number) => {
      if (fileId) {
        onPageCount(side, fileId, count);
      }
    },
    [fileId, onPageCount, side],
  );

  return (
    <section className="pane">
      <div className="pane-header">
        <div className="pane-heading">
          {icon}
          <strong>{title}</strong>
          <span>{total} pages</span>
        </div>
        <div className="file-strip">
          {resolved.file ? (
            <>
              <strong>{resolved.file.name}</strong>
              <span>
                file page {resolved.localPage} of {resolved.totalPagesInFile}
              </span>
            </>
          ) : (
            <span>No page at this position</span>
          )}
        </div>
      </div>
      <div className="stage">
        {resolved.file ? (
          <DocumentPage
            file={resolved.file}
            localPage={resolved.localPage}
            zoom={zoom}
            renderMode={renderMode}
            onPageCount={handlePageCount}
          />
        ) : (
          <EmptySide side={side} />
        )}
      </div>
    </section>
  );
}
