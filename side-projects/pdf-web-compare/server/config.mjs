import path from "node:path";
import { fileURLToPath } from "node:url";

const serverDir = path.dirname(fileURLToPath(import.meta.url));

export const root = path.resolve(serverDir, "..");
export const mainProjectRoot = path.resolve(root, "../..");
export const inputDir = path.join(mainProjectRoot, "input");
export const outputDir = path.join(mainProjectRoot, "output");
export const port = Number(process.env.PORT ?? 5177);
export const isProduction = process.env.NODE_ENV === "production";
