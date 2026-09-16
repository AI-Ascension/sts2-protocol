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
