// SPDX-License-Identifier: MIT
// Independent JavaScript witness for game-information-query-v1.

import {readFileSync} from "node:fs";
import {resolve} from "node:path";

export const PROFILE = "game-information-query-v1";
export const SCHEMA_DIGEST =
  "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9";
const root = resolve(import.meta.dirname, "../..");
const artifact = resolve(root, "artifacts/game-information-query-v1");
const casePath = resolve(root, "conformance/cases/game-information-query-v1.json");
const goldenNames = [
  "capabilities-response",
  "error-stale-cursor",
  "live-detail-request",
  "live-detail-response",
  "static-page-1-request",
  "static-page-1-response",
  "static-page-2-request",
  "static-page-2-response",
];

const ENTITY_KINDS = new Set([
  "card", "character", "enemy", "event", "map_node", "potion", "power", "relic", "room", "status",
]);
const QUERY_KINDS = new Set(["list", "search", "get", "detail", "availability"]);
const PROJECTIONS = new Set(["summary", "standard", "full"]);
const DETAIL_LEVELS = new Set(["summary", "standard", "full"]);
const FIELD_KINDS = new Set([
  "boolean", "definition_ref", "integer", "instance_ref", "text", "text_list",
]);
const AVAILABILITIES = new Set([
  "available", "unavailable", "not_observable", "redacted", "unsupported", "missing",
]);
const FIELD_NAMES = new Set([
  "amount", "cost", "description", "display_name", "flags", "owner", "position", "rarity",
  "source_id", "tags",
]);
const ERROR_CODES = new Set([
  "unknown_kind", "unknown_id", "ambiguous_id", "unsupported_filter", "unsupported_projection",
  "unsupported_version", "denied_scope", "stale_snapshot", "stale_cursor",
  "result_limit_exceeded", "missing_capability", "unsupported_field", "invalid_identity",
  "invalid_bounds", "mixed_generation", "live_fence_mismatch", "availability_mismatch",
  "field_value_mismatch", "page_state_mismatch", "invalid_accounting", "cross_epoch_cursor",
  "duplicate_key", "malformed", "read_only_violation",
]);
const ERR = {
  ambiguousId: "ambiguous_id",
  invalidBounds: "invalid_bounds",
  malformed: "malformed",
  mixedGeneration: "mixed_generation",
  missingCapability: "missing_capability",
  readOnlyViolation: "read_only_violation",
  resultLimitExceeded: "result_limit_exceeded",
  staleCursor: "stale_cursor",
  staleSnapshot: "stale_snapshot",
  unknownKind: "unknown_kind",
  unsupportedField: "unsupported_field",
  unsupportedProjection: "unsupported_projection",
  unsupportedVersion: "unsupported_version",
};

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function parseUniqueJson(text) {
  let offset = 0;

  const fail = (message) => {
    throw new SyntaxError(`${message} at byte ${offset}`);
  };
  const skipWhitespace = () => {
    while ([" ", "\t", "\r", "\n"].includes(text[offset])) offset += 1;
  };
  const parseString = () => {
    const start = offset;
    if (text[offset] !== "\"") fail("expected string");
    offset += 1;
    while (offset < text.length) {
      const character = text[offset];
      offset += 1;
      if (character === "\"") {
        try {
          return JSON.parse(text.slice(start, offset));
        } catch {
          fail("invalid string");
        }
      }
      if (character === "\\") {
        if (offset >= text.length) fail("unterminated escape");
        const escaped = text[offset];
        offset += 1;
        if (escaped === "u") {
          const hex = text.slice(offset, offset + 4);
          if (!/^[0-9a-fA-F]{4}$/.test(hex)) fail("invalid unicode escape");
          offset += 4;
        } else if (!"\"\\/bfnrt".includes(escaped)) {
          fail("invalid escape");
        }
      } else if (character.charCodeAt(0) < 0x20) {
        fail("control character in string");
      }
    }
    fail("unterminated string");
  };
  const parseNumber = () => {
    const match = text.slice(offset).match(
      /^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/,
    );
    if (!match) fail("expected value");
    offset += match[0].length;
  };
  const parseValue = () => {
    skipWhitespace();
    switch (text[offset]) {
      case "\"":
        parseString();
        return;
      case "{":
        parseObject();
        return;
      case "[":
        parseArray();
        return;
      case "t":
        if (!text.startsWith("true", offset)) fail("expected value");
        offset += 4;
        return;
      case "f":
        if (!text.startsWith("false", offset)) fail("expected value");
        offset += 5;
        return;
      case "n":
        if (!text.startsWith("null", offset)) fail("expected value");
        offset += 4;
        return;
      default:
        parseNumber();
    }
  };
  const parseObject = () => {
    offset += 1;
    skipWhitespace();
    const keys = new Set();
    if (text[offset] === "}") {
      offset += 1;
      return;
    }
    while (true) {
      skipWhitespace();
      const key = parseString();
      if (keys.has(key)) fail(`duplicate object member ${key}`);
      keys.add(key);
      skipWhitespace();
      if (text[offset] !== ":") fail("expected object separator");
      offset += 1;
      parseValue();
      skipWhitespace();
      if (text[offset] === "}") {
        offset += 1;
        return;
      }
      if (text[offset] !== ",") fail("expected object member");
      offset += 1;
    }
  };
  const parseArray = () => {
    offset += 1;
    skipWhitespace();
    if (text[offset] === "]") {
      offset += 1;
      return;
    }
    while (true) {
      parseValue();
      skipWhitespace();
      if (text[offset] === "]") {
        offset += 1;
        return;
      }
      if (text[offset] !== ",") fail("expected array member");
      offset += 1;
    }
  };

  parseValue();
  skipWhitespace();
  if (offset !== text.length) fail("trailing input");
  return JSON.parse(text);
}

