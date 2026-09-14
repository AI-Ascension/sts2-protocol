// SPDX-License-Identifier: MIT
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import test from "node:test";
import {PROFILE, accounting, canonical, witness} from "./witness.mjs";

const root = new URL("../../", import.meta.url);
const read = (path) => JSON.parse(readFileSync(new URL(path, root), "utf8"));
const index = read(`conformance/cases/${PROFILE}.json`);
const golden = (name) => read(`artifacts/${PROFILE}/golden/${name}.json`);
const source = (name) => read(`conformance/fixtures/${PROFILE}/contexts/${name}.json`);
const mutate = (value, mutations) => {
  for (const change of mutations) {
    const parts = change.path.slice(1).split("/");
    const key = parts.pop();
    const parent = parts.reduce((value, key) => value[key], value);
    if (change.remove) {
      if (Array.isArray(parent)) parent.splice(Number(key), 1);
      else delete parent[key];
    } else parent[key] = structuredClone(change.value);
  }
};

test("independent witness accepts all original goldens and canonical accounting", () => {
  for (const [name, context] of Object.entries(index.golden_contexts)) {
    assert.doesNotThrow(() => witness(golden(name), source(context)), name);
    const raw = readFileSync(new URL(`artifacts/${PROFILE}/golden/${name}.json`, root), "utf8");
    assert.equal(raw, canonical(golden(name)) + "\n");
  }
});

test("all declared positive capture variants agree with the independent witness", () => {
  for (const path of index.cases) {
    const fixture = read(path.slice(6));
    if (fixture.expected !== null) continue;
    const value = golden(fixture.golden);
    const context = source(fixture.context);
    mutate(value, fixture.mutations);
    mutate(context, fixture.context_mutations);
    if (fixture.recount && value.result) value.result.page.accounting = accounting(value.result.page);
    assert.doesNotThrow(() => witness(value, context), fixture.id);
  }
});

test("independent capture/accounting negatives fail without using Rust rejection logic", () => {
  const cases = new Set(["wrong-result-generation", "wrong-parent-generation", "foreign-source",
    "forged-occurrence", "complete-despite-unavailable", "nonfinal-complete",
    "wrong-empty-array-accounting", "wrong-page-accounting", "selector-count-sum",
    "selector-wrong-kind", "selector-choice-count", "selector-duplicate-choice",
    "unregistered-action", "eligibility-not-inferred", "source-domain-unknown",
    "text-budget", "item-budget", "page-budget", "utf8-one-byte-over", "no-id-as-description"]);
  for (const path of index.cases) {
    const fixture = read(path.slice(6));
    if (!cases.has(fixture.id)) continue;
    const value = golden(fixture.golden);
    const context = source(fixture.context);
    mutate(value, fixture.mutations);
    mutate(context, fixture.context_mutations);
    if (fixture.recount && value.result) value.result.page.accounting = accounting(value.result.page);
    assert.throws(() => witness(value, context), undefined, fixture.id);
    cases.delete(fixture.id);
  }
  assert.equal(cases.size, 0);
});

test("two pages cover all nine original IDs and preserve unavailable fields", () => {
  const first = golden("live-list-page1-response");
  const second = golden("live-list-page2-response");
  const items = [...first.result.page.items, ...second.result.page.items];
  const ids = items.map((item) => item.fields.find((field) => field.name === "rest_option_id").value);
  assert.deepEqual(ids, ["clone", "cook", "dig", "hatch", "heal", "kindle", "lift", "mend", "smith"]);
  assert.equal(first.result.page.final_page, false);
  assert.equal(second.result.page.final_page, true);
  assert.equal(first.result.page.coverage, "partial");
  assert.equal(second.result.page.coverage, "partial");
  assert.ok(items.every((item) => item.fields.find((field) => field.name === "rest_eligible").value === null));
  assert.ok(items.every((item) => item.fields.find((field) => field.name === "description").value === null));
});
