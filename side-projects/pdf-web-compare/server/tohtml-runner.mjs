import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";

const generatedOutputVariantSuffixes = [".include-images"];

export function outputHtmlPath(outputDir, relativePath) {
  const parsed = path.parse(relativePath);
  return path.join(outputDir, parsed.dir, `${parsed.name}.html`);
}

export async function runToHtml({
  inputPath,
  outputPath,
  includeImages,
  mainProjectRoot,
}) {
  const tempOutputPath = `${outputPath}.${process.pid}.${Date.now()}.tmp`;
  const { command, args } = await resolveCommand({
    inputPath,
    outputPath: tempOutputPath,
    includeImages,
    mainProjectRoot,
  });

  await spawnCommand(command, args, mainProjectRoot);
  try {
    const producedOutputPath = await findProducedOutput(tempOutputPath);
    await swapFile(producedOutputPath, outputPath);
    await removeFiles([
      ...generatedVariantPaths(tempOutputPath),
      ...generatedVariantPaths(outputPath),
    ]);
  } catch (error) {
    await removeFiles([tempOutputPath, ...generatedVariantPaths(tempOutputPath)]);
    throw error;
  }
}

async function resolveCommand({ inputPath, outputPath, includeImages, mainProjectRoot }) {
  const binary = path.join(
    mainProjectRoot,
    "target",
    "debug",
    process.platform === "win32" ? "tohtml.exe" : "tohtml",
  );
  const args = toHtmlArgs(inputPath, outputPath, includeImages);

  try {
    await fs.access(binary);
    return { command: binary, args };
  } catch {
    return {
      command: "cargo",
      args: [
        "run",
        "--quiet",
        "--manifest-path",
        path.join(mainProjectRoot, "Cargo.toml"),
        "--",
        ...args,
      ],
    };
  }
}

function toHtmlArgs(inputPath, outputPath, includeImages) {
  const args = [inputPath, "--output", outputPath];
  if (includeImages) {
    args.push("--include-images");
  }
  return args;
}

function spawnCommand(command, args, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, windowsHide: true });
    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(commandError(command, code, stdout, stderr));
      }
    });
  });
}

function commandError(command, code, stdout, stderr) {
  return new Error(
    [
      `${path.basename(command)} exited with code ${code}`,
      stdout.trim() ? `stdout:\n${stdout.trim()}` : "",
      stderr.trim() ? `stderr:\n${stderr.trim()}` : "",
    ]
      .filter(Boolean)
      .join("\n\n"),
  );
}

async function swapFile(tempPath, outputPath) {
  try {
    await fs.rename(tempPath, outputPath);
  } catch (error) {
    if (!["EEXIST", "EPERM"].includes(error?.code)) {
      throw error;
    }
    await fs.rm(outputPath, { force: true });
    await fs.rename(tempPath, outputPath);
  }
}

async function findProducedOutput(outputPath) {
  if (await fileExists(outputPath)) {
    return outputPath;
  }

  for (const variantPath of generatedVariantPaths(outputPath)) {
    if (await fileExists(variantPath)) {
      return variantPath;
    }
  }

  throw new Error(`Expected output was not created: ${outputPath}`);
}

function generatedVariantPaths(outputPath) {
  const parsed = path.parse(outputPath);
  return generatedOutputVariantSuffixes.map((suffix) =>
    path.join(parsed.dir, `${parsed.name}${suffix}${parsed.ext}`),
  );
}

async function removeFiles(paths) {
  await Promise.all(paths.map((filePath) => fs.rm(filePath, { force: true }).catch(() => {})));
}

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}
