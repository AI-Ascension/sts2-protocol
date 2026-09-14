// SPDX-License-Identifier: MIT
// Original synthetic corpus; no product, host, or consumer capture is read.
import {createHash} from "node:crypto";
import {mkdirSync, writeFileSync, readFileSync, readdirSync} from "node:fs";
import {PROFILE, DIGEST, schemaBytes} from "./schema.mjs";

const root = new URL("../../", import.meta.url);
const artifact = `artifacts/${PROFILE}`;
const fixtureDir = `conformance/fixtures/${PROFILE}`;
const canonical = (value) => Array.isArray(value) ? `[${value.map(canonical).join(",")}]`
  : value !== null && typeof value === "object" ? `{${Object.keys(value).sort().map(
    (key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}` : JSON.stringify(value);
const bytes = (value) => Buffer.byteLength(canonical(value));
const write = (path, value) => {
  const url = new URL(path, root);
  mkdirSync(new URL(".", url), {recursive: true});
  writeFileSync(url, typeof value === "string" ? value : canonical(value) + "\n");
};
export function recount(envelope) {
  if (!envelope.result) return;
  const page = envelope.result.page;
  const bare = Object.fromEntries(Object.entries(page).filter(([key]) => key !== "accounting"));
  const text = page.items.flatMap((item) => item.fields).filter((field) =>
    field.availability === "available").reduce((sum, field) => sum +
      (field.kind === "text" ? Buffer.byteLength(field.value) : field.kind === "text_list"
        ? field.value.reduce((n, text) => n + Buffer.byteLength(text), 0) : 0), 0);
  page.accounting = {
    item_count: page.items.length, item_bytes: Math.max(0, ...page.items.map(bytes)),
    payload_bytes: bytes(page.items), page_bytes: bytes(bare), text_bytes: text,
  };
}
const source = {kind: "synthetic", ref: "fixture:owned-rest"};
const manifest = "fixture:manifest";
const authority = {producer_lease: "fixture:producer-lease", instance_id: "fixture:instance",
  run_id: "fixture:run", epoch: 1};
const instance = (kind, id) => ({instance_id: authority.instance_id, run_id: authority.run_id,
  epoch: authority.epoch, entity_kind: kind, entity_id: id});
const room = instance("room", "fixture:room:1");
const snapshot = {instance_ref: room, snapshot_id: "fixture:snapshot:7", state_generation: 7};
const binding = {mode: "live", content_manifest_id: manifest, locale: "en-US",
  visibility_scope: "player", instance_ref: room, snapshot_ref: snapshot};
const parent = {instance_ref: room, snapshot_ref: snapshot, state_generation: 7};
const limits = {page_items: 32, item_bytes: 4096, page_bytes: 65536, text_bytes: 4096};
const field = (name, kind, value, reason = null, availability = "available") => ({
  name, kind, value, reason, availability, unit: availability === "available" ? "none" : null, source,
});
const absent = (name, kind, reason, availability = "unsupported") =>
  field(name, kind, null, reason, availability);
