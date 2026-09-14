// SPDX-License-Identifier: MIT
// Explicit original conformance vectors. No test derives its expected outcome from the validator.
const mutate = (path, value) => ({path, value});
const response = "live-detail-heal-response";
export const cases = [];
function add(id, golden, expected, mutations = [], contextMutations = [], schemaValid = true, extra = {}) {
  cases.push({id, golden, context: "default", expected, mutations,
    context_mutations: contextMutations, schema_valid: schemaValid, recount: true, ...extra});
}
const bad = (id, path, value, expected = "malformed", schema = false, golden = response) =>
  add(id, golden, expected, [mutate(path, value)], [], schema);
bad("unknown-profile", "/protocol_version", "game-information-query-v9", "unsupported_version");
bad("unknown-digest", "/schema_digest", "0".repeat(64), "unsupported_version", true);
bad("wrong-provenance", "/provenance/artifact", "fixture:foreign");
bad("unknown-envelope-key", "/hidden", false);
bad("unknown-item-key", "/result/page/items/0/hidden", false);
bad("unknown-field-kind", "/result/page/items/0/fields/2/kind", "future_action_ref");
bad("unknown-entity-kind", "/query/entity_kind", "future_room");
bad("read-only-false", "/result/read_only", false, "read_only_violation");
bad("missing-required-null", "/result/page/items/0/fields/0/reason", null, "malformed");
bad("unknown-rest-option", "/result/page/items/0/fields/5/value", "future_option");
bad("unknown-mode", "/result/page/items/0/fields/4/value", "future_mode");
bad("wrong-field-unit", "/result/page/items/0/fields/4/unit", "count");
bad("unavailable-unit", "/result/page/items/0/fields/0/unit", "none");
bad("classification-unavailable", "/result/page/items/0/fields/4/availability", "unsupported");
bad("available-null", "/result/page/items/0/fields/4/value", null);
bad("action-id-129", "/result/page/items/0/fields/2/value/action_id", "a".repeat(129));
bad("action-extra-member", "/result/page/items/0/fields/2/value/dispatch", true);
bad("action-kind-drift", "/result/page/items/0/fields/2/value/kind", "heal");
bad("selector-required-257", "/result/page/items/0/fields/6/value/required_count", 257,
  "malformed", false, "live-detail-smith-response");
bad("selector-required-zero", "/result/page/items/0/fields/6/value/required_count", 0,
  "malformed", false, "live-detail-smith-response");
bad("selector-extra-member", "/result/page/items/0/fields/6/value/candidates", [],
  "malformed", false, "live-detail-smith-response");
bad("static-live-field", "/query/fields", ["rest_action"], "malformed", false, "static-get-response");
bad("live-search", "/query/query_kind", "search");
bad("static-detail", "/query/query_kind", "detail", "malformed", false, "static-get-response");
bad("rest-full-projection", "/query/projection", "full");
bad("live-filter", "/query/filters/namespaced_ids", ["fixture:rest:heal"]);
bad("rest-room-as-occurrence", "/result/page/items/0/instance_ref/entity_kind", "room",
  "invalid_identity", true);
bad("forged-occurrence", "/result/page/items/0/instance_ref/entity_id", "fixture:foreign",
  "invalid_identity", true);
bad("target-occurrence-confusion", "/query/target/instance_ref/entity_id", "fixture:room:1",
  "invalid_identity", true);
bad("foreign-definition", "/result/page/items/0/definition_ref/namespaced_id", "fixture:foreign",
  "invalid_identity", true);
bad("foreign-source", "/result/page/items/0/fields/0/source/ref", "fixture:foreign",
  "unsupported_field", true);
bad("wrong-result-generation", "/result/result_generation", 8, "mixed_generation", true);
bad("wrong-parent-generation", "/result/parent_observation/state_generation", 8,
  "mixed_generation", true);
bad("epoch-safe-integer-overflow", "/query/binding/instance_ref/epoch", 9007199254740992);
bad("negative-epoch", "/query/binding/instance_ref/epoch", -1);
bad("fractional-generation", "/query/binding/snapshot_ref/state_generation", 7.5);
bad("complete-despite-unavailable", "/result/page/coverage", "complete", "malformed", true);
bad("nonfinal-complete", "/result/page/coverage", "complete", "malformed", true,
  "live-list-page1-response");
bad("wrong-empty-array-accounting", "/result/page/accounting/payload_bytes", 0,
  "malformed", true, "live-unavailable-response");
cases.at(-1).recount = false;
bad("wrong-page-accounting", "/result/page/accounting/page_bytes", 1, "malformed", true);
cases.at(-1).recount = false;
bad("unknown-total-with-count", "/result/page/total_count_known", false);
bad("wrong-known-total", "/result/page/total_count", 2, "malformed", true);
bad("unavailable-nonempty", "/result/page/coverage", "unavailable", "malformed", true);
bad("wrong-rest-order", "/result/page/ordering/direction", "descending", "malformed", true);
add("missing-null-member", response, "malformed",
  [{path: "/result/page/items/0/definition_ref/variant", remove: true}], [], false);
