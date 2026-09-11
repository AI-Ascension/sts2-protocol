// SPDX-License-Identifier: MIT
import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {fixture,bundleFor,record,completedGameplay} from "./fixtures.mjs";
import {identityAlias,punctuatedOptional,wrongProcessSource} from "./cases.mjs";
import {validateBundle} from "./validate.mjs";
import {checkRecord,checkReconciliation,unknownEvidence,COMMON,STS} from "./semantics.mjs";
import {readZip,writeZip,LIMITS} from "./zip.mjs";

test("review F1: wrong process stream with absent result is rejected",()=>{
  assert.throws(()=>validateBundle(bundleFor(wrongProcessSource())),/profile_source_stream/);
});
test("review F1: every STS2 kind/source mapping, without narrowing common profiles",()=>{
  const f=fixture(),examples=[
    f.events.find(r=>r.payload.kind==="seed_start"),
    record("trajectory",0,"observation_summary",{generation:"1",state_id_digest:"a".repeat(64),
      observation_digest:"b".repeat(64),legal_action_count:0}),
    f.events[0],f.events.find(r=>r.payload.kind==="action_outcome"),
    f.accounting[0],f.events.find(r=>r.payload.kind==="process_result")];
  for(const r of examples) {
    const good=r.source.stream;
    for(const bad of ["trajectory","decisions","provider-accounting","result","manifest","custom"]) {
      if(bad===good || (r.payload.kind==="decision_summary" && ["decisions","trajectory"].includes(bad))) continue;
      const changed=structuredClone(r);changed.source.stream=bad;
      assert.throws(()=>checkRecord(changed,f.manifest),/profile_source_stream/);
    }
  }
  const common=completedGameplay(),r=common.events.find(r=>r.payload.profile===COMMON);
  r.source.stream="custom-owner-stream";
  assert.doesNotThrow(()=>checkRecord(r,common.manifest));
  const opaque=punctuatedOptional();opaque.events[0].source.stream="custom-owner-stream";
  assert.doesNotThrow(()=>checkRecord(opaque.events[0],opaque.manifest));
});
test("review F2: source diagnostics require digests, source-free codes retain optionality",()=>{
  for(const code of ["unsupported_source_event","unsupported_source_status"]) {
    const f=fixture(),r=f.events.find(r=>r.payload.value.code==="operation_wait_completed");
    r.payload.value={code};assert.throws(()=>validateBundle(bundleFor(f)),/schema_record|diagnostic_digest/);
    r.payload.value.value_digest="a".repeat(64);assert.ok(validateBundle(bundleFor(f)));
  }
  assert.ok(validateBundle(bundleFor(fixture()))); // wait diagnostic has no digest.
});
test("review F3: role separation does not require tuple inequality",()=>{
  const f=identityAlias(),out=validateBundle(bundleFor(f));
  assert.deepEqual(out.manifest.recording.identity,out.manifest.recording.identities.session);
  f.manifest.recording.identities.session.namespace="another.owner.session";
  assert.ok(validateBundle(bundleFor(f)));
  const r=f.events.find(r=>r.payload.kind==="decision_summary"&&r.identities.model_execution);
  const a=f.accounting[0];
  a.identities.model_execution.value=r.identities.model_execution.value;
  const sameSpelling=validateBundle(bundleFor(f));
  assert.notDeepEqual(sameSpelling.accounting[0].identities.model_execution,r.identities.model_execution);
});
test("review F4: opaque names preserve the full manifest grammar",()=>{
  for(const name of ["Example.V2","example+v2","example:v2","example_v2","example-v2","Example_v2.+:x-1"]) {
    const f=punctuatedOptional();f.manifest.optional_profiles=[name];f.events[0].payload.profile=name;
    assert.ok(validateBundle(bundleFor(f)));
  }
});
test("review F5: all forbidden reason/disposition pairs fail",()=>{
  const admitted={raw_mcp_disallowed:"filtered",private_source_metadata:"filtered",
    unsupported_source_event:"unsupported",unsupported_source_status:"unsupported",
    invalid_source_record:"rejected",partial_final_record:"rejected"};
  for(const [reason,disposition] of Object.entries(admitted))
    for(const bad of ["filtered","unsupported","rejected"].filter(x=>x!==disposition)) {
      const f=fixture(),s=f.report.streams.find(s=>s.stream==="manifest");
      s.state="present";s.filtered_rows=0;s.rejected_rows=0;s.unsupported_rows=0;s[bad+"_rows"]=1;
      s.dispositions=[{first:0,last:0,reason,disposition:bad}];f.manifest.completeness.status="partial";
      assert.throws(()=>validateBundle(bundleFor(f)),/schema_omissions|disposition_reason/);
    }
});
test("review F5: correct class still requires correct source and tail placement",()=>{
  const f=fixture(),s=f.report.streams.find(s=>s.stream==="manifest");
  s.dispositions[0].reason="raw_mcp_disallowed";
  assert.throws(()=>validateBundle(bundleFor(f)),/disposition_reason/);
  s.dispositions[0]={first:0,last:0,disposition:"rejected",reason:"partial_final_record"};
  s.state="present";s.filtered_rows=0;s.rejected_rows=1;f.manifest.completeness.status="partial";
  assert.throws(()=>validateBundle(bundleFor(f)),/partial_tail_disposition/);
});
function manyRecords(events,accounting) {
  const f=fixture();
  f.events=Array.from({length:events},(_,n)=>record("decisions",n,"decision_summary",{action_id_digests:[]}));
  f.accounting=Array.from({length:accounting},(_,n)=>record("provider-accounting",n,"accounting",
    {source_schema:"sts2.provider-accounting-v1",provider_execution_status:"unknown",decision_status:"unknown",counts:{},usage:{}}));
  f.manifest.evidence=unknownEvidence();
  for(const name of ["decisions","provider-accounting","trajectory","result"]) {
    const s=f.report.streams.find(s=>s.stream===name),n=name==="decisions"?events:name==="provider-accounting"?accounting:0;
    Object.assign(s,{state:n?"present":"absent",input_records:n||null,emitted_rows:n,output_records:n,
      filtered_rows:0,unsupported_rows:0,rejected_rows:0,field_omissions:[],dispositions:[]});
  }
  return f;
}
test("review F6: 25,001 combined rows reject before parsing ANY record",()=>{
  const bytes=bundleFor(manyRecords(12501,12500),true),parse=JSON.parse;let materialized=0;
  JSON.parse=function(text,...args) {
    if(typeof text==="string" && text.includes('"record_ordinal"')) materialized++;
    return parse(text,...args);
  };
  try {assert.throws(()=>validateBundle(bytes),/record_count/);assert.equal(materialized,0);}
  finally {JSON.parse=parse;}
});
test("review F6: exactly 25,000 combined rows can validate",()=>{
  const result=validateBundle(bundleFor(manyRecords(12500,12500),true));
  assert.equal(result.events.length+result.accounting.length,LIMITS.records);
});
test("review F7: reject oversized typed view before any Buffer.from",()=>{
  const tooLarge=new Uint8Array(LIMITS.archive+1),from=Buffer.from;let copies=0;
  Buffer.from=function(...args) {copies++;return from(...args);};
  try {assert.throws(()=>readZip(tooLarge),/archive_limit/);assert.equal(copies,0);}
  finally {Buffer.from=from;}
});
test("review F7: accepted subview is bounded and shares only its selected extent",()=>{
  const zip=writeZip(new Map([["a.bin",Buffer.from("synthetic")]]));
  const backing=new Uint8Array(LIMITS.archive+100),offset=37;backing.set(zip,offset);
  const view=backing.subarray(offset,offset+zip.length),result=readZip(view).get("a.bin");
  assert.equal(result.toString(),"synthetic");assert.equal(result.buffer,backing.buffer);
  assert.throws(()=>readZip("not-byte-input"),/archive_input_type/);
  assert.throws(()=>readZip(new Uint8Array(new SharedArrayBuffer(32))),/archive_shared_buffer/);
});
test("candidate2 artifact and runnable tooling history keep their exact pin",async()=>{
  const {sha256}=await import("./validate.mjs");
  const prefix="../../history/recorded-run-candidate2/";
  const old=await import(prefix+"tools/recorded-run/validate.mjs");
  const root=new URL("../../artifacts/recorded-run-bundle-v1/",import.meta.url);
  assert.equal(sha256(readFileSync(new URL("SHA256SUMS",root))),"41d760f8c41064c4e6b49a48dbe6e1a6c8f2a9958afbc50374986a54858fd598");
  const b=readFileSync(new URL("golden/legacy-failed.zip",root));
  assert.equal(old.validateBundle(b).manifest.format_version,"1.0.0-candidate.2");
  assert.throws(()=>validateBundle(b),/schema_manifest/);
  const tooling=JSON.parse(readFileSync(new URL("tooling.json",root)));
  for(const item of tooling.files)
    assert.equal(sha256(readFileSync(new URL(prefix+item.path,import.meta.url))),item.sha256,item.path);
});
