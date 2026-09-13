// SPDX-License-Identifier: MIT
// Independent JavaScript witness for the game-information-query-v1 vectors.
// It validates the frozen wire bytes and semantic fences without importing Rust code.

import {readFileSync} from "node:fs";
import {resolve} from "node:path";

export const PROFILE = "game-information-query-v1";
export const SCHEMA_DIGEST =
  "e5ba81b0520687cf59db6a94aea3b38606e86300f6eb2b0e858f55704e62f76c";

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

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
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
    parent[leaf] = mutation.value;
  }
  return copy;
}

function assert(condition, message) {
  if (!condition) throw new Error(`game-information: ${message}`);
}

function binding(query) {
  return {
    query_kind: query.query_kind,
    entity_kind: query.entity_kind,
    target: query.target,
    filters: query.filters,
    projection: query.projection,
    detail_level: query.detail_level,
    binding: query.binding,
    parent_observation: query.parent_observation,
    limits: query.limits,
  };
}

function validateGolden(name) {
  const path = resolve(artifact, "golden", `${name}.json`);
  const text = readFileSync(path, "utf8").trimEnd();
  const value = JSON.parse(text);
  assert(canonical(value) === text, `${name} is not canonical compact JSON`);
  assert(value.protocol_version === PROFILE, `${name} profile`);
  assert(value.schema_digest === SCHEMA_DIGEST, `${name} digest`);
  assert(value.provenance?.artifact === `sts2-protocol/${PROFILE}`, `${name} provenance`);
  if (value.kind === "query_response") {
    assert(value.result.read_only === true, `${name} must remain read-only`);
    const page = value.result.page;
    assert(page.accounting.item_count === page.items.length, `${name} item accounting`);
    assert(page.accounting.payload_bytes <= page.limits.page_bytes, `${name} byte bound`);
    assert(page.accounting.text_bytes <= page.limits.text_bytes, `${name} text bound`);
    if (page.cursor_binding) {
      assert(canonical(page.cursor_binding) === canonical(binding(value.query)),
        `${name} cursor binding`);
    }
    for (const item of page.items) {
      assert(item.definition_ref.content_manifest_id === value.query.binding.content_manifest_id,
        `${name} definition/content binding`);
      let previous = "";
      for (const field of item.fields) {
        assert(field.name > previous, `${name} fields are deterministic and unique`);
        previous = field.name;
        if (field.availability === "available") {
          assert(field.value !== null, `${name} available field cannot be null`);
        } else {
          assert(field.value === null, `${name} unavailable field must be null`);
        }
      }
    }
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
    if (vector.id === "GIQ-VALID-SOURCE-PROVENANCE") {
      assert(fixture.fields.every((field) => field.source?.kind && "ref" in field.source),
        "source provenance");
    }
    if (vector.id === "GIQ-VALID-EMPTY-VS-UNAVAILABLE") {
      assert(fixture.fields[0].availability === "available" &&
        Array.isArray(fixture.fields[0].value), "empty value");
      assert(fixture.fields[1].availability === "unavailable" &&
        fixture.fields[1].value === null, "unavailable value");
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
      assert((fixture.raw.match(/"protocol_version"/g) ?? []).length > 1,
        `${descriptor.id} duplicate-key witness`);
      continue;
    }
    const base = readJson(resolve(root, fixture.base_fixture));
    const mutated = applyMutations(base, fixture.mutations);
    const path = fixture.mutations[0].path;
    if (descriptor.id.includes("CROSS-")) {
      assert(mutated.query.cursor !== null, `${descriptor.id} cursor`);
    }
    if (descriptor.id === "GIQ-INVALID-MIXED-GENERATION") {
      assert(mutated.result.result_generation !==
        mutated.query.binding.snapshot_ref.state_generation, `${descriptor.id} generation`);
    }
    if (descriptor.id === "GIQ-INVALID-MISSING-VS-EMPTY") {
      const field = mutated.result.page.items[0].fields[0];
      assert(field.availability !== "available" && field.value !== null, `${descriptor.id} null`);
    }
    if (descriptor.id === "GIQ-INVALID-UNKNOWN-ENUM-VERSION") {
      assert(mutated.protocol_version !== PROFILE && mutated.query.entity_kind === "card_instance",
        `${descriptor.id} enum`);
    }
    if (descriptor.id === "GIQ-INVALID-INTEGER-BOUNDS") {
      assert(getPath(mutated, path) > Number.MAX_SAFE_INTEGER, `${descriptor.id} bounds`);
    }
    if (descriptor.id === "GIQ-INVALID-OVERSIZED") {
      assert(mutated.query.limits.page_bytes > 262144, `${descriptor.id} size`);
    }
    if (descriptor.id === "GIQ-INVALID-READ-ONLY-VIOLATION") {
      assert(mutated.result.read_only === false, `${descriptor.id} mutation`);
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
  assert(goldens.get("static-page-1-response").result.page.final_page === false, "first page");
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
