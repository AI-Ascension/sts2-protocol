// Read-only comparison of a protocol artifact with immutable consumer revisions.
import {createHash} from "node:crypto";
import {execFileSync} from "node:child_process";
import {mkdtempSync, readFileSync, rmSync, statSync} from "node:fs";
import {tmpdir} from "node:os";
import {basename, join, resolve} from "node:path";

const root = resolve(import.meta.dirname, "../..");

function fail(message) {
  throw new Error(`consumer compatibility: ${message}`);
}

function bytes(path) {
  try {
    return readFileSync(path);
  } catch (error) {
    fail(`cannot read ${path}: ${error.message}`);
  }
}

function digest(value) {
  return createHash("sha256").update(value).digest("hex");
}

function matrixAt(path) {
  let matrix;
  try {
    matrix = JSON.parse(bytes(path));
  } catch (error) {
    fail(`cannot parse matrix ${path}: ${error.message}`);
  }
  if (matrix.matrix_version !== 2 || !matrix.consumers || !Array.isArray(matrix.profiles)
    || matrix.profiles.length === 0 || Object.keys(matrix.consumers).length === 0) {
    fail("matrix must have version 2, consumers, and profiles");
  }
  for (const consumer of Object.values(matrix.consumers)) {
    if (!/^AI-Ascension\/[A-Za-z0-9_.-]+$/.test(consumer.repository)
      || !/^[0-9a-f]{40}$/.test(consumer.commit)
      || typeof consumer.artifact_root !== "string" || !safeRelative(consumer.artifact_root)
      || !["required", "advisory"].includes(consumer.tier)) {
      fail(`invalid consumer entry: ${JSON.stringify(consumer)}`);
    }
  }
  const names = new Set();
  const requiredConsumers = new Set(Object.entries(matrix.consumers)
    .filter(([, consumer]) => consumer.tier === "required").map(([name]) => name));
  const referencedConsumers = new Set();
  const pairs = new Set();
  for (const profile of matrix.profiles) {
    if (!profile || typeof profile.artifact !== "string" || !safeRelative(profile.artifact)
      || names.has(profile.artifact) || !Array.isArray(profile.consumers) || profile.consumers.length === 0
      || !/^[0-9a-f]{64}$/.test(profile.schema_digest)) fail(`invalid profile entry: ${JSON.stringify(profile)}`);
    names.add(profile.artifact);
    const profileConsumers = new Set();
    for (const name of profile.consumers) {
      if (!matrix.consumers[name]) fail(`profile ${profile.artifact} names unknown consumer ${name}`);
      if (profileConsumers.has(name)) fail(`profile ${profile.artifact} duplicates consumer ${name}`);
      profileConsumers.add(name);
      const pair = `${profile.artifact}\u0000${name}`;
      if (pairs.has(pair)) fail(`duplicate profile/consumer pair for ${profile.artifact} and ${name}`);
      pairs.add(pair);
      referencedConsumers.add(name);
    }
  }
  for (const name of requiredConsumers) {
    if (!referencedConsumers.has(name)) fail(`required consumer ${name} is not referenced by any profile`);
  }
  return matrix;
}

function safeRelative(path) {
  return path.length > 0 && !path.startsWith("/") && !path.split(/[\\/]/).some(part => !part || part === "." || part === "..");
}

function inventoryPaths(artifact, directory) {
  const entries = bytes(join(artifact, "SHA256SUMS")).toString("utf8").trim().split("\n");
  if (entries.length === 0 || entries[0] === "") fail(`empty SHA256SUMS for ${basename(artifact)}`);
  const paths = entries.map((line) => {
    const match = /^([0-9a-f]{64})  (.+)$/.exec(line);
    if (!match) fail(`invalid SHA256SUMS entry in ${basename(artifact)}: ${line}`);
    // Older inventories also bind repository-local ../../conformance files.
    // This gate reads only schema and golden bytes; the separate inventory CI
    // step verifies the full producer inventory from its intended directory.
    if (match[2] === "schema.json" || match[2].startsWith(`${directory}/`)) {
      if (!safeRelative(match[2])) fail(`unsafe wire path: ${match[2]}`);
      const actual = digest(bytes(join(artifact, match[2])));
      if (actual !== match[1]) fail(`wire inventory digest mismatch: ${match[2]}`);
    }
    return match[2];
  });
  if (new Set(paths).size !== paths.length || !paths.includes("schema.json")) fail(`invalid SHA256SUMS inventory for ${basename(artifact)}`);
  const goldens = paths.filter(path => path.startsWith(`${directory}/`));
  const manifest = JSON.parse(bytes(join(artifact, "manifest.json")));
  if (manifest.goldens !== undefined && (!Array.isArray(manifest.goldens)
    || manifest.goldens.some(path => !goldens.includes(path)))) {
    fail(`manifest golden inventory disagrees for ${basename(artifact)}`);
  }
  return {paths: ["schema.json", ...goldens].sort(), declared: [...(manifest.goldens ?? [])].sort()};
}

