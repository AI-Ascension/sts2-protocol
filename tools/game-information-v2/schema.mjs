// SPDX-License-Identifier: MIT
// Deterministic original delta over this repository's immutable v1 schema.
import {createHash} from "node:crypto";
import {readFileSync, writeFileSync, mkdirSync} from "node:fs";

const old = readFileSync(new URL("../../schemas/game-information-query-v1.schema.json", import.meta.url));
if (createHash("sha256").update(old).digest("hex") !==
    "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9") {
  throw new Error("v1 schema pin changed");
}
const schema = JSON.parse(old);
export const PROFILE = "game-information-query-v2";
const ref = (name) => ({$ref: `#/$defs/${name}`});
const nullable = (value) => ({anyOf: [value, {type: "null"}]});
const object = (properties) => ({
  type: "object", additionalProperties: false, properties, required: Object.keys(properties),
});
const props = (properties) => ({properties});
const condition = (test, then, otherwise) => ({
  if: props(test), then: props(then), ...(otherwise ? {else: props(otherwise)} : {}),
});
const d = schema.$defs;
schema.$id = "sts2-game-information-query-v2";
schema.title = "STS2 game-information query v2 rest-read candidate";
schema.description = "Independent closed rest-read successor; candidate, no runtime adoption.";
d.envelope.properties.protocol_version.const = PROFILE;
d.capabilities.properties.profile.const = PROFILE;
d.provenance.properties.artifact.const = `sts2-protocol/${PROFILE}`;
d.provenance.properties.source.const = `schemas/${PROFILE}.schema.json`;
d.entity_kind.enum.push("rest_option");
const newNames = ["rest_action", "rest_eligible", "rest_mode", "rest_option_id", "rest_selector"];
d.field_name.enum.push(...newNames);
d.capabilities.properties.entity_kinds.maxItems = 11;
d.capabilities.properties.fields.maxItems = 15;
d.rest_option_id = {enum: ["clone", "cook", "dig", "hatch", "heal", "kindle", "lift", "smith", "mend"]};
d.rest_mode = {enum: ["immediate", "card_selection", "player_selection"]};
d.rest_action_ref = object({
  action_id: ref("identity"), kind: {const: "rest_option"}, rest_option_id: ref("rest_option_id"),
});
d.rest_selector_ref = object({
  selection_id: ref("identity"), selection_kind: {enum: ["card", "player"]},
  required_count: {type: "integer", minimum: 1, maximum: 256},
  selected_count: {type: "integer", minimum: 0, maximum: 256},
  remaining_count: {type: "integer", minimum: 0, maximum: 256},
});
const room = props({entity_kind: {const: "room"}});
d.rest_read_context = object({
  binding: ref("query_binding"), parent_observation: ref("parent_observation"),
});
d.rest_read_context.allOf = [props({binding: props({
  mode: {const: "live"}, visibility_scope: {const: "player"},
  instance_ref: room, snapshot_ref: props({instance_ref: room}),
})})];
const unavailable = ["unavailable", "not_observable", "redacted", "unsupported", "missing"];
d.rest_context = object({
  availability: {enum: ["available", ...unavailable]},
  value: nullable(ref("rest_read_context")), source: ref("source"),
  reason: {enum: [null, "not_rest_site", "source_unavailable", "identity_out_of_bounds",
    "scope_denied", "source_failed", "inconsistent_source"]},
});
d.rest_context.oneOf = [
  props({availability: {const: "available"}, value: ref("rest_read_context"), reason: {const: null}}),
  props({availability: {enum: unavailable}, value: {const: null}, reason: {not: {const: null}}}),
];
d.capabilities.properties.rest_context = ref("rest_context");
d.capabilities.required.push("rest_context");
for (const kind of ["rest_action_ref", "rest_selector_ref"]) {
  d.field.allOf[0].properties.kind.enum.push(kind);
  d.field.allOf[0].properties.value.oneOf.push(ref(kind));
  d.field.allOf[1].oneOf.push(props({kind: {const: kind}, value: nullable(ref(kind))}));
}
const rows = {
  description: ["text", ref("text")], display_name: ["text", ref("text")],
  rest_action: ["rest_action_ref", ref("rest_action_ref")],
  rest_eligible: ["boolean", {type: "boolean"}],
  rest_mode: ["text", ref("rest_mode")], rest_option_id: ["text", ref("rest_option_id")],
  rest_selector: ["rest_selector_ref", ref("rest_selector_ref")],
};
// These restrictions apply only inside rest items; existing text-field rules stay intact.
const restField = {allOf: [
  props({name: {enum: Object.keys(rows)}}),
  ...Object.entries(rows).map(([name, [kind, value]]) =>
    condition({name: {const: name}}, {kind: {const: kind}, value: nullable(value)})),
  condition({availability: {const: "available"}}, {unit: {const: "none"}}, {unit: {const: null}}),
  condition({name: {enum: ["rest_mode", "rest_option_id"]}}, {availability: {const: "available"}}),
]};
const oldField = props({
  name: {not: {enum: newNames}}, kind: {not: {enum: ["rest_action_ref", "rest_selector_ref"]}},
});
d.item.allOf = [condition(
  {definition_ref: props({entity_kind: {const: "rest_option"}})},
  {fields: {items: restField}}, {fields: {items: oldField}},
)];
const staticNames = ["description", "display_name", "rest_mode", "rest_option_id"];
const emptyFilters = props({
  display_name: {const: null}, namespaced_ids: {maxItems: 0},
  definition_refs: {maxItems: 0}, instance_ids: {maxItems: 0},
});
const restQuery = {
  projection: {const: "standard"}, detail_level: {const: "standard"},
  fields: {items: {enum: Object.keys(rows)}},
};
const staticMode = {binding: props({mode: {const: "static"}})};
const liveMode = {binding: props({mode: {const: "live"}})};
const restShape = {allOf: [
  props(restQuery),
  condition(staticMode, {
    query_kind: {enum: ["list", "search", "get", "availability"]},
    fields: {items: {enum: staticNames}}, binding: props({visibility_scope: {const: "public"}}),
    target: props({instance_ref: {type: "null"}}),
  }),
  condition(liveMode, {
    query_kind: {enum: ["list", "detail", "availability"]},
    binding: props({visibility_scope: {const: "player"}, instance_ref: room}),
    filters: emptyFilters,
  }),
  condition({query_kind: {enum: ["get", "detail", "availability"]}},
    {target: props({definition_ref: {allOf: [ref("definition_ref"),
      props({entity_kind: {const: "rest_option"}})]}})}),
  condition({query_kind: {enum: ["list", "search"]}},
    {target: props({definition_ref: {type: "null"}})}),
  {if: {allOf: [props(liveMode), props({query_kind: {const: "list"}})]},
    then: props({target: props({instance_ref: {allOf: [ref("instance_ref"), room]}})})},
  {if: {allOf: [props(liveMode), props({query_kind: {enum: ["detail", "availability"]}})]},
    then: props({target: props({instance_ref: {allOf: [ref("instance_ref"),
      props({entity_kind: {const: "rest_option"}})]}})})},
]};
for (const name of ["query", "cursor_binding"]) {
  (d[name].allOf ??= []).push({
    if: props({entity_kind: {const: "rest_option"}}), then: restShape,
    else: props({fields: {items: {not: {enum: newNames}}}}),
  });
}
export const schemaBytes = JSON.stringify(schema, null, 2) + "\n";
export const DIGEST = createHash("sha256").update(schemaBytes).digest("hex");
if (process.argv[1] && new URL(process.argv[1], "file:").href === import.meta.url) {
  const root = new URL("../../", import.meta.url);
  mkdirSync(new URL(`artifacts/${PROFILE}/`, root), {recursive: true});
  writeFileSync(new URL(`schemas/${PROFILE}.schema.json`, root), schemaBytes);
  writeFileSync(new URL(`artifacts/${PROFILE}/schema.json`, root), schemaBytes);
  console.log(DIGEST);
}
