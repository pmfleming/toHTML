import { DocumentPage, EmptySide } from "./DocumentPage";
import type { PaneProps } from "../types";

export function Pane({ title, icon, side, resolved, total, zoom, onPageCount }: PaneProps) {
  return (
    <section className="pane">
      <div className="pane-header">
        <div>
          {icon}
          <strong>{title}</strong>
        </div>
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
      <div className="stage">
        {resolved.file ? (
          <DocumentPage
            file={resolved.file}
            localPage={resolved.localPage}
            zoom={zoom}
            onPageCount={(count) => onPageCount(side, resolved.file!.id, count)}
          />
        ) : (
          <EmptySide side={side} />
        )}
      </div>
    </section>
  );
}
