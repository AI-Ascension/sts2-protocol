// SPDX-License-Identifier: MIT
// Local conformance tooling only; protocol product crate stays inert.
import {createHash} from "node:crypto";
import {readFileSync, openSync, fstatSync, readSync, closeSync} from "node:fs";
import {pathToFileURL} from "node:url";
import {canonical, parseCanonical, insist} from "./jcs.mjs";
import {readZip, LIMITS} from "./zip.mjs";
import {validateSchema} from "./schema.mjs";
import {checkProfiles, checkRecord, checkReconciliation, checkIdentityPrivacy} from "./semantics.mjs";
export const schemaBytes = readFileSync(new URL("../../artifacts/recorded-run-bundle-v1/schema.json", import.meta.url));
export const schema = JSON.parse(schemaBytes);
export const sha256 = bytes => createHash("sha256").update(bytes).digest("hex");
export const schemaDigest = sha256(schemaBytes);
export const privacyDigest = (kind, value) => sha256(Buffer.from("ai-ascension.recorded-run.v1/" + kind + "\0" + value,"utf8"));
export const semanticDigest = m => {
  const copy = structuredClone(m); delete copy.integrity.bundle_semantic_digest;
  return sha256(canonical(copy));
};
function ndjson(data) {
  if (!data.length) return [];
  insist(data.at(-1)===10, "partial_final_record");
  const result=[]; let start=0;
  for(let i=0;i<data.length;i++) if(data[i]===10) {
    insist(i>start && i-start<=LIMITS.record && result.length<LIMITS.records,"record_limit");
    const row=parseCanonical(data.subarray(start,i),LIMITS.record);
    validateSchema(schema,row,"record"); result.push(row); start=i+1;
  }
  return result;
}
export function validateBundle(input) {
  const entries=readZip(input);
  insist(entries.has("manifest.json"),"missing_manifest");
  const manifest=parseCanonical(entries.get("manifest.json"),LIMITS.manifest);
  validateSchema(schema,manifest,"manifest"); checkProfiles(manifest);
  checkIdentityPrivacy(manifest.recording.identities);
  insist(manifest.contract_schema_sha256===schemaDigest,"contract_digest");
  insist(manifest.integrity.bundle_semantic_digest===semanticDigest(manifest),"semantic_digest");
  insist(manifest.bundle_id===privacyDigest("bundle-id",canonical(manifest.recording.identity)),"bundle_identity");
  const names=manifest.entries.map(e=>e.path);
  insist(names.every((n,i)=>i===0||names[i-1]<n),"entry_order");
  insist(names.includes("records/events.ndjson")&&names.includes("reports/omissions.json"),"required_entries");
  insist(entries.size===names.length+1,"undeclared_entry");
  for(const e of manifest.entries) {
    const b=entries.get(e.path);
    insist(b && b.length===e.bytes && sha256(b)===e.sha256,"entry_integrity");
    insist(e.media_type===(e.path.endsWith(".ndjson")?"application/x-ndjson":"application/json"),"entry_media");
  }
  const report=parseCanonical(entries.get("reports/omissions.json"),LIMITS.omissions);
  validateSchema(schema,report,"omissions");
  const events=ndjson(entries.get("records/events.ndjson"));
  const accounting=entries.has("records/accounting.ndjson")?ndjson(entries.get("records/accounting.ndjson")):[];
  insist(events.length+accounting.length<=LIMITS.records,"record_count");
  insist(events.every(r=>r.payload.kind!=="accounting")&&accounting.every(r=>r.payload.kind==="accounting"),"accounting_authority");
  const records=[...events,...accounting];
  for(const file of [events,accounting]) {
    let previous=null; const subs=new Map();
    for(const r of file) {
      checkRecord(r,manifest);
      const s=r.source, key=s.stream+":"+s.record_ordinal;
      insist(s.subrecord_ordinal===(subs.get(key)??0),"subrecord_order");
      subs.set(key,s.subrecord_ordinal+1);
      const tuple=[s.stream,s.record_ordinal,s.subrecord_ordinal];
      if(previous) insist(tuple[0]>previous[0] || (tuple[0]===previous[0] &&
        (tuple[1]>previous[1] || tuple[1]===previous[1] && tuple[2]>previous[2])),"record_order");
      previous=tuple;
    }
  }
  insist(new Set(records.map(r=>canonical([r.source.stream,r.source.record_ordinal,r.source.subrecord_ordinal]))).size===records.length,"duplicate_record");
  checkReconciliation(manifest,report,records);
  return {manifest,omissions:report,events,accounting,summary:{
    semantic_digest:manifest.integrity.bundle_semantic_digest,recording_identity:manifest.recording.identity,
    evidence:manifest.evidence,completeness:manifest.completeness,streams:report.streams,
    event_records:events.length,accounting_records:accounting.length,
    unsupported_records:records.filter(r=>r.payload.kind==="opaque").length}};
}
function boundedFile(path) {
  const fd=openSync(path,"r");
  try {
    const st=fstatSync(fd); insist(st.isFile()&&st.size<=LIMITS.archive,"archive_limit");
    const b=Buffer.alloc(st.size); let n=0;
    while(n<b.length) {const got=readSync(fd,b,n,b.length-n,null); insist(got>0,"file_changed");n+=got;}
    insist(fstatSync(fd).size===st.size,"file_changed"); return b;
  } finally {closeSync(fd);}
}
if(process.argv[1] && import.meta.url===pathToFileURL(process.argv[1]).href) {
  try {
    insist(process.argv.length===3,"usage_node_validate_bundle");
    console.log(canonical({valid:true,...validateBundle(boundedFile(process.argv[2])).summary}));
  } catch(e) {console.error(canonical({valid:false,code:/^[a-z0-9_]+$/.test(e.message)?e.message:"validation_failed"}));process.exitCode=1;}
}
