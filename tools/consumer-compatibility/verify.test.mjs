import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from "node:fs";
import {execFileSync} from "node:child_process";
import {tmpdir} from "node:os";
import {join} from "node:path";
import test from "node:test";
import {verify} from "./verify.mjs";

function fixture({missing = false, changed = false} = {}) {
  const root = mkdtempSync(join(tmpdir(), "sts2-consumer-compatibility-test-"));
  const producer = join(root, "artifacts", "fixture");
  const consumer = join(root, "consumers", "fixture-consumer", "artifact", "fixture");
  mkdirSync(join(producer, "golden"), {recursive: true});
  mkdirSync(join(consumer, "golden"), {recursive: true});
  writeFileSync(join(producer, "schema.json"), "schema\n");
  writeFileSync(join(producer, "golden", "message.json"), "golden\n");
  writeFileSync(join(producer, "manifest.json"), JSON.stringify({goldens: ["golden/message.json"]}));
  writeFileSync(join(consumer, "manifest.json"), JSON.stringify({goldens: ["golden/message.json"]}));
  writeFileSync(join(consumer, "schema.json"), changed ? "changed\n" : "schema\n");
  if (!missing) writeFileSync(join(consumer, "golden", "message.json"), changed ? "changed\n" : "golden\n");
  const inventory = [
    `${createHash("sha256").update("schema\n").digest("hex")}  schema.json`,
    `${createHash("sha256").update("golden\n").digest("hex")}  golden/message.json`
  ].join("\n") + "\n";
  writeFileSync(join(producer, "SHA256SUMS"), inventory);
  writeFileSync(join(consumer, "SHA256SUMS"), inventory);
  const matrix = {
    matrix_version: 2,
    consumers: {fixture: {repository: "AI-Ascension/fixture-consumer", commit: "a".repeat(40), artifact_root: "artifact", tier: "required"}},
    profiles: [{artifact: "fixture", schema_digest: createHash("sha256").update("schema\n").digest("hex"), consumers: ["fixture"]}]
  };
  const matrixPath = join(root, "matrix.json");
  writeFileSync(matrixPath, JSON.stringify(matrix));
  return {root, matrixPath, producerRoot: join(root, "artifacts")};
}

test("matches an immutable consumer artifact", () => {
  const value = fixture();
  try {
    verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot});
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("documents missing and changed consumer bytes as failures", () => {
  const missing = fixture({missing: true});
  const changed = fixture({changed: true});
  try {
    assert.throws(
      () => verify({matrixPath: missing.matrixPath, consumerRoot: join(missing.root, "consumers"), producerRoot: missing.producerRoot}),
      /cannot read.*golden\/message\.json/
    );
    assert.throws(
      () => verify({matrixPath: changed.matrixPath, consumerRoot: join(changed.root, "consumers"), producerRoot: changed.producerRoot}),
      /schema differs/
    );
  } finally {
    rmSync(missing.root, {recursive: true, force: true});
    rmSync(changed.root, {recursive: true, force: true});
  }
});

test("reports an advisory mismatch without making it a required gate", () => {
  const value = fixture({changed: true});
  try {
    const matrix = JSON.parse(readFileSync(value.matrixPath));
    matrix.consumers.fixture.tier = "advisory";
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot});
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("rejects a vacuous matrix and unsafe consumer paths", () => {
  const value = fixture();
  try {
    const matrix = JSON.parse(readFileSync(value.matrixPath));
    matrix.profiles = [];
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    assert.throws(() => verify({matrixPath: value.matrixPath}), /profiles/);
    matrix.profiles = [{artifact: "../fixture", schema_digest: "0".repeat(64), consumers: []}];
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    assert.throws(() => verify({matrixPath: value.matrixPath}), /invalid profile/);
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("rejects an omitted required consumer and duplicate profile consumer", () => {
  const value = fixture();
  try {
    const matrix = JSON.parse(readFileSync(value.matrixPath));
    matrix.consumers.unused = {...matrix.consumers.fixture, repository: "AI-Ascension/unused"};
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    assert.throws(() => verify({matrixPath: value.matrixPath}), /required consumer unused is not referenced/);
    delete matrix.consumers.unused;
    matrix.profiles[0].consumers.push("fixture");
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    assert.throws(() => verify({matrixPath: value.matrixPath}), /duplicates consumer fixture/);
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("rejects dirty pinned artifact inputs but permits unrelated local edits", () => {
  const value = fixture();
  const checkout = join(value.root, "consumers", "fixture-consumer");
  try {
    execFileSync("git", ["-C", checkout, "init", "--quiet"]);
    execFileSync("git", ["-C", checkout, "add", "."]);
    execFileSync("git", ["-C", checkout, "-c", "user.name=test", "-c", "user.email=test@example.invalid", "commit", "--quiet", "-m", "fixture"]);
    const matrix = JSON.parse(readFileSync(value.matrixPath));
    matrix.consumers.fixture.commit = execFileSync("git", ["-C", checkout, "rev-parse", "HEAD"], {encoding: "utf8"}).trim();
    writeFileSync(value.matrixPath, JSON.stringify(matrix));
    writeFileSync(join(checkout, "unrelated-ci-note.txt"), "allowed\n");
    verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot, verifyLocalProvenance: true});
    writeFileSync(join(checkout, "artifact", "fixture", "schema.json"), "dirty\n");
    assert.throws(() => verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot, verifyLocalProvenance: true}), /altered pinned artifact input/);
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("uses the frozen consumer inventory when a producer drops a golden", () => {
  const value = fixture();
  try {
    writeFileSync(join(value.producerRoot, "fixture", "manifest.json"), JSON.stringify({goldens: []}));
    assert.throws(
      () => verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot}),
      /golden inventory differs/
    );
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("keeps provenance differences separate from frozen wire compatibility", () => {
  const value = fixture();
  try {
    const producer = join(value.producerRoot, "fixture");
    const inventory = readFileSync(join(producer, "SHA256SUMS"), "utf8");
    writeFileSync(join(producer, "SHA256SUMS"), inventory +
      `${"b".repeat(64)}  README.md\n${"c".repeat(64)}  ../../conformance/cases/fixture.json\n`);
    verify({matrixPath: value.matrixPath, consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot});
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});

test("rejects a producer golden removal even when its manifest and inventory both shrink", () => {
  const value = fixture();
  try {
    const producer = join(value.producerRoot, "fixture");
    writeFileSync(join(producer, "manifest.json"), JSON.stringify({goldens: []}));
    const schemaLine = readFileSync(join(producer, "SHA256SUMS"), "utf8").split("\n")[0];
    writeFileSync(join(producer, "SHA256SUMS"), schemaLine + "\n");
    assert.throws(() => verify({matrixPath: value.matrixPath,
      consumerRoot: join(value.root, "consumers"), producerRoot: value.producerRoot}), /golden inventory differs/);
  } finally {
    rmSync(value.root, {recursive: true, force: true});
  }
});
