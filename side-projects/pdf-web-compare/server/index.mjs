import express from "express";
import fs from "node:fs/promises";
import path from "node:path";
import { inputDir, isProduction, mainProjectRoot, outputDir, port, root } from "./config.mjs";
import { scanSide } from "./file-library.mjs";
import { createRenderJobService } from "./render-job.mjs";

await fs.mkdir(inputDir, { recursive: true });
await fs.mkdir(outputDir, { recursive: true });

const app = express();
const renderJobs = createRenderJobService({ inputDir, outputDir, mainProjectRoot });

app.use(express.json({ limit: "1mb" }));
app.use("/files/input", staticFiles(inputDir));
app.use("/files/output", staticFiles(outputDir));

app.get("/api/library", async (_request, response, next) => {
  try {
    response.json({
      inputDir,
      outputDir,
      input: await scanSide(inputDir, "input"),
      output: await scanSide(outputDir, "output"),
    });
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

if (isProduction) {
  app.use(express.static(path.join(root, "dist")));
  app.get("*", (_request, response) => {
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

function staticFiles(directory) {
  return express.static(directory, {
    fallthrough: false,
    setHeaders(response) {
      response.setHeader("Cross-Origin-Resource-Policy", "same-origin");
    },
  });
}
