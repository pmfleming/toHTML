import type { RenderJob } from "../types";

export function RenderProgress({ message, job }: { message: string; job: RenderJob | null }) {
  const percent = job && job.total > 0 ? Math.round((job.completed / job.total) * 100) : 100;
  return (
    <section className="render-progress" aria-label="Background generation progress">
      <div className="render-progress-row">
        <strong>{message}</strong>
        {job ? <span>{job.completed}/{job.total} generated</span> : null}
      </div>
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${percent}%` }} />
      </div>
    </section>
  );
}
