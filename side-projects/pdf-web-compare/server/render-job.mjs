import fs from "node:fs/promises";
import path from "node:path";
import { collectFiles } from "./file-library.mjs";
import { outputHtmlPath, runToHtml } from "./tohtml-runner.mjs";

export function createRenderJobService({ inputDir, outputDir, mainProjectRoot }) {
  let job = idleRenderJob();

  return {
    current() {
      return job;
    },

    async start({ includeImages }) {
      if (job.status === "running") {
        return { statusCode: 409, job };
      }

      const pdfs = await inputPdfs(inputDir);
      job = createRunningJob(pdfs, includeImages);
      if (pdfs.length > 0) {
        void runJob({ pdfs, initialJob: job, inputDir, outputDir, mainProjectRoot, setJob });
      }
      return { statusCode: pdfs.length === 0 ? 200 : 202, job };
    },
  };

  function setJob(nextJob) {
    job = nextJob;
  }

}

function idleRenderJob() {
  return {
    status: "idle",
    includeImages: false,
    scanned: 0,
    total: 0,
    completed: 0,
    current: null,
    results: [],
    error: null,
    startedAt: null,
    finishedAt: null,
  };
}

async function inputPdfs(inputDir) {
  return (await collectFiles(inputDir)).filter(
    (entry) => path.extname(entry).toLowerCase() === ".pdf",
  );
}

async function runJob({ pdfs, initialJob, inputDir, outputDir, mainProjectRoot, setJob }) {
  let job = initialJob;
  try {
    for (const relativePath of pdfs) {
      job = updateCurrent(job, relativePath);
      setJob(job);
      const outputPath = outputHtmlPath(outputDir, relativePath);
      await fs.mkdir(path.dirname(outputPath), { recursive: true });
      await runToHtml({
        inputPath: path.join(inputDir, relativePath),
        outputPath,
        includeImages: initialJob.includeImages,
        mainProjectRoot,
      });
      job = completeFile(job, relativePath, outputDir, outputPath);
      setJob(job);
    }
    setJob(finishJob(job, "done"));
  } catch (error) {
    setJob(finishJob(job, "error", error));
  }

  function updateCurrent(currentJob, relativePath) {
    return {
      ...currentJob,
      current: relativePath.replaceAll(path.sep, "/"),
    };
  }
}

function createRunningJob(pdfs, includeImages) {
  return {
    status: pdfs.length === 0 ? "done" : "running",
    includeImages,
    scanned: pdfs.length,
    total: pdfs.length,
    completed: 0,
    current: pdfs[0]?.replaceAll(path.sep, "/") ?? null,
    results: [],
    error: null,
    startedAt: new Date().toISOString(),
    finishedAt: pdfs.length === 0 ? new Date().toISOString() : null,
  };
}

function completeFile(job, relativePath, outputDir, outputPath) {
  return {
    ...job,
    completed: job.completed + 1,
    results: [
      ...job.results,
      {
        input: relativePath.replaceAll(path.sep, "/"),
        output: path.relative(outputDir, outputPath).replaceAll(path.sep, "/"),
      },
    ],
  };
}

function finishJob(job, status, error = null) {
  return {
    ...job,
    status,
    current: null,
    error: error ? errorMessage(error) : null,
    finishedAt: new Date().toISOString(),
  };
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