function compareArtifact({producer, consumer, schemaPath, goldenDirectory, schemaDigest}) {
  const sourceSchema = bytes(join(producer, schemaPath));
  if (digest(sourceSchema) !== schemaDigest) {
    fail(`producer schema digest does not match matrix for ${basename(producer)}`);
  }
  const consumerSchema = bytes(join(consumer, schemaPath));
  if (!sourceSchema.equals(consumerSchema)) {
    fail(`schema differs at ${consumer}`);
  }
  const sourceInventory = inventoryPaths(producer, goldenDirectory);
  const consumerInventory = inventoryPaths(consumer, goldenDirectory);
  if (JSON.stringify(sourceInventory) !== JSON.stringify(consumerInventory)) {
    fail(`wire golden inventory differs at ${consumer}`);
  }
  for (const path of sourceInventory.paths) {
    const source = bytes(join(producer, path));
    const consumerBytes = bytes(join(consumer, path));
    if (!source.equals(consumerBytes)) {
      fail(`artifact file differs at ${join(consumer, path)}`);
    }
  }
}

function checkedOutConsumer(consumer, checkoutRoot) {
  const destination = join(checkoutRoot, consumer.repository.replace("/", "--"));
  execFileSync("git", ["clone", "--no-checkout", "--filter=blob:none", `https://github.com/${consumer.repository}.git`, destination], {stdio: "inherit"});
  execFileSync("git", ["-C", destination, "fetch", "--depth", "1", "origin", consumer.commit], {stdio: "inherit"});
  execFileSync("git", ["-C", destination, "checkout", "--detach", consumer.commit], {stdio: "inherit"});
  const actual = execFileSync("git", ["-C", destination, "rev-parse", "HEAD"], {encoding: "utf8"}).trim();
  if (actual !== consumer.commit) fail(`checkout did not resolve ${consumer.repository} to its pinned commit`);
  return destination;
}

function assertLocalArtifactInputsClean(checkout, entry, profiles) {
  const changed = execFileSync("git", ["-C", checkout, "diff", "HEAD", "--name-only", "-z"], {encoding: "utf8"})
    + execFileSync("git", ["-C", checkout, "ls-files", "--others", "--exclude-standard", "-z"], {encoding: "utf8"});
  const relevant = profiles.map(profile => `${entry.artifact_root}/${profile.artifact}/`);
  for (const path of changed.split("\0").filter(Boolean)) {
    if (relevant.some(prefix => path.startsWith(prefix))) {
      fail(`local checkout has altered pinned artifact input ${entry.repository}:${path}`);
    }
  }
}

export function verify({matrixPath, consumerRoot, producerRoot, verifyLocalProvenance = false}) {
  const matrix = matrixAt(matrixPath);
  const temporaryRoot = consumerRoot ? null : mkdtempSync(join(tmpdir(), "sts2-protocol-consumers-"));
  try {
    const checkouts = new Map();
    for (const [name, entry] of Object.entries(matrix.consumers)) {
      if (!consumerRoot) checkouts.set(name, checkedOutConsumer(entry, temporaryRoot));
      else {
        const checkout = resolve(consumerRoot, entry.repository.replace("AI-Ascension/", ""));
        if (!checkout.startsWith(`${resolve(consumerRoot)}/`) || !statSync(checkout).isDirectory()) fail(`missing local checkout for ${entry.repository}`);
        if (verifyLocalProvenance) {
          const actual = execFileSync("git", ["-C", checkout, "rev-parse", "HEAD"], {encoding: "utf8"}).trim();
          if (actual !== entry.commit) fail(`local checkout did not resolve ${entry.repository} to its pinned commit`);
          assertLocalArtifactInputsClean(checkout, entry,
            matrix.profiles.filter(profile => profile.consumers.includes(name)));
        }
        checkouts.set(name, checkout);
      }
    }
    for (const profile of matrix.profiles) {
      const producer = producerRoot ? join(producerRoot, profile.artifact) : join(root, "artifacts", profile.artifact);
      for (const name of profile.consumers) {
        const entry = matrix.consumers[name];
        if (!entry) fail(`profile ${profile.artifact} names unknown consumer ${name}`);
        try {
          compareArtifact({
            producer,
            consumer: join(checkouts.get(name), entry.artifact_root, profile.artifact),
            schemaPath: "schema.json",
            goldenDirectory: "golden",
            schemaDigest: profile.schema_digest
          });
          console.log(`${entry.tier}: ${entry.repository}@${entry.commit} matches ${profile.artifact}`);
        } catch (error) {
          if (entry.tier === "required") throw error;
          console.warn(`advisory: ${entry.repository}@${entry.commit} differs for ${profile.artifact}: ${error.message}`);
        }
      }
    }
  } finally {
    if (temporaryRoot) rmSync(temporaryRoot, {recursive: true, force: true});
  }
}

if (process.argv[1] === import.meta.filename) {
  const consumerRootIndex = process.argv.indexOf("--consumer-root");
  const consumerRoot = consumerRootIndex < 0 ? undefined : resolve(process.argv[consumerRootIndex + 1] ?? fail("--consumer-root needs a value"));
  verify({
    matrixPath: resolve(root, "conformance/consumer-matrix/immutable-consumers.json"),
    consumerRoot,
    verifyLocalProvenance: Boolean(consumerRoot)
  });
}