add("duplicate-member", response, "malformed", [], [], true, {raw_replace: {
  from: '"epoch":1', to: '"epoch":1,"epoch":1'}});
add("fractional-integer-token", response, "malformed", [], [], true, {raw_replace: {
  from: '"epoch":1', to: '"epoch":1.0'}});
add("request-raw-byte-bound", "live-list-page1-request", "result_limit_exceeded",
  [], [], true, {raw_prefix_spaces: 16384});
add("raw-response-byte-bound", response, "result_limit_exceeded", [], [], true,
  {raw_prefix_spaces: 262144});
for (const [name, key, value] of [
  ["profile", "profile", "game-information-query-v1"],
  ["digest", "schema_digest", "0".repeat(64)],
  ["authority", "producer_lease", "fixture:other-lease"],
  ["revision", "occurrence_revision", "fixture:old-revision"],
]) add(`cursor-${name}`, "live-list-page2-request", "stale_cursor",
  [], [mutate(`/cursors/0/${key}`, value)]);
for (const [name, path, value] of [
  ["locale", "/query/binding/locale", "fr-FR"],
  ["manifest", "/query/binding/content_manifest_id", "fixture:other-manifest"],
  ["epoch", "/query/binding/instance_ref/epoch", 2],
  ["run", "/query/binding/instance_ref/run_id", "fixture:other-run"],
  ["scope", "/query/binding/visibility_scope", "fixture:denied"],
  ["limits", "/query/limits/page_items", 31],
  ["fields", "/query/fields", ["rest_mode"]],
  ["parent", "/query/parent_observation/state_generation", 8],
]) add(`cross-${name}-cursor`, "live-list-page2-request",
  name === "scope" ? "malformed" : "stale_cursor", [mutate(path, value)], [], name !== "scope");
add("cursor-offset", "live-list-page2-response", "invalid_identity",
  [], [mutate("/cursors/0/offset", 4)]);
add("cursor-too-many", response, "result_limit_exceeded", [],
  [mutate("/cursors", Array.from({length: 65}, (_, n) => ({
    cursor: `fixture:cursor:${n}`, profile: "game-information-query-v2",
    schema_digest: "0".repeat(64), producer_lease: "fixture:producer-lease",
    occurrence_revision: "fixture:occurrence-revision:7", normalized_query: {}, offset: 0,
  })))]);
for (const [name, path, value, expected] of [
  ["native-unknown-option", "/rest/entries/4/option_id", "future_option", "unsupported_field"],
  ["native-duplicate-option", "/rest/entries/4/option_id", "clone", "ambiguous_id"],
  ["capture-wrong-mode", "/rest/entries/4/fields/4/value", "card_selection", "unsupported_field"],
  ["selector-count-sum", "/rest/entries/8/fields/6/value/remaining_count", 0, "malformed"],
  ["selector-wrong-kind", "/rest/entries/8/fields/6/value/selection_kind", "player", "malformed"],
  ["selector-choice-count", "/rest/entries/8/selected_choices", ["fixture:choice:1"], "malformed"],
  ["selector-duplicate-choice", "/rest/entries/7/selected_choices", Array(256).fill("fixture:choice"), "malformed"],
  ["unregistered-action", "/rest/entries/4/fields/2/value/action_id", "fixture:not-offered", "invalid_identity"],
  ["capture-foreign-source", "/rest/entries/4/fields/0/source/ref", "fixture:foreign", "invalid_identity"],
  ["capture-room-occurrence", "/rest/entries/4/instance_ref/entity_id", "fixture:room:1", "invalid_identity"],
  ["missing-live-context", "/capabilities/rest_context",
    {availability: "unsupported", value: null, source: {kind: "synthetic", ref: "fixture:owned-rest"},
      reason: "source_unavailable"}, "missing_capability"],
]) add(name, response, expected, [], [mutate(path, value)]);
add("eligibility-not-inferred", response, "unsupported_field", [], [
  mutate("/rest/entries/4/fields/3", {name: "rest_eligible", kind: "boolean", value: true,
    availability: "available", unit: "none", reason: null,
    source: {kind: "synthetic", ref: "fixture:owned-rest"}}),
]);
add("source-domain-unknown", "static-classifications-response", "malformed", [],
  [mutate("/rest/fully_classified", false)]);
add("partial-domain-honest", "static-classifications-response", null,
  [mutate("/result/page/coverage", "partial")], [mutate("/rest/fully_classified", false)]);
