import fs from "node:fs/promises";
import path from "node:path";

const supportedExtensions = new Set([
  ".pdf",
  ".html",
  ".htm",
  ".png",
  ".jpg",
  ".jpeg",
  ".webp",
]);

const generatedOutputVariantSuffixes = [".include-images"];

export async function scanSide(directory, side) {
  const entries = await collectFiles(directory);
  const files = entries
    .filter((entry) => supportedExtensions.has(path.extname(entry).toLowerCase()))
    .filter((entry) => side !== "output" || !isGeneratedOutputVariant(entry))
    .sort((left, right) => left.localeCompare(right, undefined, { numeric: true }));

  return Promise.all(files.map((relativePath) => fileEntry(directory, side, relativePath)));
}

export async function collectFiles(directory, prefix = "") {
  const entries = await fs.readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    if (entry.name.startsWith(".")) {
      continue;
    }
    const relativePath = path.join(prefix, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(directory, relativePath)));
    } else if (entry.isFile()) {
      files.push(relativePath);
    }
  }
  return files;
}

async function fileEntry(directory, side, relativePath) {
  const extension = path.extname(relativePath).toLowerCase().slice(1);
  const urlPath = relativePath.split(path.sep).map(encodeURIComponent).join("/");
  const stats = await fs.stat(path.join(directory, relativePath));
  const normalizedPath = relativePath.replaceAll(path.sep, "/");
  return {
    id: `${side}:${normalizedPath}`,
    side,
    name: path.basename(relativePath),
    relativePath: normalizedPath,
    extension,
    kind: fileKind(extension),
    url: `/files/${side}/${urlPath}?v=${Math.trunc(stats.mtimeMs)}`,
  };
}

function fileKind(extension) {
  if (extension === "pdf") {
    return "pdf";
  }
  if (extension === "html" || extension === "htm") {
    return "html";
  }
  return "image";
}

function isGeneratedOutputVariant(relativePath) {
  const parsed = path.parse(relativePath);
  const stem = parsed.name.toLocaleLowerCase();
  return generatedOutputVariantSuffixes.some((suffix) => stem.endsWith(suffix));
}
