import express from "express";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { defaultInputDir, isProduction, mainProjectRoot, outputDir, port, root } from "./config.mjs";
import { editorPreviewHtml } from "./editor-preview.mjs";
import { scanSide } from "./file-library.mjs";
import { createRenderJobService } from "./render-job.mjs";

let inputDir = defaultInputDir;

await fs.mkdir(inputDir, { recursive: true });
await fs.mkdir(outputDir, { recursive: true });

const app = express();
const renderJobs = createRenderJobService({
  getInputDir: () => inputDir,
  outputDir,
  mainProjectRoot,
});

app.use(express.json({ limit: "1mb" }));
app.use("/files/input", (request, response, next) => {
  staticFiles(inputDir)(request, response, next);
});
app.use("/files/output", staticFiles(outputDir));

app.get("/api/library", async (_request, response, next) => {
  try {
    response.json(await libraryPayload());
  } catch (error) {
    next(error);
  }
});

app.post("/api/input-folder", async (request, response, next) => {
  try {
    ensureNoActiveRender();
    inputDir = await validateInputDir(request.body?.inputDir);
    response.json(await libraryPayload());
  } catch (error) {
    next(error);
  }
});

app.post("/api/input-folder/pick", async (_request, response, next) => {
  try {
    ensureNoActiveRender();
    const pickedDir = await pickInputDirectory(inputDir);
    if (pickedDir) {
      inputDir = await validateInputDir(pickedDir);
    }
    response.json(await libraryPayload());
  } catch (error) {
    next(error);
  }
});

app.post("/api/render-output", async (request, response, next) => {
  try {
    const result = await renderJobs.start({ includeImages: Boolean(request.body?.includeImages) });
    response.status(result.statusCode).json(result.job);
  } catch (error) {
    next(error);
  }
});

app.get("/api/render-status", (_request, response) => {
  response.json(renderJobs.current());
});

app.get("/api/editor-preview", async (request, response, next) => {
  try {
    const html = await editorPreviewHtml({
      mainProjectRoot,
      outputDir,
      relativePath: request.query.file,
    });
    response.type("html").send(html);
  } catch (error) {
    next(error);
  }
});

if (isProduction) {
  app.use(express.static(path.join(root, "dist")));
  app.get(/.*/, (_request, response) => {
    response.sendFile(path.join(root, "dist", "index.html"));
  });
} else {
  const { createServer } = await import("vite");
  const vite = await createServer({
    root,
    server: { middlewareMode: true, hmr: false },
    appType: "spa",
  });
  app.use(vite.middlewares);
}

app.listen(port, "127.0.0.1", () => {
  console.log(`Interactive compare: http://127.0.0.1:${port}`);
  console.log(`Input folder:  ${inputDir}`);
  console.log(`Output folder: ${outputDir}`);
});

async function libraryPayload() {
  return {
    inputDir,
    outputDir,
    input: await scanSide(inputDir, "input"),
    output: await scanSide(outputDir, "output"),
  };
}

function staticFiles(directory) {
  return express.static(directory, {
    fallthrough: false,
    setHeaders(response) {
      response.setHeader("Cross-Origin-Resource-Policy", "same-origin");
    },
  });
}

function ensureNoActiveRender() {
  if (renderJobs.current().status !== "running") {
    return;
  }
  throw httpError(409, "Wait for the current output generation to finish before changing input folders.");
}

async function validateInputDir(value) {
  const requested = typeof value === "string" ? value.trim() : "";
  if (!requested) {
    throw httpError(400, "Enter an input folder path.");
  }

  const resolved = path.resolve(path.isAbsolute(requested) ? requested : path.join(mainProjectRoot, requested));
  let stats;
  try {
    stats = await fs.stat(resolved);
  } catch {
    throw httpError(400, `Input folder does not exist: ${resolved}`);
  }
  if (!stats.isDirectory()) {
    throw httpError(400, `Input path is not a folder: ${resolved}`);
  }
  return resolved;
}

async function pickInputDirectory(selectedPath) {
  if (os.platform() !== "win32") {
    throw httpError(501, "Native folder picking is only available on Windows. Enter a folder path manually.");
  }

  const script = [
    "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8",
    "Add-Type -AssemblyName System.Windows.Forms",
    "$dialog = New-Object System.Windows.Forms.FolderBrowserDialog",
    "$dialog.Description = 'Choose PDF input folder'",
    "$dialog.ShowNewFolderButton = $true",
    `$dialog.SelectedPath = ${powershellString(selectedPath)}`,
    "if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { [Console]::WriteLine($dialog.SelectedPath) }",
  ].join("; ");

  return new Promise((resolve, reject) => {
    execFile(
      "powershell.exe",
      ["-NoProfile", "-STA", "-ExecutionPolicy", "Bypass", "-Command", script],
      { encoding: "utf8" },
      (error, stdout, stderr) => {
        if (error) {
          reject(httpError(500, stderr.trim() || error.message));
          return;
        }
        resolve(stdout.trim() || null);
      },
    );
  });
}

function powershellString(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function httpError(statusCode, message) {
  const error = new Error(message);
  error.statusCode = statusCode;
  return error;
}

app.use((error, _request, response, _next) => {
  const statusCode = Number.isInteger(error.statusCode) ? error.statusCode : 500;
  response.status(statusCode).send(error.message || "Server error");
});
