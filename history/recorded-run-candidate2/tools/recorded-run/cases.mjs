// SPDX-License-Identifier: MIT
import {fixture, missingAccounting, completedGameplay, bundleFor, entriesFor, record} from "./fixtures.mjs";
import {canonical} from "./jcs.mjs";
import {writeZip} from "./zip.mjs";
import {privacyDigest, semanticDigest, sha256} from "./validate.mjs";
export function partialSource() {
  const f=fixture(),s=f.report.streams.find(s=>s.stream==="trajectory");
  s.state="interrupted";s.input_records=7;s.rejected_rows=1;
  s.dispositions=[{first:6,last:6,disposition:"rejected",reason:"partial_final_record"}];
  f.manifest.completeness.status="partial";return f;
}
export function optionalUnknown() {
  const f=fixture(),r=f.events[0];f.manifest.optional_profiles=["example.optional.v2"];
  r.payload={profile:"example.optional.v2",kind:"opaque",value:{content_digest:privacyDigest("unsupported-value","synthetic"),bytes:12,media_type:"application/json"}};
  return f;
}
export const positives=()=>({
  "legacy-failed":bundleFor(fixture()),"legacy-failed-deflated":bundleFor(fixture(),true),
  "missing-accounting":bundleFor(missingAccounting()),"source-partial":bundleFor(partialSource()),
  "optional-unknown":bundleFor(optionalUnknown()),"completed-gameplay":bundleFor(completedGameplay())
});
export function repack(entries, path, bytes) {
  entries.set(path,Buffer.from(bytes));
  if(path!=="manifest.json") {
    const m=JSON.parse(entries.get("manifest.json"));
    const e=m.entries.find(e=>e.path===path); e.bytes=bytes.length;e.sha256=sha256(bytes);
    m.integrity.bundle_semantic_digest=semanticDigest(m);entries.set("manifest.json",Buffer.from(canonical(m)));
  }
  return writeZip(entries);
}
const mutate=f=>{const x=fixture();f(x);return bundleFor(x);};
export const negatives=()=>({
  "unsupported-major":mutate(f=>f.manifest.format_version="2.0.0"),
  "unsupported-required":mutate(f=>f.manifest.required_profiles.push("example.required.v2")),
  "workflow-field-invention":mutate(f=>f.events[0].workflow_run_id="invented"),
  "raw-rationale":mutate(f=>f.events[0].payload.value.rationale="SENTINEL_PRIVATE_PROMPT"),
  "raw-action-id":mutate(f=>f.events[4].identities.action.value="SENTINEL_RAW_ACTION"),
  "precision-number":mutate(f=>f.events[0].time.unix_ns=1789090000123456789),
  "missing-evidence":mutate(f=>delete f.events[0].evidence.gameplay),
  "provider-as-game-success":mutate(f=>f.manifest.evidence.gameplay="completed"),
  "unknown-action-as-settled":mutate(f=>{f.events[4].payload.value.status="settled";f.events[4].evidence.action="settled";}),
  "invented-model-join":mutate(f=>f.accounting[0].identities.model_execution.namespace="seed-readiness.trajectory.model-execution"),
  "unknown-usage-zero":mutate(f=>f.accounting[0].payload.value.usage.cached_input_tokens.value="0"),
  "omission-arithmetic":mutate(f=>f.report.streams.find(s=>s.stream==="mcp").filtered_rows=229),
  "overlapping-row-disposition":mutate(f=>f.report.streams[0].dispositions.push({first:0,last:0,disposition:"filtered",reason:"private_source_metadata"})),
  "duplicate-source-ordinal":mutate(f=>f.events[3].source.record_ordinal=0),
  "incomplete-source-as-complete":(()=>{const f=partialSource();f.manifest.completeness.status="complete";return bundleFor(f);})(),
  "partial-output":repack(entriesFor(fixture()),"records/events.ndjson",Buffer.from('{"profile":')),
  "duplicate-json-key":repack(entriesFor(fixture()),"reports/omissions.json",Buffer.from('{"profile":"x","profile":"y"}')),
  "unsafe-path":(()=>{const e=entriesFor(fixture());e.set("../escape",Buffer.from("x"));return writeZip(e);})(),
  "undeclared-entry":(()=>{const e=entriesFor(fixture());e.set("extra.json",Buffer.from("{}"));return writeZip(e);})(),
  "tampered-content":(()=>{const e=entriesFor(fixture());e.set("records/events.ndjson",Buffer.from("{}\n"));return writeZip(e);})(),
  "missing-manifest":(()=>{const e=entriesFor(fixture());e.delete("manifest.json");return writeZip(e);})()
});
