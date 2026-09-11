// SPDX-License-Identifier: MIT
import {canonical} from "./jcs.mjs";
import {writeZip} from "./zip.mjs";
import {COMMON, STS, unknownEvidence} from "./semantics.mjs";
import {sha256, schemaDigest, semanticDigest, privacyDigest} from "./validate.mjs";
const id=(namespace,value)=>({namespace,value});
const digest=s=>sha256(Buffer.from(s));
export function record(stream, ordinal, kind, value, identities={}) {
  return {profile:COMMON,source:{stream,record_ordinal:ordinal,subrecord_ordinal:0},
    identities,evidence:unknownEvidence(),payload:{profile:STS,kind,value}};
}
export function fixture() {
  const action=privacyDigest("action","synthetic-action");
  const decision=record("decisions",0,"decision_summary",{action_id_digests:[action]});
  decision.time={unix_ns:"1789090000123456789"};
  const seed=record("trajectory",0,"seed_start",{
    protocol_version:"seeded-run-v1",schema_digest:digest("synthetic-seeded-schema"),
    context_digest:digest("synthetic-context"),generation:"8",run_started:true,host_ready:true,
    effect_kind:"run_started",requested_seed_digest:privacyDigest("seed","synthetic-seed"),
    canonical_seed_digest:privacyDigest("seed","synthetic-seed"),seed_match:true,status:"settled"
  },{operation:id("sts2.seeded.operation","synthetic-start"),session:id("sts2.runtime.session","synthetic-session")});
  seed.evidence={...unknownEvidence(),request:"accepted",action:"settled",outcome:"observed"};
  const model=record("trajectory",1,"decision_summary",{action_id_digests:[action],reused_model_execution:false,
    observation:{generation:"9",state_id_digest:digest("synthetic-state"),observation_digest:digest("synthetic-observation"),
      legal_action_count:3,player:{hp:"70",max_hp:"80",energy:"3",gold:"99"}}},
    {model_execution:id("seed-readiness.trajectory.model-execution","17")});
  const receipts=[2,3].map(n=>record("trajectory",n,"action_outcome",{status:"unknown"},
    {operation:id("sts2.runtime.operation","synthetic-operation"),action:id("ai-ascension.action.sha256",action)}));
  const wait=record("trajectory",4,"diagnostic",{code:"operation_wait_completed"});
  const failed=record("trajectory",5,"diagnostic",{code:"episode_failed",value_digest:privacyDigest("unsupported-value","other")});
  failed.evidence.gameplay="episode_failed";
  const result=record("result",0,"process_result",{exit_code:0},{session:id("seed-readiness.result.session","synthetic-session")});
  result.evidence.process_exit="completed";
  const accounting=record("provider-accounting",0,"accounting",{
    source_schema:"sts2.provider-accounting-v1",provider:"synthetic-provider",model:"synthetic-model",
    provider_execution_status:"completed",decision_status:"valid",provider_request_identity_kind:"codex_thread_id",
    provider_request_identity_status:"reported",request_sha256:digest("synthetic-request"),decision_sha256:digest("synthetic-decision"),
    counts:{event_count:"9",turn_count:"1",completed_turn_count:"1",stdout_bytes:"120",stderr_bytes:"0"},
    usage:{input_tokens:{unit:"tokens",scope:"model_execution",value_status:"reported",value:"20"},
      output_tokens:{unit:"tokens",scope:"model_execution",value_status:"reported",value:"7"},
      cached_input_tokens:{unit:"tokens",scope:"model_execution",value_status:"unknown",value:null}}
  },{model_execution:id("seed-readiness.accounting.model-execution","different-execution")});
  const stream=(name,n,emitted=n)=>({stream:name,state:emitted?"present":"omitted",input_records:n,
    emitted_rows:emitted,filtered_rows:n-emitted,unsupported_rows:0,rejected_rows:0,
    output_records:emitted,dispositions:emitted?[]:[{first:0,last:n-1,disposition:"filtered",
      reason:name==="mcp"?"raw_mcp_disallowed":"private_source_metadata"}],field_omissions:[]});
  const report={profile:COMMON,fixture_provenance:"synthetic",streams:[
    stream("decisions",1),stream("manifest",1,0),{...stream("mcp",230,0),mcp_redacted_path_count:0},
    stream("provider-accounting",1),stream("result",1),stream("trajectory",6)]};
  report.streams[0].field_omissions=[{rule:"raw_decision_text_disallowed",affected_rows:1}];
  const manifest={format:"ai-ascension.recorded-run-bundle",format_version:"1.0.0-candidate.2",
    contract_schema_sha256:schemaDigest,required_profiles:[COMMON,STS],optional_profiles:[],
    bundle_id:"",recording:{identity:id("seed-readiness.recording.sha256",privacyDigest("run","synthetic-run")),identities:{}},
    producer:{name:"sts2-harness",version:"synthetic",source_format:"seed-readiness-controller-release-v2"},
    adapter:{name:"synthetic-fixture",version:"1",source_revision:"synthetic"},
    versions:{},capabilities:{inspection:true,execution_replay:false,live_control:false,raw_mcp:false},
    completeness:{status:"complete",source_snapshot:"stable"},evidence:{...unknownEvidence(),process_exit:"completed",gameplay:"episode_failed"},
    entries:[],integrity:{digest_algorithm:"sha-256",bundle_semantic_digest:""}};
  return {manifest,report,events:[decision,result,seed,model,...receipts,wait,failed],accounting:[accounting]};
}
export function entriesFor(f) {
  const entries=new Map();
  const lines=rs=>Buffer.from(rs.map(r=>canonical(r)+"\n").join(""));
  entries.set("records/events.ndjson",lines(f.events));
  if(f.accounting!==null) entries.set("records/accounting.ndjson",lines(f.accounting));
  entries.set("reports/omissions.json",Buffer.from(canonical(f.report)));
  const m=structuredClone(f.manifest);
  m.entries=[...entries].sort(([a],[b])=>a<b?-1:1).map(([path,b])=>({path,
    media_type:path.endsWith(".ndjson")?"application/x-ndjson":"application/json",bytes:b.length,sha256:sha256(b)}));
  m.bundle_id=privacyDigest("bundle-id",canonical(m.recording.identity));
  m.integrity.bundle_semantic_digest=semanticDigest(m);
  entries.set("manifest.json",Buffer.from(canonical(m)));
  return entries;
}
export const bundleFor=(f,compress=false)=>writeZip(entriesFor(f),compress);
export function missingAccounting() {
  const f=fixture(); f.accounting=null;
  Object.assign(f.report.streams.find(s=>s.stream==="provider-accounting"),{state:"absent",input_records:null,
    emitted_rows:0,filtered_rows:0,unsupported_rows:0,rejected_rows:0,output_records:0,dispositions:[]});
  return f;
}
export function completedGameplay() {
  const f=fixture();
  f.manifest.producer.source_format="synthetic-authoritative-gameplay-v1";
  f.events=f.events.filter(r=>r.source.stream!=="trajectory");
  const r=record("trajectory",0,"gameplay_result",{status:"completed",result:"victory",
    authority:id("synthetic.game-host","completion"),source_profile:"synthetic-game-v1",
    source_schema_sha256:digest("synthetic-schema"),witness_sha256:digest("synthetic-witness")});
  r.payload.profile=COMMON;r.evidence={...unknownEvidence(),gameplay:"completed",outcome:"observed"};f.events.push(r);
  const s=f.report.streams.find(s=>s.stream==="trajectory"); s.input_records=1;s.emitted_rows=1;s.output_records=1;
  f.manifest.evidence.gameplay="completed";f.manifest.evidence.outcome="observed";return f;
}
