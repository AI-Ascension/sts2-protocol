import test from "node:test";
import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {readFileSync} from "node:fs";

const root = new URL("../..", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");
const digest = (text) => createHash("sha256").update(text).digest("hex");

test("content manifest artifact pins schema and whole-payload limit", () => {
  const manifest = JSON.parse(read("artifacts/game-information-content-manifest-v1/manifest.json"));
  const schema = read("artifacts/game-information-content-manifest-v1/schema.json");
  assert.equal(manifest.protocol_version, "game-information-content-manifest-v1");
  assert.equal(manifest.schema_digest, digest(schema));
  assert.equal(manifest.max_message_bytes, 16 * 1024 * 1024);
  const shape = JSON.parse(schema);
  assert.equal(shape.$defs.manifest.additionalProperties, false);
  assert.equal(shape.$defs.definition.additionalProperties, false);
  assert.equal(shape.$defs.manifest.properties.definitions.maxItems, 1048576);
});

test("canonical producer response is closed and negative wire drift refuses", () => {
  const artifact = JSON.parse(read("artifacts/game-information-content-manifest-v1/manifest.json"));
  const value = JSON.parse(read("conformance/fixtures/game-information-content-manifest-v1/valid/canonical-manifest-response.json"));
  const keys = ["protocol_version","schema_digest","provenance","correlation_id","kind","manifest","error"];
  assert.deepEqual(Object.keys(value).sort(), keys.sort());
  assert.equal(value.protocol_version, artifact.protocol_version);
  assert.equal(value.schema_digest, artifact.schema_digest);
  assert.equal(value.kind, "content_manifest_response");
  assert.equal(value.error, null);
  assert.equal(value.manifest.inventory_revision.length, 64);
  const reject = (candidate) => candidate.protocol_version !== artifact.protocol_version
    || candidate.schema_digest !== artifact.schema_digest
    || Object.keys(candidate).length !== keys.length
    || !candidate.manifest
    || JSON.stringify(candidate).length > artifact.max_message_bytes;
  assert(reject({...value, protocol_version:"game-information-content-manifest-v2"}));
  assert(reject({...value, schema_digest:"0".repeat(64)}));
  assert(reject({...value, unknown:true}));
  assert(reject({...value, manifest:null}));
  assert(reject({...value, manifest:{...value.manifest, definitions:"x".repeat(artifact.max_message_bytes)}}));
});
