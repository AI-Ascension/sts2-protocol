import test from "node:test";
import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {readFileSync} from "node:fs";

const root = new URL("../..", import.meta.url);
const read = (path) => readFileSync(new URL(path, root));
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("source schema, release-like artifact, manifest, and checksum inventory agree", () => {
  const source = read("schemas/game-information-content-manifest-v1.schema.json");
  const schema = read("artifacts/game-information-content-manifest-v1/schema.json");
  const manifestBytes = read("artifacts/game-information-content-manifest-v1/manifest.json");
  const checksums = read("artifacts/game-information-content-manifest-v1/SHA256SUMS").toString("utf8");
  const manifest = JSON.parse(manifestBytes);

  assert.deepEqual(schema, source);
  assert.equal(manifest.protocol_version, "game-information-content-manifest-v1");
  assert.equal(manifest.schema_digest, digest(schema));
  assert.equal(manifest.max_message_bytes, 16 * 1024 * 1024);

  const inventory = new Map(checksums.trim().split("\n").map((line) => {
    const [hash, path] = line.split(/\s{2,}/);
    return [path, hash];
  }));
  assert.equal(inventory.size, 5);
  for (const path of [
    "schema.json",
    "manifest.json",
    "README.md",
    "golden/canonical-manifest-response.json",
    "golden/access-denied-error-response.json",
  ]) {
    assert.equal(inventory.get(path), digest(read(`artifacts/game-information-content-manifest-v1/${path}`)));
  }
});

test("valid fixtures are byte-identical to the checked-in goldens", () => {
  for (const name of [
    "canonical-manifest-response.json",
    "access-denied-error-response.json",
  ]) {
    assert.deepEqual(
      read(`conformance/fixtures/game-information-content-manifest-v1/valid/${name}`),
      read(`artifacts/game-information-content-manifest-v1/golden/${name}`),
    );
  }
});