const ids = ["clone", "cook", "dig", "hatch", "heal", "kindle", "lift", "mend", "smith"];
const entries = ids.map((id, index) => {
  const mode = id === "smith" ? "card_selection" : id === "mend" ? "player_selection" : "immediate";
  const selector = id === "smith" || id === "mend";
  const selected = id === "mend" ? 256 : 0;
  const selectorValue = {selection_id: `fixture:selector:${id}`,
    selection_kind: id === "smith" ? "card" : "player", required_count: id === "mend" ? 256 : 1,
    selected_count: selected, remaining_count: id === "mend" ? 0 : 1};
  return {
    definition_ref: {content_manifest_id: manifest, entity_kind: "rest_option",
      namespaced_id: `fixture:rest:${id}`, variant: null},
    instance_ref: instance("rest_option", `fixture:occurrence:${index}`), option_id: id,
    eligibility: null,
    selected_choices: Array.from({length: selected}, (_, n) => `fixture:choice:${n}`),
    fields: [
      absent("description", "text", "no_authoritative_text"),
      absent("display_name", "text", "no_authoritative_text"),
      id === "clone" ? absent("rest_action", "rest_action_ref", "no_offered_action", "not_observable")
        : field("rest_action", "rest_action_ref", {action_id: `rest-option:7:${id}`,
          kind: "rest_option", rest_option_id: id}),
      absent("rest_eligible", "boolean", "no_authoritative_eligibility"),
      field("rest_mode", "text", mode), field("rest_option_id", "text", id),
      selector ? field("rest_selector", "rest_selector_ref", selectorValue)
        : absent("rest_selector", "rest_selector_ref", "not_selector_option", "not_observable"),
    ],
  };
});
const capabilities = {
  profile: PROFILE, query_kinds: ["availability", "detail", "get", "list", "search"],
  entity_kinds: ["card", "character", "enemy", "event", "map_node", "potion", "power",
    "relic", "room", "status", "rest_option"],
  fields: ["amount", "cost", "description", "display_name", "flags", "owner", "position",
    "rarity", "source_id", "tags", "rest_action", "rest_eligible", "rest_mode", "rest_option_id", "rest_selector"],
  projections: ["full", "standard", "summary"], detail_levels: ["full", "standard", "summary"],
  limits, max_message_bytes: 262144, max_cursor_bytes: 512,
  snapshot_policy: {supports_live: true, lifetime_generations: 1, max_retained_snapshots: 1,
    expiry_behavior: "reject_stale_snapshot", invalidated_by: [
      "restore", "restart", "profile_change", "content_change", "run_change", "epoch_change"]},
  rest_context: {availability: "available", value: {binding, parent_observation: parent},
    source, reason: null},
};
const context = {
  capabilities, authority, rest: {content_manifest_id: manifest, locale: "en-US", context: capabilities.rest_context.value,
    revision: "fixture:occurrence-revision:7", fully_classified: true, text_index_available: false,
    legal_actions: entries.flatMap((entry) => entry.fields[2].availability === "available"
      ? [entry.fields[2].value] : []), sources: [source], entries}, cursors: [],
};
const query = (mode, kind = "list", entry = null, fields = []) => ({
  query_kind: kind, entity_kind: "rest_option", projection: "standard", detail_level: "standard",
  fields, cursor: null, limits,
  binding: mode === "live" ? binding : {...binding, mode, visibility_scope: "public",
    instance_ref: null, snapshot_ref: null},
  parent_observation: mode === "live" ? parent : null,
  filters: {display_name: null, namespaced_ids: [], definition_refs: [], instance_ids: []},
  target: {definition_ref: entry?.definition_ref ?? null,
    instance_ref: mode === "live" ? entry?.instance_ref ?? room : null},
});
const envelope = (kind, queryValue = null, result = null, caps = null, error = null) => ({
  protocol_version: PROFILE, schema_digest: DIGEST, correlation_id: "fixture:correlation",
  provenance: {artifact: `sts2-protocol/${PROFILE}`, source: `schemas/${PROFILE}.schema.json`,
    generator: "hand-authored"}, kind, query: queryValue, result, capabilities: caps, error,
});
const normalize = (query) => Object.fromEntries(Object.entries(query).filter(([key]) => key !== "cursor"));
const page = (q, selected, cursor = null, coverage = "partial", total = ids.length) => {
  const names = q.fields.length ? q.fields : q.binding.mode === "live" ? entries[0].fields.map((f) => f.name)
    : ["description", "display_name", "rest_mode", "rest_option_id"];
  const items = selected.map((entry) => ({definition_ref: entry.definition_ref,
    instance_ref: q.binding.mode === "live" ? entry.instance_ref : null,
    fields: entry.fields.filter((field) => names.includes(field.name))}));
  const result = {read_only: true, result_generation: q.binding.mode === "live" ? 7 : null,
    parent_observation: q.parent_observation,
    page: {items, next_cursor: cursor, cursor_binding: cursor ? normalize(q) : null,
      final_page: cursor === null, total_count_known: total !== null, total_count: total,
      coverage, ordering: {key: "definition_ref", direction: "ascending",
        algorithm: "identity_bytes", deterministic: true}, limits: q.limits, accounting: {}}};
  const value = envelope("query_response", q, result);
  recount(value);
  return value;
};
const goldens = {};
goldens["capabilities-response"] = envelope("capabilities_response", null, null, capabilities);
const live1 = query("live");
const live2 = {...live1, cursor: "fixture:cursor:live:2"};
context.cursors.push({cursor: live2.cursor, profile: PROFILE, schema_digest: DIGEST,
  producer_lease: authority.producer_lease, occurrence_revision: context.rest.revision,
  normalized_query: normalize(live1), offset: 5});
goldens["live-list-page1-request"] = envelope("query_request", live1);
goldens["live-list-page1-response"] = page(live1, entries.slice(0, 5), live2.cursor);
goldens["live-list-page2-request"] = envelope("query_request", live2);
goldens["live-list-page2-response"] = page(live2, entries.slice(5));
goldens["static-list-response"] = page(query("static"), entries);
goldens["static-classifications-response"] = page(
  query("static", "list", null, ["rest_mode", "rest_option_id"]), entries, null, "complete");
