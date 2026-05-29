import fs from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";

const htmlExtensions = new Set([".html", ".htm"]);
const previewCache = new Map();

export async function editorPreviewHtml({ mainProjectRoot, outputDir, relativePath }) {
  const safePath = resolveOutputHtml(outputDir, relativePath);
  const stats = await fs.stat(safePath);
  const cacheKey = `${safePath}:${Math.trunc(stats.mtimeMs)}:${stats.size}`;
  const cached = previewCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const html = await renderWithHtmlEditor(mainProjectRoot, safePath);
  previewCache.clear();
  previewCache.set(cacheKey, html);
  return html;
}

function resolveOutputHtml(outputDir, relativePath) {
  if (typeof relativePath !== "string" || !relativePath.trim()) {
    throw httpError(400, "Missing output file path");
  }
  const normalized = relativePath.replaceAll("\\", "/");
  if (normalized.includes("\0") || path.isAbsolute(normalized)) {
    throw httpError(400, "Invalid output file path");
  }
  const extension = path.extname(normalized).toLowerCase();
  if (!htmlExtensions.has(extension)) {
    throw httpError(400, "Editor preview requires an HTML output file");
  }
  const resolvedRoot = path.resolve(outputDir);
  const resolvedFile = path.resolve(outputDir, normalized);
  const relative = path.relative(resolvedRoot, resolvedFile);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw httpError(400, "Output file path escapes the output folder");
  }
  return resolvedFile;
}

function renderWithHtmlEditor(mainProjectRoot, htmlPath) {
  const htmlEditorDir = path.join(mainProjectRoot, "side-projects", "html-editor");
  const manifestPath = path.join(htmlEditorDir, "Cargo.toml");
  const cargo = process.env.CARGO ?? (process.platform === "win32" ? "cargo.exe" : "cargo");
  const args = [
    "run",
    "--quiet",
    "--manifest-path",
    manifestPath,
    "--bin",
    "editor-preview",
    "--",
    htmlPath,
  ];

  return new Promise((resolve, reject) => {
    const child = spawn(cargo, args, { cwd: htmlEditorDir, windowsHide: true });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve(Buffer.concat(stdout).toString("utf8"));
        return;
      }
      reject(httpError(500, Buffer.concat(stderr).toString("utf8") || "Editor preview failed"));
    });
  });
}

function httpError(statusCode, message) {
  const error = new Error(message);
  error.statusCode = statusCode;
  return error;
}
