import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import {
  parseCompatibilityPeer,
  parseCompatibilityRequest,
  parseSharedContractDocument,
} from "../generated/typescript/studio-protocol.js";
import { decideCompatibility } from "./compatibility.js";
import {
  SchemaValidationException,
  migratePluginManifest,
  parseWithDefinition,
  validateDefinition,
} from "./validation.js";

const fixtureRoot = resolve(process.cwd(), "schemas/fixtures");
let fixtureCount = 0;

for (const fixture of fixturesIn("compatibility")) {
  fixtureCount += 1;
  runCompatibilityFixture(fixture);
}
for (const fixture of fixturesIn("migrations")) {
  fixtureCount += 1;
  runMigrationFixture(fixture);
}

process.stdout.write(`TypeScript shared-schema fixtures passed: ${fixtureCount}\n`);

function runCompatibilityFixture(fixture: JsonRecord): void {
  switch (fixture.kind) {
    case "compatibility": {
      const request = parseCompatibilityRequest(fixture.request);
      const peer = parseCompatibilityPeer(fixture.peer);
      assert.deepStrictEqual(
        canonicalize(decideCompatibility(request, peer)),
        canonicalize(fixture.expect),
        fixtureName(fixture),
      );
      return;
    }
    case "validation": {
      const definition = requiredString(fixture, "definition");
      const knownExtensions = stringArray(fixture.knownExtensions);
      const result = validateDefinition(definition, fixture.input, { knownExtensions });
      const expected = requiredRecord(fixture, "expect");
      if (expected.valid === true) {
        assert.equal(result.ok, true, fixtureName(fixture));
        parseWithDefinition(definition, fixture.input, { knownExtensions });
      } else {
        assert.equal(result.ok, false, fixtureName(fixture));
        if (!result.ok) {
          assert.equal(result.error.code, expected.code, fixtureName(fixture));
        }
      }
      return;
    }
    case "roundtrip": {
      const knownExtensions = stringArray(fixture.knownExtensions);
      const parsed = parseSharedContractDocument(fixture.input, { knownExtensions });
      const reparsed = JSON.parse(JSON.stringify(parsed)) as unknown;
      assert.deepStrictEqual(
        canonicalize(reparsed),
        canonicalize(fixture.input),
        fixtureName(fixture),
      );
      return;
    }
    default:
      throw new Error(`Unsupported compatibility fixture kind: ${String(fixture.kind)}`);
  }
}

function runMigrationFixture(fixture: JsonRecord): void {
  const expected = requiredRecord(fixture, "expect");
  const knownExtensions = stringArray(fixture.knownExtensions);
  if (typeof expected.error === "string") {
    assert.throws(
      () => migratePluginManifest(fixture.input, { knownExtensions }),
      (error: unknown) =>
        error instanceof SchemaValidationException && error.error.code === expected.error,
      fixtureName(fixture),
    );
    return;
  }

  const result = migratePluginManifest(fixture.input, { knownExtensions });
  assert.equal(result.migrated, expected.migrated, fixtureName(fixture));
  assert.deepStrictEqual(
    canonicalize(result.value),
    canonicalize(expected.value),
    fixtureName(fixture),
  );
}

function fixturesIn(directory: string): JsonRecord[] {
  return readdirSync(resolve(fixtureRoot, directory))
    .filter((entry) => entry.endsWith(".json"))
    .sort()
    .map((entry) =>
      JSON.parse(readFileSync(resolve(fixtureRoot, directory, entry), "utf8")) as JsonRecord,
    );
}

function canonicalize(input: unknown): unknown {
  if (Array.isArray(input)) {
    return input.map(canonicalize);
  }
  if (isRecord(input)) {
    return Object.fromEntries(
      Object.entries(input)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, value]) => [key, canonicalize(value)]),
    );
  }
  return input;
}

function fixtureName(fixture: JsonRecord): string {
  return typeof fixture.name === "string" ? fixture.name : "unnamed fixture";
}

function requiredRecord(record: JsonRecord, field: string): JsonRecord {
  const value = record[field];
  if (!isRecord(value)) {
    throw new Error(`Fixture field ${field} must be an object.`);
  }
  return value;
}

function requiredString(record: JsonRecord, field: string): string {
  const value = record[field];
  if (typeof value !== "string") {
    throw new Error(`Fixture field ${field} must be a string.`);
  }
  return value;
}

function stringArray(value: unknown): string[] {
  if (!Array.isArray(value) || !value.every((entry) => typeof entry === "string")) {
    return [];
  }
  return value;
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

type JsonRecord = Record<string, unknown>;
