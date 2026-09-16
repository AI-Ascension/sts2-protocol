// SPDX-License-Identifier: MIT
// Independent JavaScript witness for game-information-lookup-binding-v1.

import {createHash} from "node:crypto";
import {readFileSync} from "node:fs";
import {resolve} from "node:path";

export const PROFILE = "game-information-lookup-binding-v1";
export const SCHEMA_DIGEST =
  "f10f9af01d6be1de104069ba842e7971971e88f27553e782e81174ee7aa1cd58";
const root = resolve(import.meta.dirname, "../..");
const artifact = resolve(root, "artifacts/game-information-lookup-binding-v1");
const casePath = resolve(root, "conformance/cases/game-information-lookup-binding-v1.json");
const identityFixtures = [
  "conformance/fixtures/game-information-lookup-binding-v1/valid/binding-identity-epoch.json",
  "conformance/fixtures/game-information-lookup-binding-v1/valid/binding-identity-input.json",
];
const goldenNames = [
  "discovery-response",
  "observation-response",
  "reobserve-exhausted-response",
  "reobserve-required-response",
  "reobserved-response",
];

const KINDS = new Set([
  "lookup_binding_discovery_response",
  "lookup_binding_observation_response",
  "error_response",
]);
const STATES = new Set([
  "not_yet_observed",
  "observed",
  "reobserve_required",
  "reobserve_exhausted",
]);
const ERROR_CODES = new Set([
  "denied_scope",
  "invalid_identity",
  "malformed",
  "missing_capability",
  "mixed_binding",
  "reobserve_unavailable",
  "stale_snapshot",
  "unsupported_version",
]);
const ERR = {
  deniedScope: "denied_scope",
  invalidIdentity: "invalid_identity",
  malformed: "malformed",
  missingCapability: "missing_capability",
  mixedBinding: "mixed_binding",
  reobserveUnavailable: "reobserve_unavailable",
  staleSnapshot: "stale_snapshot",
  unsupportedVersion: "unsupported_version",
};

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isNullish(value) {
  return value === null || value === undefined;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function digestOf(text) {
  return createHash("sha256").update(text, "utf8").digest("hex");
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

export function bindingId(binding) {
  return digestOf(canonical({
    agent_id: binding.scope.agent_id ?? null,
    authority_epoch: binding.authority_epoch ?? null,
    content_manifest_id: binding.content_manifest_id ?? null,
    episode_id: binding.scope.episode_id ?? null,
    game_profile: binding.game_profile ?? null,
    locale: binding.locale ?? null,
    project_id: binding.scope.project_id ?? null,
    run_id: binding.scope.run_id ?? null,
  }));
}

function behind(observed, retained) {
  if (!Number.isInteger(observed) || !Number.isInteger(retained)) return false;
  return observed < retained;
}

export function semanticRejection(document, context) {
  if (!KINDS.has(document?.kind)) return ERR.malformed;
  if (document.protocol_version !== PROFILE || document.schema_digest !== SCHEMA_DIGEST) {
    return ERR.unsupportedVersion;
  }
  if (isObject(document.binding)) {
    if (!isObject(document.binding.scope)) return ERR.malformed;
    if (bindingId(document.binding) !== document.binding.binding_id) return ERR.invalidIdentity;
    if (canonical(document.binding.scope) !== canonical(context.scope)) return ERR.deniedScope;
    if (document.binding.instance_id !== context.instance_id) return ERR.deniedScope;
  }
  if (isObject(document.discovery)) {
    const required = document.discovery.required_capabilities?.profile;
    const negotiated = context.negotiated_capabilities;
    if (!Array.isArray(negotiated) || !negotiated.includes(required)) {
      return ERR.missingCapability;
    }
  }
  if (!isNullish(document.observation)) {
    if (document.observation.binding_id !== document.binding?.binding_id) {
      return ERR.mixedBinding;
    }
    if (behind(document.observation.state_generation, context.retained_state_generation)) {
      return ERR.staleSnapshot;
    }
  }
  return stateRejection(document, context);
}

function stateRejection(document, context) {
  if (document.kind === "lookup_binding_discovery_response") {
    if (isNullish(document.binding)
      || !isObject(document.discovery)
      || !isNullish(document.observation)
      || !isNullish(document.error)
      || document.discovery.observation_state !== "not_yet_observed"
      || !isNullish(document.discovery.reobserve)) {
      return ERR.malformed;
    }
    return null;
  }
  return observationShape(document, context);
}

function observationShape(document, context) {
  if (document.kind === "error_response") return errorShape(document);
  const discovery = document.discovery;
  const state = discovery?.observation_state;
  if (typeof state !== "string" || !STATES.has(state)) return ERR.malformed;
  if (isNullish(document.binding) || !isObject(discovery) || !isNullish(document.error)) {
    return ERR.malformed;
  }
  const observation = document.observation ?? null;
  const reobserve = discovery.reobserve ?? null;
  if (state === "observed") {
    if (isNullish(observation) || !isNullish(reobserve)) return ERR.malformed;
    return null;
  }
  if (state !== "reobserve_required" && state !== "reobserve_exhausted") return ERR.malformed;
  if (!isNullish(observation) || !isObject(reobserve)) return ERR.malformed;
  const attempts = Number.isInteger(reobserve.attempts) ? reobserve.attempts : 0;
  if ((state === "reobserve_exhausted" && attempts < 1)
    || (!isNullish(context.retained_observation_id)
      && reobserve.supersedes_observation_id !== context.retained_observation_id)) {
    return ERR.malformed;
  }
  return null;
}

function errorShape(document) {
  const code = document.error?.code;
  if (typeof code !== "string" || !ERROR_CODES.has(code) || !isNullish(document.observation)) {
    return ERR.malformed;
  }
  if (isNullish(document.binding) !== isNullish(document.discovery)) return ERR.malformed;
  if (code === ERR.reobserveUnavailable
    && document.discovery?.observation_state !== "reobserve_exhausted") {
    return ERR.malformed;
  }
  return null;
}

function validateGolden(relative, context, id) {
  const name = relative.split("/").pop().replace(/\.json$/, "");
  assert(goldenNames.includes(name), `${id}: golden is registered for ${name}`);
  const origin = readFileSync(resolve(root, relative), "utf8");
  const value = JSON.parse(origin);
  assert(canonical(value) === origin, `${id}: canonical golden bytes`);
  assert(value.protocol_version === PROFILE, `${id}: profile`);
  assert(value.schema_digest === SCHEMA_DIGEST, `${id}: schema digest`);
  assert(
    value.provenance?.artifact === "sts2-protocol/game-information-lookup-binding-v1",
    `${id}: provenance`,
  );
  assert(value.binding.binding_id === bindingId(value.binding), `${id}: binding identity`);
  assert(semanticRejection(value, context) === null, `${id}: accepted by the state machine`);
  return value;
}

function validateIdentity(relative, id) {
  assert(identityFixtures.includes(relative), `${id}: known identity fixture`);
  const fixture = readJson(resolve(root, relative));
  assert(canonical(fixture.identity_input) === fixture.canonical_json, `${id}: canonical bytes`);
  assert(digestOf(fixture.canonical_json) === fixture.binding_id, `${id}: identity digest`);
  return fixture;
}

function validateValidVectors(caseDocument) {
  const vectors = caseDocument.valid_vectors;
  assert(vectors.length === 7, `valid vectors: ${vectors.length}`);
  const identities = new Set();
  let goldens = 0;
  for (const vector of vectors) {
    if (typeof vector.response === "string") {
      goldens += 1;
      validateGolden(vector.response, caseDocument.context, vector.id);
    } else {
      identities.add(validateIdentity(vector.fixture, vector.id).binding_id);
    }
  }
  assert(goldens === goldenNames.length, `goldens: ${goldens}`);
  assert(identities.size === identityFixtures.length, `distinct identities: ${identities.size}`);
  return goldens;
}

function validateInvalidVectors(caseDocument) {
  const vectors = caseDocument.invalid_vectors;
  assert(vectors.length === 12, `invalid vectors: ${vectors.length}`);
  for (const vector of vectors) {
    const fixture = readJson(resolve(root, vector.fixture));
    assert(fixture.id === vector.id, `${vector.id}: fixture ID`);
    assert(fixture.expected_error === vector.expected_error, `${vector.id}: expectation`);
    assert(ERROR_CODES.has(vector.expected_error), `${vector.id}: typed error vocabulary`);
    if (typeof fixture.raw === "string") {
      assert(fixture.schema_valid === false, `${vector.id}: raw input is not schema valid`);
      assert(fixture.expected_error === ERR.malformed, `${vector.id}: raw input is malformed`);
      let rejected = false;
      try {
        parseUniqueJson(fixture.raw);
      } catch {
        rejected = true;
      }
      assert(rejected, `${vector.id}: duplicate object member is rejected`);
      continue;
    }
    const context = fixture.context ?? caseDocument.context;
    assert(
      semanticRejection(fixture.document, context) === fixture.expected_error,
      `${vector.id}: typed rejection ${fixture.expected_error}`,
    );
  }
  return vectors.length;
}

export function validate() {
  const caseDocument = readJson(casePath);
  assert(caseDocument.profile === PROFILE, "case profile");
  return {
    profile: PROFILE,
    schemaDigest: SCHEMA_DIGEST,
    goldens: validateValidVectors(caseDocument),
    invalidVectors: validateInvalidVectors(caseDocument),
  };
}

if (process.argv[1] === import.meta.filename) {
  console.log(JSON.stringify(validate()));
}
