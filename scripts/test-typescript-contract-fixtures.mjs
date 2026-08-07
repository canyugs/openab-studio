#!/usr/bin/env node

import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const outputDirectory = mkdtempSync(resolve(tmpdir(), "openab-studio-schema-ts-"));
writeFileSync(resolve(outputDirectory, "package.json"), '{"type":"module"}\n', "utf8");

try {
  run(process.platform === "win32" ? "pnpm.cmd" : "pnpm", [
    "exec",
    "tsc",
    "--project",
    resolve(repositoryRoot, "schemas/typescript/tsconfig.json"),
    "--outDir",
    outputDirectory,
  ]);
  run(process.execPath, [resolve(outputDirectory, "typescript/fixture-harness.js")]);
} finally {
  rmSync(outputDirectory, { recursive: true, force: true });
}

function run(command, argumentsAfterCommand) {
  const result = spawnSync(command, argumentsAfterCommand, {
    cwd: repositoryRoot,
    stdio: "inherit",
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