add("static-search-no-index", "static-list-response", "unsupported_filter", [
  mutate("/query/query_kind", "search"), mutate("/query/filters/display_name", "Fixture text"),
]);
add("text-budget", response, "result_limit_exceeded", [
  mutate("/query/limits/text_bytes", 1), mutate("/result/page/limits/text_bytes", 1),
]);
add("item-budget", response, "result_limit_exceeded", [
  mutate("/query/limits/item_bytes", 1), mutate("/result/page/limits/item_bytes", 1),
]);
add("page-budget", response, "result_limit_exceeded", [
  mutate("/query/limits/page_bytes", 1), mutate("/result/page/limits/page_bytes", 1),
]);
add("item-count-budget", "live-list-page1-response", "stale_cursor", [
  mutate("/query/limits/page_items", 1), mutate("/result/page/limits/page_items", 1),
]);
// A response cursor also binds the changed limit; adjust its binding to isolate the count ceiling.
cases.at(-1).expected = "result_limit_exceeded";
add("capability-limit-ceiling", response, "result_limit_exceeded", [],
  [mutate("/capabilities/limits/page_items", 33)]);
add("nonrest-new-field", response, "malformed",
  [mutate("/result/page/items/0/definition_ref/entity_kind", "card")], [], false);
add("action-id-128", response, null,
  [mutate("/result/page/items/0/fields/2/value/action_id", "a".repeat(128))],
  [mutate("/rest/entries/4/fields/2/value/action_id", "a".repeat(128)),
    mutate("/rest/legal_actions/3/action_id", "a".repeat(128))]);
const noAction = {name: "rest_action", kind: "rest_action_ref", availability: "not_observable",
  value: null, unit: null, reason: "identity_out_of_bounds",
  source: {kind: "synthetic", ref: "fixture:owned-rest"}};
add("oversized-action-explicit-unavailable", response, null,
  [mutate("/result/page/items/0/fields/2", noAction)],
  [mutate("/rest/entries/4/fields/2", noAction)]);
const noSelector = {name: "rest_selector", kind: "rest_selector_ref", availability: "not_observable",
  value: null, unit: null, reason: "no_active_selector",
  source: {kind: "synthetic", ref: "fixture:owned-rest"}};
add("inactive-selector", "live-detail-smith-response", null,
  [mutate("/result/page/items/0/fields/6", noSelector)],
  [mutate("/rest/entries/8/fields/6", noSelector)]);
add("empty-query-fields-all-static", "static-list-response", null);
add("empty-query-fields-all-live", response, null);
add("selector-count-1", "live-detail-smith-response", null);
add("selector-count-256-zero-remaining", "live-detail-mend-response", null);
add("absent-action-unknown-eligibility", "live-detail-clone-response", null);
add("empty-unavailable-two-bytes", "live-unavailable-response", null);
add("empty-not-observable-two-bytes", "live-not_observable-response", null);
add("static-without-invented-live-context", "static-classifications-response", null,
  [], [], true, {context: "static-only"});
add("missing-field", response, "unsupported_field",
  [{path: "/result/page/items/0/fields/3", remove: true}]);
add("duplicate-field", response, "malformed",
  [mutate("/result/page/items/0/fields/5", {name: "rest_mode", kind: "text", value: "immediate",
    availability: "available", unit: "none", reason: null,
    source: {kind: "synthetic", ref: "fixture:owned-rest"}})]);
add("cursor-513", "live-list-page2-request", "malformed",
  [mutate("/query/cursor", "c".repeat(513))], [], false);
add("scope-denied", response, "denied_scope",
  [mutate("/query/binding/instance_ref/instance_id", "fixture:foreign-instance")]);
add("snapshot-stale", response, "stale_snapshot",
  [mutate("/query/binding/snapshot_ref/snapshot_id", "fixture:old-snapshot")]);
add("locale-stale", response, "stale_snapshot",
  [mutate("/query/binding/locale", "fr-FR")]);
const textField = (value) => ({name: "description", kind: "text", value,
  availability: "available", unit: "none", reason: null,
  source: {kind: "synthetic", ref: "fixture:owned-rest"}});
add("utf8-exact-text-budget", response, null,
  [mutate("/result/page/items/0/fields/0", textField("é".repeat(512))),
    mutate("/query/limits/text_bytes", 1037), mutate("/result/page/limits/text_bytes", 1037)],
  [mutate("/rest/entries/4/fields/0", textField("é".repeat(512)))]);
add("utf8-one-byte-over", response, "result_limit_exceeded",
  [mutate("/result/page/items/0/fields/0", textField("é".repeat(512))),
    mutate("/query/limits/text_bytes", 1036), mutate("/result/page/limits/text_bytes", 1036)],
  [mutate("/rest/entries/4/fields/0", textField("é".repeat(512)))]);