export function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function byteLength(value) {
  return Buffer.byteLength(canonical(value), "utf8");
}

function getPath(value, path) {
  return path.split("/").filter(Boolean).reduce((cursor, segment) => cursor[segment], value);
}

function applyMutations(value, mutations) {
  const copy = structuredClone(value);
  for (const mutation of mutations) {
    const segments = mutation.path.split("/").filter(Boolean);
    const leaf = segments.pop();
    const parent = segments.reduce((cursor, segment) => cursor[segment], copy);
    parent[leaf] = structuredClone(mutation.value);
  }
  return copy;
}

function assert(condition, message) {
  if (!condition) throw new Error(`game-information: ${message}`);
}

export function binding(query) {
  return {
    query_kind: query.query_kind,
    entity_kind: query.entity_kind,
    target: query.target,
    filters: query.filters,
    projection: query.projection,
    detail_level: query.detail_level,
    binding: query.binding,
    fields: query.fields,
    parent_observation: query.parent_observation,
    limits: query.limits,
  };
}

function readGolden(name) {
  return readJson(resolve(artifact, "golden", `${name}.json`));
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function integer(value) {
  return typeof value === "number" && Number.isInteger(value) && Number.isSafeInteger(value);
}

function equal(left, right) {
  return canonical(left) === canonical(right);
}

function definitionRef(value) {
  return isObject(value) && typeof value.content_manifest_id === "string" &&
    ENTITY_KINDS.has(value.entity_kind) && typeof value.namespaced_id === "string" &&
    (value.variant === null || typeof value.variant === "string");
}

function instanceRef(value) {
  return isObject(value) && typeof value.instance_id === "string" &&
    typeof value.run_id === "string" && integer(value.epoch) && value.epoch >= 0 &&
    ENTITY_KINDS.has(value.entity_kind) && typeof value.entity_id === "string";
}

function boundsError(value) {
  const epoch = value?.epoch;
  return typeof epoch === "number" && Number.isInteger(epoch) &&
    (!Number.isSafeInteger(epoch) || epoch < 0) ? ERR.invalidBounds : null;
}

function fieldError(field) {
  if (!isObject(field) || typeof field.name !== "string" || !FIELD_NAMES.has(field.name) ||
    !FIELD_KINDS.has(field.kind) || !AVAILABILITIES.has(field.availability) ||
    !isObject(field.source) || typeof field.source.kind !== "string" || !("ref" in field.source)) {
    return ERR.malformed;
  }
  if (field.availability !== "available") {
    return field.value === null && typeof field.reason === "string" && field.reason.length > 0
      ? null : ERR.malformed;
  }
  if (field.reason !== null || field.value === null) return ERR.malformed;
  switch (field.kind) {
    case "boolean": return typeof field.value === "boolean" ? null : ERR.malformed;
    case "definition_ref": return definitionRef(field.value) ? null : ERR.malformed;
    case "integer":
      if (!integer(field.value)) return ERR.malformed;
      return field.value >= -2147483648 && field.value <= 2147483647
        ? null : ERR.invalidBounds;
    case "instance_ref": return instanceRef(field.value) ? null : ERR.malformed;
    case "text":
      return typeof field.value === "string" && field.value.length > 0 ? null : ERR.malformed;
    case "text_list":
      return Array.isArray(field.value) && field.value.length <= 64 &&
        field.value.every((value) => typeof value === "string" && value.length > 0)
        ? null : ERR.malformed;
    default: return ERR.malformed;
  }
}

function textBytes(items) {
  let total = 0;
  for (const item of items) {
    for (const field of item.fields ?? []) {
      if (field.availability !== "available") continue;
      if (field.kind === "text" && typeof field.value === "string") {
        total += Buffer.byteLength(field.value, "utf8");
      } else if (field.kind === "text_list" && Array.isArray(field.value)) {
        for (const value of field.value) {
          if (typeof value === "string") total += Buffer.byteLength(value, "utf8");
        }
      }
    }
  }
  return total;
}

function validLimits(limits) {
  return isObject(limits) &&
    ["page_items", "item_bytes", "page_bytes", "text_bytes"].every((key) =>
      integer(limits[key]) && limits[key] > 0);
}

function limitError(limits, capabilities) {
  if (!validLimits(limits)) return ERR.malformed;
  for (const key of ["page_items", "item_bytes", "page_bytes", "text_bytes"]) {
    if (limits[key] > capabilities.limits[key]) return ERR.resultLimitExceeded;
  }
  return null;
}

function queryError(query, context) {
  if (!isObject(query)) return ERR.malformed;
  if (!QUERY_KINDS.has(query.query_kind) || !ENTITY_KINDS.has(query.entity_kind)) {
    return ERR.unknownKind;
  }
  if (!PROJECTIONS.has(query.projection) ||
    !context.capabilities.projections.includes(query.projection) ||
    !DETAIL_LEVELS.has(query.detail_level) ||
    !context.capabilities.detail_levels.includes(query.detail_level)) {
    return ERR.unsupportedProjection;
  }
  if (!Array.isArray(query.fields) || new Set(query.fields).size !== query.fields.length) {
    return ERR.malformed;
  }
  for (const field of query.fields) {
    if (typeof field !== "string" || !FIELD_NAMES.has(field) ||
      !context.capabilities.fields.includes(field)) return ERR.unsupportedField;
  }
  const mode = query.binding?.mode;
  if (!isObject(query.binding) || !["static", "live"].includes(mode)) return ERR.malformed;
  if (mode === "live" && context.capabilities.snapshot_policy.supports_live !== true) {
    return ERR.missingCapability;
  }
  if (mode === "static") {
    if (query.binding.instance_ref !== null || query.binding.snapshot_ref !== null ||
      query.parent_observation !== null || query.target?.instance_ref !== null) return ERR.malformed;
  } else if (boundsError(query.binding.instance_ref) || boundsError(query.binding.snapshot_ref?.instance_ref) ||
    boundsError(query.target?.instance_ref) ||
    (typeof query.binding.snapshot_ref?.state_generation === "number" &&
      Number.isInteger(query.binding.snapshot_ref.state_generation) &&
      (!Number.isSafeInteger(query.binding.snapshot_ref.state_generation) ||
        query.binding.snapshot_ref.state_generation < 0))) {
    return ERR.invalidBounds;
  } else if (!instanceRef(query.binding.instance_ref) ||
    !isObject(query.binding.snapshot_ref) ||
    !instanceRef(query.binding.snapshot_ref.instance_ref) ||
    !integer(query.binding.snapshot_ref.state_generation) ||
    !isObject(query.parent_observation) || !instanceRef(query.target?.instance_ref)) {
    return ERR.malformed;
  }
  const limitsError = limitError(query.limits, context.capabilities);
  if (limitsError) return limitsError;
  if (query.query_kind === "search" && query.filters?.display_name !== null &&
    query.filters?.namespaced_ids?.length === 0 &&
    query.filters?.definition_refs?.length === 0 &&
    query.filters?.instance_ids?.length === 0) return ERR.ambiguousId;
  if (query.cursor !== null) {
    if (typeof query.cursor !== "string") return ERR.malformed;
    const expected = context.cursorBindings.get(query.cursor);
    if (!expected || !equal(expected, binding(query))) return ERR.staleCursor;
  }
  if (mode === "live") {
    const fencesMatch = equal(query.binding.instance_ref, query.binding.snapshot_ref.instance_ref) &&
      equal(query.binding.instance_ref, query.target.instance_ref) &&
      equal(query.parent_observation.instance_ref, query.binding.instance_ref) &&
      equal(query.parent_observation.snapshot_ref, query.binding.snapshot_ref) &&
      query.parent_observation.state_generation === query.binding.snapshot_ref.state_generation;
    if (!fencesMatch) return query.cursor !== null ? ERR.staleCursor : ERR.staleSnapshot;
  }
  return null;
}

function pageError(page, query, context) {
  if (!isObject(page) || !Array.isArray(page.items) || !validLimits(page.limits) ||
    !isObject(page.accounting)) return ERR.malformed;
  const pageLimitError = limitError(page.limits, context.capabilities);
  if (pageLimitError) return pageLimitError;
  const maxItemBytes = Math.max(0, ...page.items.map(byteLength));
  const payloadBytes = byteLength(page.items);
  const text = textBytes(page.items);
  const accountingPage = Object.fromEntries(
    Object.entries(page).filter(([key]) => key !== "accounting"),
  );
  const pageBytes = byteLength(accountingPage);
  if (page.items.length > page.limits.page_items || maxItemBytes > page.limits.item_bytes ||
    pageBytes > page.limits.page_bytes || text > page.limits.text_bytes) {
    return ERR.resultLimitExceeded;
  }
  if (!equal(page.limits, query.limits)) return ERR.malformed;
  if (page.accounting.item_count !== page.items.length ||
    page.accounting.item_bytes !== maxItemBytes ||
    page.accounting.payload_bytes !== payloadBytes ||
    page.accounting.page_bytes !== pageBytes ||
    page.accounting.text_bytes !== text) return ERR.malformed;
  if (page.final_page === true) {
    if (page.next_cursor !== null) return ERR.malformed;
  } else if (page.final_page === false) {
    if (page.next_cursor === null || page.cursor_binding === null) return ERR.malformed;
  } else return ERR.malformed;
  if (page.total_count_known === true) {
    if (!integer(page.total_count) || page.total_count < 0) return ERR.malformed;
  } else if (page.total_count_known === false) {
    if (page.total_count !== null) return ERR.malformed;
  } else return ERR.malformed;
  if (["unavailable", "not_observable"].includes(page.coverage) &&
    (page.items.length !== 0 || page.next_cursor !== null || page.cursor_binding !== null ||
      page.final_page !== true || page.total_count_known !== false || page.total_count !== null ||
      page.accounting.item_count !== 0 || page.accounting.item_bytes !== 0 ||
      page.accounting.payload_bytes !== 0 || page.accounting.text_bytes !== 0)) return ERR.malformed;
  if (page.cursor_binding !== null && !equal(page.cursor_binding, binding(query))) {
    return ERR.staleCursor;
  }
  if (page.next_cursor !== null && !context.cursorBindings.has(page.next_cursor)) {
    return ERR.staleCursor;
  }
  for (const item of page.items) {
    if (!isObject(item) || !definitionRef(item.definition_ref) ||
      item.definition_ref.content_manifest_id !== query.binding.content_manifest_id) return ERR.malformed;
    if (query.binding.mode === "static") {
      if (item.instance_ref !== null) return ERR.malformed;
    } else if (!equal(item.instance_ref, query.binding.instance_ref)) return ERR.malformed;
    if (!Array.isArray(item.fields)) return ERR.malformed;
    let previous = "";
    for (const field of item.fields) {
      if (typeof field.name !== "string" || field.name <= previous) return ERR.malformed;
      previous = field.name;
      const error = fieldError(field);
      if (error) return error;
    }
  }
  return null;
}

function responseError(value, context) {
  const queryErrorValue = queryError(value.query, context);
  if (queryErrorValue) return queryErrorValue;
  if (!isObject(value.result)) return ERR.malformed;
  if (value.result.read_only !== true) {
    return value.result.read_only === false ? ERR.readOnlyViolation : ERR.malformed;
  }
  const live = value.query.binding.mode === "live";
  if (live) {
    if (!integer(value.result.result_generation) || !isObject(value.result.parent_observation)) {
      return ERR.malformed;
    }
    if (value.result.result_generation !== value.query.binding.snapshot_ref.state_generation) {
      return ERR.mixedGeneration;
    }
    if (!equal(value.result.parent_observation, value.query.parent_observation)) {
      return ERR.mixedGeneration;
    }
  } else if (value.result.result_generation !== null || value.result.parent_observation !== null) {
    return ERR.malformed;
  }
  return pageError(value.result.page, value.query, context);
}

export function contextFor(name = "static") {
  const capabilities = structuredClone(readGolden("capabilities-response").capabilities);
  const staticResponse = readGolden("static-page-1-response");
  const liveRequest = readGolden("live-detail-request");
  const context = {
    capabilities,
    cursorBindings: new Map([
      [staticResponse.result.page.next_cursor, binding(staticResponse.query)],
      ["cursor:live:1", binding(liveRequest.query)],
    ]),
  };
  if (name === "no-live") capabilities.snapshot_policy.supports_live = false;
  if (name === "summary-only") capabilities.projections = ["summary"];
  if (name === "no-tags") capabilities.fields = capabilities.fields.filter((field) => field !== "tags");
  if (name === "small-message") capabilities.max_message_bytes = 2560;
  return context;
}

function normalizeContext(context) {
  if (typeof context === "string" || context === undefined) return contextFor(context);
  return context;
}

export function semanticRejection(value, context = "static") {
  const normalized = normalizeContext(context);
  if (!isObject(value)) return ERR.malformed;
  if (value.protocol_version !== PROFILE) return ERR.unsupportedVersion;
  if (value.schema_digest !== SCHEMA_DIGEST) return ERR.malformed;
  if (byteLength(value) > normalized.capabilities.max_message_bytes) {
    return ERR.resultLimitExceeded;
  }
  switch (value.kind) {
    case "capabilities_response":
      return isObject(value.capabilities) && value.capabilities.profile === PROFILE ? null : ERR.malformed;
    case "error_response": {
      const code = value.error?.code;
      const derived = value.query === null ? null : queryError(value.query, normalized);
      if (derived === ERR.missingCapability || derived === ERR.unsupportedField) {
        return derived === code ? code : ERR.malformed;
      }
      if (code === ERR.missingCapability || code === ERR.unsupportedField) return ERR.malformed;
      return typeof code === "string" && ERROR_CODES.has(code) ? code : ERR.malformed;
    }
    case "query_request": return queryError(value.query, normalized);
    case "query_response": return responseError(value, normalized);
    default: return ERR.unknownKind;
  }
}

function validateGolden(name) {
  const path = resolve(artifact, "golden", `${name}.json`);
  const text = readFileSync(path, "utf8").trimEnd();
  const value = JSON.parse(text);
  assert(canonical(value) === text, `${name} is not canonical compact JSON`);
  assert(value.protocol_version === PROFILE, `${name} profile`);
  assert(value.schema_digest === SCHEMA_DIGEST, `${name} digest`);
  assert(value.provenance?.artifact === `sts2-protocol/${PROFILE}`, `${name} provenance`);
  const semantic = semanticRejection(value, value.query?.binding?.mode ?? "static");
  assert(value.kind === "error_response" ? semantic === value.error.code : semantic === null,
    `${name} semantic rejection: ${semantic}`);
  if (value.kind === "query_response") {
    const page = value.result.page;
    assert(page.accounting.item_count === page.items.length, `${name} item accounting`);
    assert(page.accounting.payload_bytes === byteLength(page.items), `${name} payload accounting`);
    assert(page.accounting.text_bytes === textBytes(page.items), `${name} text accounting`);
    const pageWithoutAccounting = Object.fromEntries(
      Object.entries(page).filter(([key]) => key !== "accounting"),
    );
    assert(page.accounting.page_bytes === byteLength(pageWithoutAccounting),
      `${name} page accounting`);
    assert(page.accounting.page_bytes <= page.limits.page_bytes, `${name} page limit`);
  }
  return value;
}

function validateFieldVectors(caseValue) {
  for (const vector of caseValue.valid_vectors) {
    if (!vector.fixture) continue;
    const fixture = readJson(resolve(root, vector.fixture));
    assert(fixture.id === vector.id, `${vector.id} fixture ID`);
    if (vector.id === "GIQ-VALID-UTF8-BOUNDS") {
      assert(Buffer.byteLength(fixture.text, "utf8") === fixture.utf8_bytes, "UTF-8 accounting");
      assert(fixture.utf8_bytes <= fixture.maximum_utf8_bytes, "UTF-8 bound");
    }
    if (vector.id === "GIQ-VALID-BYTE-BOUNDARY") {
      const golden = readGolden("static-page-1-response");
      assert(byteLength(golden) === fixture.message_bytes, "message byte accounting");
      assert(byteLength(golden.result.page.items) === fixture.page_bytes, "page byte accounting");
      assert(Math.max(...golden.result.page.items.map(byteLength)) === fixture.item_bytes,
        "item byte accounting");
      assert(textBytes(golden.result.page.items) === fixture.text_bytes, "text byte accounting");
    }
    if (vector.id === "GIQ-VALID-SOURCE-PROVENANCE") {
      assert(fixture.fields.every((field) => field.source?.kind && "ref" in field.source),
        "source provenance");
    }
    if (vector.id === "GIQ-VALID-EMPTY-VS-UNAVAILABLE") {
      assert(fixture.fields[0].availability === "available" && Array.isArray(fixture.fields[0].value),
        "empty value");
      assert(fixture.fields[1].availability === "unavailable" && fixture.fields[1].value === null,
        "unavailable value");
    }
    if (vector.id === "GIQ-VALID-ZERO-VS-MISSING") {
      assert(fixture.fields[0].value === 0 && fixture.fields[0].availability === "available",
        "zero value");
      assert(fixture.fields[1].availability === "missing" && fixture.fields[1].value === null,
        "missing value");
    }
  }
}

function validateInvalidVectors(caseValue) {
  for (const descriptor of caseValue.invalid_vectors) {
    const fixture = readJson(resolve(root, descriptor.fixture));
    assert(fixture.id === descriptor.id, `${descriptor.id} fixture ID`);
    assert(fixture.expected_error === descriptor.expected_error, `${descriptor.id} error`);
    if (fixture.raw) {
      let rejected = false;
      try {
        parseUniqueJson(fixture.raw);
      } catch {
        rejected = true;
      }
      assert(rejected, `${descriptor.id} duplicate-key witness was accepted`);
      assert(fixture.expected_error === ERR.malformed, `${descriptor.id} raw rejection`);
      continue;
    }
    const base = readJson(resolve(root, fixture.base_fixture));
    const mutated = applyMutations(base, fixture.mutations);
    const context = fixture.context ?? descriptor.context ?? "static";
    assert(semanticRejection(mutated, context) === fixture.expected_error,
      `${descriptor.id} semantic rejection`);
    if ([ERR.missingCapability, ERR.unsupportedField].includes(fixture.expected_error)) {
      const mismatchedCode = structuredClone(mutated);
      mismatchedCode.error.code = ERR.staleCursor;
      assert(semanticRejection(mismatchedCode, context) === ERR.malformed,
        `${descriptor.id} error code must match request context`);
    }
    if (descriptor.id.includes("CROSS-")) {
      assert(mutated.query.cursor !== null, `${descriptor.id} cursor`);
    }
    if (descriptor.id === "GIQ-INVALID-MIXED-GENERATION") {
      assert(mutated.result.result_generation !==
        mutated.query.binding.snapshot_ref.state_generation, `${descriptor.id} generation`);
    }
    if (descriptor.id.includes("INTEGER-BOUNDS")) {
      assert(Math.abs(getPath(mutated, fixture.mutations[0].path)) > 2147483647,
        `${descriptor.id} bounds`);
    }
  }
}

export function validate() {
  const source = readFileSync(resolve(root, "schemas/game-information-query-v1.schema.json"), "utf8");
  const artifactSchema = readFileSync(resolve(artifact, "schema.json"), "utf8");
  assert(source === artifactSchema, "schema/artifact bytes");
  const caseValue = readJson(casePath);
  assert(caseValue.profile === PROFILE, "case profile");
  const goldens = new Map(goldenNames.map((name) => [name, validateGolden(name)]));
  const firstResponse = goldens.get("static-page-1-response");
  const firstPage = firstResponse.result.page;
  assert(firstPage.final_page === false, "first page");
  assert(firstPage.accounting.page_bytes > firstPage.accounting.payload_bytes,
    "full-page accounting includes page metadata");
  const constrained = structuredClone(firstResponse);
  constrained.query.limits.page_bytes = 800;
  constrained.result.page.limits.page_bytes = 800;
  assert(constrained.result.page.accounting.payload_bytes <= 800,
    "payload remains within constrained page limit");
  assert(constrained.result.page.accounting.page_bytes > 800,
    "full page exceeds constrained page limit");
  assert(semanticRejection(constrained, "static") === ERR.resultLimitExceeded,
    "full-page byte limit is enforced");
  assert(goldens.get("static-page-2-response").result.page.final_page === true, "final page");
  const detailRequest = goldens.get("live-detail-request");
  const detailResponse = goldens.get("live-detail-response");
  assert(detailRequest.query.parent_observation &&
    canonical(detailRequest.query.parent_observation) ===
    canonical(detailResponse.result.parent_observation), "live parent observation");
  assert(detailResponse.result.result_generation ===
    detailRequest.query.binding.snapshot_ref.state_generation, "live generation fence");
  validateFieldVectors(caseValue);
  validateInvalidVectors(caseValue);
  return {profile: PROFILE, schemaDigest: SCHEMA_DIGEST, goldens: goldens.size,
    invalidVectors: caseValue.invalid_vectors.length};
}

if (process.argv[1] === import.meta.filename) {
  console.log(JSON.stringify(validate()));
}
