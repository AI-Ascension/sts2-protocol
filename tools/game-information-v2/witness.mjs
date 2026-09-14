// SPDX-License-Identifier: MIT
// Independent JavaScript accounting/capture witness; not a schema compiler or consumer adapter.
import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {readFileSync} from "node:fs";

export const PROFILE = "game-information-query-v2";
const schema = readFileSync(new URL(`../../schemas/${PROFILE}.schema.json`, import.meta.url));
export const DIGEST = createHash("sha256").update(schema).digest("hex");
export const canonical = (value) => {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
};
const bytes = (value) => Buffer.byteLength(canonical(value), "utf8");
const equal = (a, b) => assert.equal(canonical(a), canonical(b));
const normalize = (query) => Object.fromEntries(Object.entries(query).filter(([key]) => key !== "cursor"));
const mode = (id) => id === "smith" ? "card_selection" : id === "mend" ? "player_selection"
  : ["clone", "cook", "dig", "hatch", "heal", "kindle", "lift"].includes(id) ? "immediate" : null;

export function accounting(page) {
  const available = page.items.flatMap((item) => item.fields)
    .filter((field) => field.availability === "available");
  const text = available.flatMap((field) => field.kind === "text" ? [field.value]
    : field.kind === "text_list" ? field.value : []).reduce(
      (sum, value) => sum + Buffer.byteLength(value, "utf8"), 0);
  const plain = Object.fromEntries(Object.entries(page).filter(([key]) => key !== "accounting"));
  return {item_count: page.items.length, item_bytes: Math.max(0, ...page.items.map(bytes)),
    payload_bytes: bytes(page.items), page_bytes: bytes(plain), text_bytes: text};
}

function captureWitness(context) {
  const capture = context.rest;
  if (!capture) return;
  const options = capture.entries.map((entry) => entry.option_id);
  assert.equal(new Set(options).size, options.length);
  for (const entry of capture.entries) {
    const fields = Object.fromEntries(entry.fields.map((field) => [field.name, field]));
    assert.ok(mode(entry.option_id));
    assert.equal(fields.rest_mode.value, mode(entry.option_id));
    assert.equal(fields.rest_option_id.value, entry.option_id);
    if (fields.rest_action.availability === "available") {
      assert.equal(fields.rest_action.value.rest_option_id, entry.option_id);
      assert.ok(capture.legal_actions.some((action) =>
        canonical(action) === canonical(fields.rest_action.value)));
    }
    if (fields.rest_eligible.availability === "available") {
      assert.notEqual(entry.eligibility, null);
      assert.equal(fields.rest_eligible.value, entry.eligibility);
    }
    if (fields.rest_selector.availability === "available") {
      const selector = fields.rest_selector.value;
      assert.ok((entry.option_id === "smith" && selector.selection_kind === "card") ||
        (entry.option_id === "mend" && selector.selection_kind === "player"));
      assert.equal(selector.selected_count + selector.remaining_count, selector.required_count);
      assert.equal(new Set(entry.selected_choices).size, entry.selected_choices.length);
      assert.equal(selector.selected_count, entry.selected_choices.length);
    }
  }
}

function cursorWitness(token, query, context) {
  const cursor = context.cursors.find((known) => known.cursor === token);
  assert.ok(cursor);
  assert.equal(cursor.profile, PROFILE);
  assert.equal(cursor.schema_digest, DIGEST);
  assert.equal(cursor.producer_lease, context.authority.producer_lease);
  assert.equal(cursor.occurrence_revision, context.rest.revision);
  equal(cursor.normalized_query, normalize(query));
}

export function witness(value, context) {
  assert.equal(value.protocol_version, PROFILE);
  assert.equal(value.schema_digest, DIGEST);
  captureWitness(context);
  if (value.kind === "capabilities_response") {
    equal(value.capabilities, context.capabilities);
    return;
  }
  if (value.kind === "error_response") return;
  const query = value.query;
  const live = query.binding.mode === "live";
  if (live) {
    equal(query.binding, context.rest.context.binding);
    equal(query.parent_observation, context.rest.context.parent_observation);
  }
  if (query.cursor) cursorWitness(query.cursor, query, context);
  if (value.kind === "query_request") return;
  const result = value.result;
  assert.equal(result.read_only, true);
  equal(result.parent_observation, query.parent_observation);
  assert.equal(result.result_generation, live ? query.binding.snapshot_ref.state_generation : null);
  const page = result.page;
  equal(page.limits, query.limits);
  equal(page.accounting, accounting(page));
  for (const [name, actual] of [["page_items", page.items.length],
    ["item_bytes", page.accounting.item_bytes], ["page_bytes", page.accounting.page_bytes],
    ["text_bytes", page.accounting.text_bytes]]) assert.ok(actual <= page.limits[name]);
  if (page.next_cursor) {
    cursorWitness(page.next_cursor, query, context);
    equal(page.cursor_binding, normalize(query));
  }
  if (page.coverage === "complete") {
    assert.equal(page.final_page, true);
    assert.equal(context.rest.fully_classified, true);
    assert.ok(page.items.every((item) => item.fields.every((field) => field.availability === "available")));
  }
  if (["unavailable", "not_observable"].includes(page.coverage)) {
    assert.equal(page.items.length, 0);
    assert.equal(page.final_page, true);
    assert.equal(page.total_count_known, false);
    assert.equal(page.next_cursor, null);
    assert.equal(page.accounting.payload_bytes, 2);
  }
  let previous = "";
  for (const item of page.items) {
    const identity = canonical(item.definition_ref);
    assert.ok(identity > previous);
    previous = identity;
    const entry = context.rest.entries.find((candidate) => canonical(candidate.definition_ref) === identity);
    assert.ok(entry);
    equal(item.instance_ref, live ? entry.instance_ref : null);
    const fields = query.fields.length ? query.fields : live
      ? entry.fields.map((field) => field.name) : ["description", "display_name", "rest_mode", "rest_option_id"];
    equal(item.fields, entry.fields.filter((field) => fields.includes(field.name)));
    for (const field of item.fields) {
      if (field.availability !== "available") {
        assert.equal(field.value, null);
        assert.equal(field.unit, null);
        assert.equal(typeof field.reason, "string");
      }
    }
  }
}
