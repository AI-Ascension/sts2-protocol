// SPDX-License-Identifier: MIT
import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {blobDigest, canonicalHex, canonicalize, checkpointId, parseProfile, stateId} from "./canonical.mjs";

const fixture = (name) => JSON.parse(readFileSync(new URL(`../../conformance/fixtures/exact-state-v1/${name}`, import.meta.url), "utf8"));
const vectors = fixture("canonical-vectors.json");
const expected = new Map(fixture("canonical-vectors.expected.json").vectors.map((entry) => [entry.name, entry]));
const identity = new Map(fixture("identity-vectors.expected.json").vectors.map((entry) => [entry.name, entry]));
const values = new Map(vectors.positive.map((entry) => [entry.name, entry.value]));
const text = (name) => JSON.stringify(values.get(name));

test("positive vectors match recorded bytes and state digests", () => {
  assert.equal(vectors.positive.length, 26);
  for (const entry of vectors.positive) {
    const recorded = expected.get(entry.name);
    assert.equal(canonicalHex(text(entry.name)), recorded.canonical_hex, `canonical bytes differ for ${entry.name}`);
    assert.equal(stateId(text(entry.name)), recorded.state_id, `state digest differs for ${entry.name}`);
  }
});

test("checkpoint and blob identities match the independent witness", () => {
  for (const entry of vectors.positive) {
    const recorded = identity.get(entry.name);
    assert.equal(checkpointId(text(entry.name)), recorded.checkpoint_id, `checkpoint id differs for ${entry.name}`);
    assert.equal(blobDigest(Buffer.from(canonicalize(text(entry.name)), "utf8")), recorded.blob_digest, `blob digest differs for ${entry.name}`);
  }
});

test("equivalence pairs agree and distinction pairs differ", () => {
  for (const pair of vectors.equivalence_pairs) {
    assert.equal(canonicalHex(text(pair.left)), canonicalHex(text(pair.right)));
    assert.equal(stateId(text(pair.left)), stateId(text(pair.right)));
  }
  for (const pair of vectors.distinct_pairs) {
    assert.notEqual(stateId(text(pair.left)), stateId(text(pair.right)));
  }
});

test("raw rejection vectors are refused", () => {
  assert.equal(vectors.reject_raw.length, 14);
  for (const entry of vectors.reject_raw) {
    assert.throws(() => parseProfile(entry.text), `raw rejection ${entry.name} was accepted`);
  }
});

test("malformed and out-of-profile input is refused", () => {
  for (const bad of ['{"a":1,"\\u0061":2}', '{"A":1}', '{"":1}', '{"k":"\\ud800"}', '{} trailing', '{"k":-0}', '{"k":1e3}', '{"k":9007199254740992}']) {
    assert.throws(() => parseProfile(bad), `input ${bad} was accepted`);
  }
  assert.throws(() => parseProfile("\ufeff{}"));
  const within = "[".repeat(64) + "0" + "]".repeat(64);
  assert.ok(parseProfile(within));
  const beyond = "[".repeat(65) + "0" + "]".repeat(65);
  assert.throws(() => parseProfile(beyond));
});

test("canonical ordering and whitespace removal are stable", () => {
  assert.equal(canonicalHex('{ "b" : 1 , "a" : 2 }'), "7b2261223a322c2262223a317d");
  assert.equal(canonicalHex('{"k":"a\\u2028\\u00e9\\ud83d\\ude00"}'), "7b226b223a2261e280a8c3a9f09f9880227d");
  assert.equal(canonicalHex('{"k":"\\u0001\\b\\t\\n\\f\\r\\"\\\\"}'), "7b226b223a225c75303030315c625c745c6e5c665c725c225c5c227d");
});
