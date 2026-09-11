// SPDX-License-Identifier: MIT
import {canonical, insist} from "./jcs.mjs";
export const COMMON = "ai-ascension.recorded-run.common.v1";
export const STS = "ai-ascension.sts2.seed-readiness.v1";
export const unknownEvidence = () => ({process_exit:"unknown", request:"unknown", action:"unknown", outcome:"unknown", gameplay:"unknown"});
const eq = (a,b) => canonical(a) === canonical(b);
const sortedUnique = xs => xs.every((x,i) => i === 0 || xs[i-1] < x);
const knownStreams = ["trajectory","decisions","mcp","provider-accounting","result","manifest"];
const sourceKinds = {
  seed_start:["trajectory"], observation_summary:["trajectory"],
  decision_summary:["trajectory","decisions"], action_outcome:["trajectory"],
  accounting:["provider-accounting"], process_result:["result"]
};
function checkSourceMapping(r) {
  const {kind,value} = r.payload, stream = r.source.stream;
  if (kind !== "diagnostic") {
    insist(sourceKinds[kind]?.includes(stream), "profile_source_stream");
    return;
  }
  const streams = {
    unsupported_source_event:["trajectory"],
    unsupported_source_status:["trajectory","provider-accounting"],
    operation_wait_completed:["trajectory"],episode_failed:["trajectory"],
    invalid_source_record:knownStreams,partial_final_record:knownStreams,
    unsupported_profile:knownStreams
  };
  insist(streams[value.code]?.includes(stream), "diagnostic_source_stream");
  if (["unsupported_source_event","unsupported_source_status"].includes(value.code))
    insist(typeof value.value_digest === "string" && /^[a-f0-9]{64}$/.test(value.value_digest), "diagnostic_digest");
}
function checkDisposition(d,s) {
  const pairs = {
    raw_mcp_disallowed:["filtered",["mcp"]],
    private_source_metadata:["filtered",["manifest"]],
    unsupported_source_event:["unsupported",["trajectory"]],
    unsupported_source_status:["unsupported",["trajectory","provider-accounting"]],
    invalid_source_record:["rejected",null],partial_final_record:["rejected",null]
  };
  const pair=pairs[d.reason];
  insist(pair && d.disposition===pair[0] && (!pair[1]||pair[1].includes(s.stream)), "disposition_reason");
  if (d.reason==="partial_final_record")
    insist(s.state==="interrupted" && d.first===d.last && d.last===s.input_records-1, "partial_tail_disposition");
}
export function checkIdentityPrivacy(identities) {
  for (const [key, namespace] of [["action","ai-ascension.action.sha256"],
    ["provider_request","ai-ascension.provider-request.sha256"]]) {
    const value = identities[key];
    if (value) insist(value.namespace === namespace && /^[a-f0-9]{64}$/.test(value.value), key + "_privacy");
  }
}
export function checkProfiles(m) {
  insist(sortedUnique(m.required_profiles) && sortedUnique(m.optional_profiles), "profile_order");
  insist(m.required_profiles.includes(COMMON), "common_required");
  insist(m.required_profiles.every(p => [COMMON,STS].includes(p)), "unsupported_required_profile");
  insist(!m.optional_profiles.some(p => m.required_profiles.includes(p)), "profile_duplicate");
  insist(!m.optional_profiles.includes(COMMON), "common_optional");
}
export function checkRecord(r, m) {
  const {profile, kind, value: v} = r.payload;
  checkIdentityPrivacy(r.identities);
  insist([...m.required_profiles, ...m.optional_profiles].includes(profile), "undeclared_profile");
  const expected = unknownEvidence();
  if (profile === COMMON && kind === "gameplay_result") {
    insist(m.producer.source_format !== "seed-readiness-controller-release-v2", "legacy_gameplay_overclaim");
    expected.gameplay = "completed";
    expected.outcome = "observed";
    insist(eq(r.evidence, expected), "gameplay_evidence");
    return;
  }
  if (profile !== STS) {
    insist(profile !== COMMON && m.optional_profiles.includes(profile) && kind === "opaque", "opaque_profile");
    insist(eq(r.evidence, expected), "opaque_evidence");
    return;
  }
  insist(kind !== "opaque", "known_profile_opaque");
  checkSourceMapping(r);
  if (kind === "process_result") expected.process_exit = v.exit_code === 0 ? "completed" : "failed";
  if (kind === "diagnostic" && v.code === "episode_failed") expected.gameplay = "episode_failed";
  if (kind === "seed_start") {
    expected.action = v.status;
    if (v.status === "accepted" || v.status === "settled") expected.request = "accepted";
    if (v.status === "rejected") expected.request = "rejected";
    if (v.status === "settled") {
      insist(v.run_started && v.host_ready && v.effect_kind === "run_started" && r.identities.operation, "seed_settlement");
      expected.outcome = "observed";
    } else insist(v.effect_kind === "unknown" && !v.run_started, "seed_unsettled_witness");
    insist(v.seed_match === (v.requested_seed_digest === v.canonical_seed_digest), "seed_match");
  }
  if (kind === "action_outcome") {
    // This legacy profile admits only the actually probed Unknown receipt.
    insist(v.status === "unknown" && !v.observation && !v.from_generation &&
      !v.to_generation && !v.effect_digest, "unadmitted_action_settlement");
  }
  if (kind === "accounting") {
    insist(r.source.stream === "provider-accounting", "accounting_stream");
    for (const u of Object.values(v.usage))
      insist((["unknown","not_applicable"].includes(u.value_status)) === (u.value === null), "unknown_usage");
    const c = v.counts;
    if (c.completed_turn_count !== undefined && c.turn_count !== undefined)
      insist(BigInt(c.completed_turn_count) <= BigInt(c.turn_count), "accounting_turns");
  }
  if (r.identities.model_execution) {
    const ns = r.identities.model_execution.namespace;
    insist(ns === (kind === "accounting" ? "seed-readiness.accounting.model-execution" :
      "seed-readiness.trajectory.model-execution"), "model_execution_namespace");
  }
  insist(eq(r.evidence, expected), "evidence_mismatch");
}
export function checkReconciliation(m, report, records) {
  const streams = report.streams;
  insist(sortedUnique(streams.map(s=>s.stream)), "stream_order");
  for (const name of ["trajectory","decisions","mcp","provider-accounting","result","manifest"])
    insist(streams.some(s=>s.stream===name), "missing_stream_disposition");
  insist(records.every(r=>streams.some(s=>s.stream===r.source.stream)), "undeclared_stream");
  for (const s of streams) {
    const rows = records.filter(r=>r.source.stream===s.stream);
    const emitted = new Set(rows.map(r=>r.source.record_ordinal));
    insist(rows.length === s.output_records && emitted.size === s.emitted_rows, "output_counts");
    if (["absent","unknown"].includes(s.state)) {
      insist(s.input_records === null && s.emitted_rows + s.filtered_rows + s.unsupported_rows +
        s.rejected_rows + s.output_records === 0 && s.dispositions.length === 0 && s.field_omissions.length === 0, "absent_counts");
      continue;
    }
    insist(s.input_records !== null, "missing_input_count");
    const covered = new Set(emitted), counts = {filtered:0,unsupported:0,rejected:0};
    for (const ordinal of covered) insist(ordinal < s.input_records, "source_ordinal");
    let previous = -1;
    for (const d of s.dispositions) {
      checkDisposition(d,s);
      insist(d.first <= d.last && d.first > previous && d.last < s.input_records, "disposition_range");
      previous = d.last;
      for (let n=d.first;n<=d.last;n++) { insist(!covered.has(n), "disposition_overlap"); covered.add(n); counts[d.disposition]++; }
    }
    insist(covered.size === s.input_records && counts.filtered === s.filtered_rows &&
      counts.unsupported === s.unsupported_rows && counts.rejected === s.rejected_rows, "reconciliation");
    insist(s.emitted_rows+s.filtered_rows+s.unsupported_rows+s.rejected_rows === s.input_records, "row_arithmetic");
    for (const f of s.field_omissions) insist(f.affected_rows <= s.input_records, "field_omission_count");
    insist(new Set(s.field_omissions.map(f=>f.rule)).size===s.field_omissions.length, "field_rule_duplicate");
    if (s.state === "omitted") insist(s.filtered_rows === s.input_records, "omitted_state");
    if (s.state === "unsupported") insist(s.unsupported_rows === s.input_records, "unsupported_state");
    if (s.state === "interrupted") insist(s.dispositions.some(d=>d.reason==="partial_final_record" &&
      d.disposition==="rejected" && d.first===d.last && d.last===s.input_records-1), "interrupted_tail");
    if (s.stream === "mcp") insist(s.emitted_rows === 0 && s.filtered_rows === s.input_records &&
      s.dispositions.every(d=>d.reason==="raw_mcp_disallowed"), "mcp_disallowed");
  }
  // Run evidence is deliberately conservative: action/run-start are record scoped.
  const e = unknownEvidence();
  const exits = records.filter(r=>r.payload.kind==="process_result");
  insist(exits.length <= 1, "duplicate_process_result");
  if (exits.length) e.process_exit = exits[0].evidence.process_exit;
  if (records.some(r=>r.payload.kind==="diagnostic" && r.payload.value.code==="episode_failed")) e.gameplay = "episode_failed";
  if (records.some(r=>r.payload.kind==="gameplay_result")) {
    insist(e.gameplay !== "episode_failed", "conflicting_gameplay_evidence");
    e.gameplay = "completed";
    e.outcome = "observed";
  }
  insist(eq(m.evidence,e), "summary_evidence");
  if (m.completeness.status === "complete") insist(m.completeness.source_snapshot === "stable" &&
    streams.every(s=>!["unknown","interrupted","unsupported"].includes(s.state) &&
      s.rejected_rows===0 && s.unsupported_rows===0), "completeness_overclaim");
}
