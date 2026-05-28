import type { RenderJob } from "../types";

export function RenderProgress({ message, job }: { message: string; job: RenderJob | null }) {
  const processed = job ? job.completed + (job.failed ?? 0) : 0;
  const percent = job && job.total > 0 ? Math.round((processed / job.total) * 100) : 100;
  return (
    <section className="render-progress" aria-label="Background generation progress">
      <div className="render-progress-row">
        <strong>{message}</strong>
        {job ? (
          <span>
            {job.completed}/{job.total} generated
            {(job.failed ?? 0) > 0 ? `, ${job.failed} failed` : ""}
          </span>
        ) : null}
      </div>
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${percent}%` }} />
      </div>
    </section>
  );
}
