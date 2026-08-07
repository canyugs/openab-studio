import { sharedSchema } from "../generated/typescript/schema.js";

export interface ValidationOptions {
  knownExtensions?: readonly string[];
}

export interface SchemaValidationError {
  code: string;
  path: string;
}

export type ValidationResult<T> =
  | {
      ok: true;
      value: T;
    }
  | {
      ok: false;
      error: SchemaValidationError;
    };

export class SchemaValidationException extends Error {
  readonly error: SchemaValidationError;

  constructor(error: SchemaValidationError) {
    super(`${error.code} at ${error.path}`);
    this.name = "SchemaValidationException";
    this.error = error;
  }
}

type SchemaNode = Record<string, unknown>;
type JsonRecord = Record<string, unknown>;

const schema = sharedSchema as unknown as SchemaNode;

export function validateDefinition(
  definition: string,
  input: unknown,
  options: ValidationOptions = {},
): ValidationResult<unknown> {
  const definitions = asRecord(schema.$defs);
  const definitionSchema = definitions?.[definition];
  if (!isSchemaNode(definitionSchema)) {
    return failure("schema-definition-unavailable", "$");
  }

  const result = validateNode(definitionSchema, input, "$", schema);
  if (!result.ok) {
    return result;
  }

  if (definition === "PluginManifest") {
    return validateRequiredExtensions(input, options.knownExtensions ?? []);
  }

  return success(input);
}

export function parseWithDefinition<T>(
  definition: string,
  input: unknown,
  options: ValidationOptions = {},
): T {
  const result = validateDefinition(definition, input, options);
  if (!result.ok) {
    throw new SchemaValidationException(result.error);
  }
  return result.value as T;
}

export function migratePluginManifest(
  input: unknown,
  options: ValidationOptions = {},
): { migrated: boolean; value: unknown } {
  if (!isRecord(input)) {
    throw new SchemaValidationException({
      code: "schema-type-mismatch",
      path: "$",
    });
  }
  if (typeof input.schemaVersion !== "string") {
    throw new SchemaValidationException({
      code: "schema-required-field",
      path: "$.schemaVersion",
    });
  }

  const migrations = Array.isArray(schema["x-openab-migrations"])
    ? schema["x-openab-migrations"]
    : [];
  const migration = migrations.find(
    (candidate) =>
      isRecord(candidate) &&
      candidate.definition === "PluginManifest" &&
      candidate.fromSchemaVersion === input.schemaVersion,
  );

  if (isRecord(migration)) {
    const value = cloneJson(input);
    const rename = asRecord(migration.rename) ?? {};
    for (const [from, target] of Object.entries(rename)) {
      if (typeof target !== "string" || !(from in value)) {
        continue;
      }
      if (target in value) {
        throw new SchemaValidationException({
          code: "schema-migration-conflict",
          path: "$",
        });
      }
      value[target] = value[from];
      delete value[from];
    }
    value.schemaVersion = migration.toSchemaVersion;

    const result = validateDefinition("PluginManifest", value, options);
    if (!result.ok) {
      throw new SchemaValidationException(result.error);
    }
    return { migrated: true, value };
  }

  if (input.schemaVersion !== schema["x-openab-schema-version"]) {
    throw new SchemaValidationException({
      code: "schema-migration-unavailable",
      path: "$.schemaVersion",
    });
  }

  const result = validateDefinition("PluginManifest", input, options);
  if (!result.ok) {
    throw new SchemaValidationException(result.error);
  }
  return { migrated: false, value: input };
}

function validateNode(
  node: SchemaNode,
  input: unknown,
  path: string,
  root: SchemaNode,
): ValidationResult<unknown> {
  if (typeof node.$ref === "string") {
    const reference = resolveReference(root, node.$ref);
    return reference
      ? validateNode(reference, input, path, root)
      : failure("schema-reference-unavailable", path);
  }

  if ("const" in node && input !== node.const) {
    return failure("schema-const-mismatch", path);
  }
  if (Array.isArray(node.enum) && !node.enum.includes(input)) {
    return failure("schema-enum-mismatch", path);
  }

  switch (node.type) {
    case "object":
      return validateObject(node, input, path, root);
    case "array":
      return validateArray(node, input, path, root);
    case "string":
      return validateString(node, input, path);
    case "integer":
      return validateNumber(node, input, path, true);
    case "number":
      return validateNumber(node, input, path, false);
    case "boolean":
      return typeof input === "boolean"
        ? success(input)
        : failure("schema-type-mismatch", path);
    default:
      return failure("schema-type-unsupported", path);
  }
}