add("control-text", response, "malformed",
  [mutate("/result/page/items/0/fields/0", textField("fixture\u0001text"))], [], false);
add("no-id-as-description", response, "unsupported_field",
  [mutate("/result/page/items/0/fields/0", textField("fixture:rest:heal"))]);
for (const value of [true, false]) {
  const eligible = {name: "rest_eligible", kind: "boolean", value, availability: "available",
    unit: "none", reason: null, source: {kind: "synthetic", ref: "fixture:owned-rest"}};
  add(`explicit-eligibility-${value}`, response, null,
    [mutate("/result/page/items/0/fields/3", eligible)],
    [mutate("/rest/entries/4/fields/3", eligible), mutate("/rest/entries/4/eligibility", value)]);
}
// Request-only rejection diagnostics. Existing response cases above remain byte-identical.
const request = "live-detail-heal-request";
const asRequest = [mutate("/kind", "query_request"), mutate("/result", null)];
const requestCase = (id, expected, mutations, golden = request) =>
  add(id, golden, expected, mutations, [], false);
requestCase("request-rest-full-projection", "unsupported_projection",
  [mutate("/query/projection", "full")]);
requestCase("request-rest-summary-projection", "unsupported_projection",
  [mutate("/query/projection", "summary")]);
requestCase("request-rest-full-detail", "unsupported_projection",
  [mutate("/query/detail_level", "full")]);
requestCase("request-static-live-field", "unsupported_field",
  [...asRequest, mutate("/query/fields", ["rest_action"])], "static-get-response");
requestCase("request-static-other-field", "unsupported_field",
  [...asRequest, mutate("/query/fields", ["amount"])], "static-get-response");
requestCase("request-live-other-field", "unsupported_field", [mutate("/query/fields", ["amount"])]);
requestCase("request-live-get", "unsupported_projection", [mutate("/query/query_kind", "get")]);
requestCase("request-live-search-detail-target", "unsupported_projection",
  [mutate("/query/query_kind", "search")]);
requestCase("request-live-search-room-target", "unsupported_projection",
  [mutate("/query/query_kind", "search")], "live-list-page1-request");
requestCase("request-static-detail", "unsupported_projection",
  [...asRequest, mutate("/query/query_kind", "detail")], "static-get-response");
requestCase("request-live-filter", "unsupported_filter",
  [mutate("/query/filters/namespaced_ids", ["fixture:rest:heal"])]);
requestCase("request-live-display-filter", "unsupported_filter",
  [mutate("/query/filters/display_name", "Fixture text")]);
requestCase("request-projection-field-filter-priority", "unsupported_projection", [
  mutate("/query/projection", "full"), mutate("/query/fields", ["amount"]),
  mutate("/query/filters/display_name", "Fixture text"),
]);
requestCase("request-field-filter-priority", "unsupported_field", [
  mutate("/query/fields", ["amount"]), mutate("/query/filters/display_name", "Fixture text"),
]);
for (const [id, mutation] of [
  ["unknown-envelope-member", mutate("/extra", false)],
  ["unknown-query-member", mutate("/query/extra", false)],
  ["unknown-target-member", mutate("/query/target/extra", false)],
  ["unknown-binding-member", mutate("/query/binding/extra", false)],
  ["unknown-filter-member", mutate("/query/filters/extra", false)],
  ["unknown-provenance-member", mutate("/provenance/extra", false)],
  ["missing-query-member", {path: "/query/fields", remove: true}],
  ["missing-target-member", {path: "/query/target/definition_ref", remove: true}],
  ["missing-provenance-member", {path: "/provenance/generator", remove: true}],
  ["wrong-fields-type", mutate("/query/fields", "rest_mode")],
  ["wrong-limit-type", mutate("/query/limits/page_items", "32")],
  ["unknown-field-enum", mutate("/query/fields", ["future_field"])],
  ["duplicate-fields", mutate("/query/fields", ["amount", "amount"])],
  ["wrong-projection-type", mutate("/query/projection", 7)],
  ["unknown-projection-enum", mutate("/query/projection", "future_projection")],
  ["wrong-target-kind", mutate("/query/target/instance_ref/entity_kind", "room")],
  ["wrong-scope", mutate("/query/binding/visibility_scope", "public")],
  ["wrong-room-kind", mutate("/query/binding/instance_ref/entity_kind", "card")],
  ["wrong-provenance", mutate("/provenance/generator", "fixture:foreign")],
  ["missing-live-parent", mutate("/query/parent_observation", null)],
  ["invalid-limit", mutate("/query/limits/page_items", 129)],
  ["wrong-query-kind", mutate("/query/query_kind", "future_query")],
]) requestCase(`request-unsupported-plus-${id}`, "malformed",
  [mutate("/query/projection", "full"), mutation]);
