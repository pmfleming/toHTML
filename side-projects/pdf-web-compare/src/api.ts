import type { LibraryResponse, RenderJob } from "./types";
import { sleep } from "./utils";

export async function fetchLibrary(): Promise<LibraryResponse> {
  const response = await fetch("/api/library");
  if (!response.ok) {
    throw new Error(`Library request failed: ${response.status}`);
  }
  return (await response.json()) as LibraryResponse;
}

export async function startRenderOutput(includeImages: boolean): Promise<RenderJob> {
  const response = await fetch("/api/render-output", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ includeImages }),
  });
  if (!response.ok && response.status !== 409) {
    const text = await response.text();
    throw new Error(text || `Render request failed: ${response.status}`);
  }
  return (await response.json()) as RenderJob;
}

export async function fetchRenderJob(): Promise<RenderJob> {
  const response = await fetch("/api/render-status");
  if (!response.ok) {
    throw new Error(`Render status request failed: ${response.status}`);
  }
  return (await response.json()) as RenderJob;
}

export async function waitForRenderJob(
  onProgress: (job: RenderJob) => void | Promise<void>,
): Promise<RenderJob> {
  for (;;) {
    await sleep(750);
    const job = await fetchRenderJob();
    await onProgress(job);
    if (job.status !== "running") {
      return job;
    }
  }
}

export function renderJobMessage(job: RenderJob) {
  const mode = job.includeImages ? "images on" : "images off";
  if (job.status === "running") {
    const active = job.current ? `: ${job.current}` : "";
    const current = job.total > 0 ? Math.min(job.completed + 1, job.total) : 0;
    return `Generating ${current} of ${job.total} (${mode})${active}`;
  }
  if (job.status === "done") {
    if (job.total === 0) {
      return `No PDFs found in input. Scanned ${job.scanned} files.`;
    }
    return `Generated ${job.completed} PDF ${job.completed === 1 ? "output" : "outputs"} with ${mode}.`;
  }
  if (job.status === "error") {
    return `Render failed after ${job.completed} of ${job.total}: ${job.error ?? "unknown error"}`;
  }
  return `Ready to render with ${mode}.`;
}