function validateObject(
  node: SchemaNode,
  input: unknown,
  path: string,
  root: SchemaNode,
): ValidationResult<unknown> {
  if (!isRecord(input)) {
    return failure("schema-type-mismatch", path);
  }

  const properties = asRecord(node.properties) ?? {};
  const required = Array.isArray(node.required) ? node.required : [];
  for (const field of required) {
    if (typeof field === "string" && !(field in input)) {
      return failure("schema-required-field", childPath(path, field));
    }
  }

  for (const [field, value] of Object.entries(input)) {
    const propertySchema = properties[field];
    if (isSchemaNode(propertySchema)) {
      const result = validateNode(
        propertySchema,
        value,
        childPath(path, field),
        root,
      );
      if (!result.ok) {
        return result;
      }
      continue;
    }

    if (node.additionalProperties === false) {
      return failure("schema-unknown-field", childPath(path, field));
    }
    if (isSchemaNode(node.additionalProperties)) {
      const result = validateNode(
        node.additionalProperties,
        value,
        childPath(path, field),
        root,
      );
      if (!result.ok) {
        return result;
      }
    }
  }

  const propertyNames = asRecord(node.propertyNames);
  if (typeof propertyNames?.pattern === "string") {
    for (const field of Object.keys(input)) {
      if (!matchesSupportedPattern(field, propertyNames.pattern)) {
        return failure(
          "schema-invalid-extension-namespace",
          childPath(path, field),
        );
      }
    }
  }

  return success(input);
}

function validateArray(
  node: SchemaNode,
  input: unknown,
  path: string,
  root: SchemaNode,
): ValidationResult<unknown> {
  if (!Array.isArray(input)) {
    return failure("schema-type-mismatch", path);
  }
  if (typeof node.minItems === "number" && input.length < node.minItems) {
    return failure("schema-min-items", path);
  }
  if (!isSchemaNode(node.items)) {
    return failure("schema-type-unsupported", path);
  }

  for (const [index, item] of input.entries()) {
    const result = validateNode(node.items, item, `${path}[${index}]`, root);
    if (!result.ok) {
      return result;
    }
  }
  return success(input);
}

function validateString(
  node: SchemaNode,
  input: unknown,
  path: string,
): ValidationResult<unknown> {
  if (typeof input !== "string") {
    return failure("schema-type-mismatch", path);
  }
  if (
    typeof node.minLength === "number" &&
    [...input].length < node.minLength
  ) {
    return failure("schema-min-length", path);
  }
  return success(input);
}

function validateNumber(
  node: SchemaNode,
  input: unknown,
  path: string,
  integer: boolean,
): ValidationResult<unknown> {
  if (
    typeof input !== "number" ||
    !Number.isFinite(input) ||
    (integer && !Number.isInteger(input))
  ) {
    return failure("schema-type-mismatch", path);
  }
  if (typeof node.minimum === "number" && input < node.minimum) {
    return failure("schema-minimum", path);
  }
  if (typeof node.maximum === "number" && input > node.maximum) {
    return failure("schema-maximum", path);
  }
  return success(input);
}

function validateRequiredExtensions(
  input: unknown,
  knownExtensions: readonly string[],
): ValidationResult<unknown> {
  if (!isRecord(input) || !isRecord(input.compatibility)) {
    return failure("schema-type-mismatch", "$");
  }
  const requiredExtensions = input.compatibility.requiredExtensions;
  if (requiredExtensions === undefined) {
    return success(input);
  }
  if (!Array.isArray(requiredExtensions)) {
    return failure(
      "schema-type-mismatch",
      "$.compatibility.requiredExtensions",
    );
  }

  const extensions = isRecord(input.extensions) ? input.extensions : {};
  for (const [index, namespace] of requiredExtensions.entries()) {
    if (
      typeof namespace !== "string" ||
      !(namespace in extensions) ||
      !knownExtensions.includes(namespace)
    ) {
      return failure(
        "required-extension-unavailable",
        `$.compatibility.requiredExtensions[${index}]`,
      );
    }
  }
  return success(input);
}

function resolveReference(
  root: SchemaNode,
  reference: string,
): SchemaNode | undefined {
  if (!reference.startsWith("#/$defs/")) {
    return undefined;
  }
  const definition = reference.slice("#/$defs/".length);
  const definitions = asRecord(root.$defs);
  const candidate = definitions?.[definition];
  return isSchemaNode(candidate) ? candidate : undefined;
}

function matchesSupportedPattern(value: string, pattern: string): boolean {
  switch (pattern) {
    case "^[a-z0-9]+(?:[.-][a-z0-9]+)+$":
      return /^[a-z0-9]+(?:[.-][a-z0-9]+)+$/u.test(value);
    default:
      return false;
  }
}

function cloneJson(input: JsonRecord): JsonRecord {
  return JSON.parse(JSON.stringify(input)) as JsonRecord;
}

function asRecord(input: unknown): JsonRecord | undefined {
  return isRecord(input) ? input : undefined;
}

function isRecord(input: unknown): input is JsonRecord {
  return typeof input === "object" && input !== null && !Array.isArray(input);
}

function isSchemaNode(input: unknown): input is SchemaNode {
  return isRecord(input);
}

function childPath(path: string, field: string): string {
  return `${path}.${field}`;
}

function success<T>(value: T): ValidationResult<T> {
  return { ok: true, value };
}

function failure(code: string, path: string): ValidationResult<never> {
  return { ok: false, error: { code, path } };
}
