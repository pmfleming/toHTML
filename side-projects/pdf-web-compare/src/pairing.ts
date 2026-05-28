import type {
  FileCoverage,
  FilePair,
  LibraryFile,
  LibraryResponse,
  PageCounts,
  ResolvedPage,
  ResolvedPairPage,
} from "./types";

const GENERATED_OUTPUT_VARIANT_SUFFIXES = [".include-images"];

export function totalPages(files: LibraryFile[], counts: Record<string, number>) {
  return files.reduce((sum, file) => sum + (counts[file.id] ?? 1), 0);
}

export function totalPairPages(pairs: FilePair[], counts: PageCounts) {
  return pairs.reduce((sum, pair) => sum + pairPageCount(pair, counts), 0);
}

export function pairPageCount(pair: FilePair, counts: PageCounts) {
  return Math.max(sidePageCount(pair.input, counts.input), sidePageCount(pair.output, counts.output));
}

export function resolvePairPage(
  pairs: FilePair[],
  counts: PageCounts,
  globalPage: number,
): ResolvedPairPage {
  let remaining = globalPage;
  for (const pair of pairs) {
    const count = pairPageCount(pair, counts);
    if (remaining <= count) {
      return {
        pair,
        localPage: remaining,
        pairPageCount: count,
        input: resolveSidePage(pair.input, counts.input, remaining, globalPage),
        output: resolveSidePage(pair.output, counts.output, remaining, globalPage),
      };
    }
    remaining -= count;
  }
  return emptyPairPage(globalPage);
}

export function pairFiles(inputFiles: LibraryFile[], outputFiles: LibraryFile[]): FilePair[] {
  const outputBuckets = bucketByMatchKey(outputFiles);
  const pairs = inputFiles.map((input, index) => {
    const key = matchKey(input);
    return makePair(key, input, takeBucketFile(outputBuckets, key), index);
  });

  for (const [key, outputs] of outputBuckets) {
    for (const output of outputs) {
      pairs.push(makePair(key, null, output, pairs.length));
    }
  }

  return pairs;
}

export function fileCoverage(pairs: FilePair[]): FileCoverage {
  return pairs.reduce<FileCoverage>(
    (summary, pair) => ({
      input: summary.input + (pair.input ? 1 : 0),
      output: summary.output + (pair.output ? 1 : 0),
      matched: summary.matched + (pair.input && pair.output ? 1 : 0),
      missingOutput: summary.missingOutput + (pair.input && !pair.output ? 1 : 0),
      extraOutput: summary.extraOutput + (!pair.input && pair.output ? 1 : 0),
    }),
    { input: 0, output: 0, matched: 0, missingOutput: 0, extraOutput: 0 },
  );
}

export function pruneCounts(current: PageCounts, library: LibraryResponse): PageCounts {
  return {
    input: pruneSideCounts(current.input, library.input),
    output: pruneSideCounts(current.output, library.output),
  };
}

function sidePageCount(file: LibraryFile | null, counts: Record<string, number>) {
  return file ? counts[file.id] ?? 1 : 0;
}

function resolveSidePage(
  file: LibraryFile | null,
  counts: Record<string, number>,
  localPage: number,
  globalPage: number,
): ResolvedPage {
  if (!file) {
    return { file: null, localPage: 0, totalPagesInFile: 0, globalPage };
  }
  const count = counts[file.id] ?? 1;
  if (localPage <= count) {
    return { file, localPage, totalPagesInFile: count, globalPage };
  }
  return { file: null, localPage: 0, totalPagesInFile: count, globalPage };
}

function emptyPairPage(globalPage: number): ResolvedPairPage {
  return {
    pair: null,
    localPage: 0,
    pairPageCount: 0,
    input: { file: null, localPage: 0, totalPagesInFile: 0, globalPage },
    output: { file: null, localPage: 0, totalPagesInFile: 0, globalPage },
  };
}

function bucketByMatchKey(files: LibraryFile[]) {
  const buckets = new Map<string, LibraryFile[]>();
  for (const file of files) {
    const key = matchKey(file);
    buckets.set(key, sortMatchFiles([...(buckets.get(key) ?? []), file]));
  }
  return buckets;
}

function takeBucketFile(buckets: Map<string, LibraryFile[]>, key: string) {
  const files = buckets.get(key) ?? [];
  const file = files.shift() ?? null;
  buckets.set(key, files);
  return file;
}

function makePair(
  key: string,
  input: LibraryFile | null,
  output: LibraryFile | null,
  index: number,
): FilePair {
  return {
    id: `pair:${key}:${index}`,
    key,
    name: stripExtension(input?.name ?? output?.name ?? "Unmatched file"),
    input,
    output,
  };
}

function matchKey(file: LibraryFile) {
  return normalizeMatchKey(stripGeneratedVariantSuffix(stripExtension(file.relativePath)));
}

function normalizeMatchKey(value: string) {
  return value
    .toLocaleLowerCase()
    .replace(/\\/g, "/")
    .replace(/[^a-z0-9/]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function stripExtension(value: string) {
  return value.replace(/\.[^.\\/]+$/, "");
}

function stripGeneratedVariantSuffix(value: string) {
  const lowerValue = value.toLocaleLowerCase();
  const suffix = GENERATED_OUTPUT_VARIANT_SUFFIXES.find((variant) => lowerValue.endsWith(variant));
  return suffix ? value.slice(0, -suffix.length) : value;
}

function sortMatchFiles(files: LibraryFile[]) {
  return [...files].sort(compareMatchFilePreference);
}

function compareMatchFilePreference(a: LibraryFile, b: LibraryFile) {
  return matchFileVariantRank(a) - matchFileVariantRank(b)
    || a.relativePath.localeCompare(b.relativePath, undefined, { numeric: true });
}

function matchFileVariantRank(file: LibraryFile) {
  const stem = stripExtension(file.relativePath).toLocaleLowerCase();
  return GENERATED_OUTPUT_VARIANT_SUFFIXES.some((suffix) => stem.endsWith(suffix)) ? 1 : 0;
}

function pruneSideCounts(current: Record<string, number>, files: LibraryFile[]) {
  const valid = new Set(files.map((file) => file.id));
  return Object.fromEntries(Object.entries(current).filter(([id]) => valid.has(id)));
}