for (const entry of entries) {
  const detail = query("live", "detail", entry);
  goldens[`live-detail-${entry.option_id}-request`] = envelope("query_request", detail);
  goldens[`live-detail-${entry.option_id}-response`] = page(detail, [entry], null, "partial", 1);
}
for (const kind of ["get", "availability"]) {
  const staticQuery = query("static", kind, entries[4], ["rest_mode", "rest_option_id"]);
  goldens[`static-${kind}-response`] = page(staticQuery, [entries[4]], null, "complete", 1);
}
goldens["live-availability-response"] = page(
  query("live", "availability", entries[4]), [entries[4]], null, "partial", 1);
for (const coverage of ["unavailable", "not_observable"]) {
  goldens[`live-${coverage}-response`] = page(live1, [], null, coverage, null);
}
goldens["error-stale-snapshot"] = envelope("error_response", live1, null, null,
  {code: "stale_snapshot", field: null, reason: "reacquire_context", retryable: false});
goldens["capabilities-unavailable-response"] = envelope("capabilities_response", null, null,
  {...capabilities, rest_context: {availability: "unsupported", value: null, source,
    reason: "source_unavailable"}});
const unavailable = structuredClone(context);
unavailable.capabilities = goldens["capabilities-unavailable-response"].capabilities;
write(`${fixtureDir}/contexts/default.json`, context);
write(`${fixtureDir}/contexts/unavailable.json`, unavailable);
const staticOnly = structuredClone(unavailable);
staticOnly.rest.context = null;
staticOnly.rest.legal_actions = [];
staticOnly.cursors = [];
for (const entry of staticOnly.rest.entries) {
  entry.instance_ref = null;
  entry.selected_choices = [];
  entry.fields[2] = absent("rest_action", "rest_action_ref", "no_offered_action", "not_observable");
  entry.fields[6] = absent("rest_selector", "rest_selector_ref", "no_active_selector", "not_observable");
}
write(`${fixtureDir}/contexts/static-only.json`, staticOnly);
write(`schemas/${PROFILE}.schema.json`, schemaBytes);
write(`${artifact}/schema.json`, schemaBytes);
for (const [name, value] of Object.entries(goldens)) write(`${artifact}/golden/${name}.json`, value);
// Keep originals distinct from policy/runtime/consumer evidence.
const manifestValue = {
  artifact: `sts2-protocol/${PROFILE}`, protocol_version: PROFILE, status: "candidate",
  schema: "schema.json", schema_digest: DIGEST,
  provenance: {source: `schemas/${PROFILE}.schema.json`, generator: "hand-authored", license: "MIT"},
  consumers: [], prospective_consumers: ["sts2-gateway", "sts2-mcp-server"],
  prospective_producer: "sts2-game-mod", consumer_evidence: "unverified", live_status: "unverified",
  goldens: Object.keys(goldens).sort().map((name) => `golden/${name}.json`),
  conformance: `../../conformance/cases/${PROFILE}.json`, checksums: "SHA256SUMS",
};
write(`${artifact}/manifest.json`, manifestValue);
// Mutation fixture construction is separate so each case remains explicit and reviewable.
const {cases} = await import("./mutations.mjs");
for (const entry of cases) write(`${fixtureDir}/cases/${entry.id}.json`, entry);
const conformance = {profile: PROFILE, schema_digest: DIGEST, evidence: "synthetic-source-only",
  golden_contexts: Object.fromEntries(Object.keys(goldens).map((name) =>
    [name, name === "capabilities-unavailable-response" ? "unavailable" : "default"])),
  cases: cases.map((entry) => `../../${fixtureDir}/cases/${entry.id}.json`),
  pending: ["two-real-consumer-confirmations", "gateway-negotiation", "mcp-structured-output",
    "mod-read-port-spy", "native-source-membership", "harness-replay", "native-thread-locale"]};
write(`conformance/cases/${PROFILE}.json`, conformance);
write(`${artifact}/conformance.json`, conformance);
const files = [
  `schemas/${PROFILE}.schema.json`, `conformance/cases/${PROFILE}.json`,
  ...readdirSync(new URL(`${fixtureDir}/contexts/`, root)).map((name) => `${fixtureDir}/contexts/${name}`),
  ...cases.map((entry) => `${fixtureDir}/cases/${entry.id}.json`),
  `${artifact}/schema.json`, `${artifact}/manifest.json`, `${artifact}/conformance.json`,
  ...manifestValue.goldens.map((name) => `${artifact}/${name}`),
].sort();
const inventory = files.map((path) => `${createHash("sha256").update(readFileSync(
  new URL(path, root))).digest("hex")}  ${path.startsWith(`${artifact}/`)
    ? path.slice(artifact.length + 1) : `../../${path}`}`).join("\n") + "\n";
write(`${artifact}/SHA256SUMS`, inventory);
console.log(`${Object.keys(goldens).length} original goldens, ${cases.length} mutation cases; ${DIGEST}`);
