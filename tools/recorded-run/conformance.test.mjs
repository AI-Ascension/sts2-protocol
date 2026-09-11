// SPDX-License-Identifier: MIT
import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {canonical, parseCanonical} from "./jcs.mjs";
import {validateBundle, schemaDigest, sha256} from "./validate.mjs";
import {fixture, bundleFor, entriesFor} from "./fixtures.mjs";
import {positives, negatives, repack} from "./cases.mjs";
import {readZip, writeZip, LIMITS} from "./zip.mjs";

for(const [name,b] of Object.entries(positives())) test("accept "+name,()=>assert.ok(validateBundle(b)));
for(const [name,b] of Object.entries(negatives())) test("reject "+name,()=>assert.throws(()=>validateBundle(b)));

test("repeated exports and container compression preserve semantic identity",()=>{
  const a=validateBundle(bundleFor(fixture())),b=validateBundle(bundleFor(fixture(),true));
  assert.equal(a.summary.semantic_digest,b.summary.semantic_digest);
  assert.equal(a.summary.evidence.gameplay,"episode_failed");
  assert.equal(a.summary.evidence.process_exit,"completed");
  assert.equal(a.accounting[0].payload.value.provider_execution_status,"completed");
  assert.notDeepEqual(a.events.find(r=>r.source.stream==="trajectory"&&r.source.record_ordinal===1).identities.model_execution,
    a.accounting[0].identities.model_execution);
  assert.equal(a.events.filter(r=>r.payload.kind==="action_outcome").length,2);
  assert.ok(a.events.filter(r=>r.payload.kind==="action_outcome").every(r=>r.evidence.action==="unknown"));
});
test("fanout source arithmetic counts rows separately",()=>{
  const f=fixture(),extra=structuredClone(f.events[0]);extra.source.subrecord_ordinal=1;
  f.events.splice(1,0,extra);f.report.streams[0].output_records++;
  assert.equal(validateBundle(bundleFor(f)).summary.event_records,9);
});
test("optional public numeric player summary stays precise and closed",()=>{
  const f=fixture(),r=f.events.find(r=>r.payload.kind==="decision_summary"&&r.payload.value.observation);
  r.payload.value.observation.player.gold="9007199254740993";
  delete r.payload.value.observation.player.energy;
  const parsed=validateBundle(bundleFor(f)).events.find(r=>r.payload.value.observation);
  assert.equal(parsed.payload.value.observation.player.gold,"9007199254740993");
  assert.equal(Object.hasOwn(parsed.payload.value.observation.player,"energy"),false);
  r.payload.value.observation.player.hp=70;
  assert.throws(()=>validateBundle(bundleFor(f)));
  r.payload.value.observation.player.hp="70";r.payload.value.observation.player.name="SENTINEL_PRIVATE";
  assert.throws(()=>validateBundle(bundleFor(f)));
});
test("privacy identities apply in manifest and unsupported optional envelopes",()=>{
  const f=fixture();
  f.manifest.recording.identities.action={namespace:"raw",value:"SENTINEL_ACTION"};
  assert.throws(()=>validateBundle(bundleFor(f)));
  delete f.manifest.recording.identities.action;
  f.manifest.optional_profiles=["example.optional.v2"];
  f.events[0].payload={profile:"example.optional.v2",kind:"opaque",value:{content_digest:"a".repeat(64),bytes:1,media_type:"application/json"}};
  f.events[0].identities.provider_request={namespace:"raw",value:"SENTINEL_REQUEST"};
  assert.throws(()=>validateBundle(bundleFor(f)));
});
test("golden file checksums and source schema match",()=>{
  const root=new URL("../../artifacts/recorded-run-bundle-v1-candidate3/",import.meta.url);
  assert.equal(sha256(readFileSync(new URL("../../schemas/recorded-run-bundle-v1-candidate3.schema.json",import.meta.url))),schemaDigest);
  for(const line of readFileSync(new URL("SHA256SUMS",root),"utf8").trim().split("\n")) {
    const [sum,path]=line.split("  ");assert.equal(sha256(readFileSync(new URL(path,root))),sum,path);
  }
  const vectors=JSON.parse(readFileSync(new URL("conformance.json",root)));
  assert.equal(vectors.schema_sha256,schemaDigest);
  assert.equal(vectors.version,"1.0.0-candidate.3");
  assert.equal(vectors.profile,"recorded-run-bundle-v1");
  const meta=JSON.parse(readFileSync(new URL("manifest.json",root)));
  assert.equal(meta.schema_sha256,schemaDigest);assert.equal(meta.version,vectors.version);
  assert.equal(meta.artifact,vectors.profile);assert.equal(meta.status,"candidate");
  const caseIndex=JSON.parse(readFileSync(new URL("../../conformance/cases/recorded-run-bundle-v1-candidate3.json",root)));
  assert.equal(caseIndex.profile,vectors.profile);assert.equal(caseIndex.version,vectors.version);
  const tooling=JSON.parse(readFileSync(new URL("tooling.json",root)));
  for(const item of tooling.files) assert.equal(sha256(readFileSync(new URL("../../"+item.path,root))),item.sha256,item.path);
  for(const c of vectors.cases) {
    const bytes=readFileSync(new URL(c.path,root));
    assert.equal(sha256(bytes),c.sha256,c.path);
    if(c.valid) assert.ok(validateBundle(bytes));else assert.throws(()=>validateBundle(bytes));
  }
});
test("JCS emits integer-like keys directly and preserves annotations",()=>{
  assert.equal(canonical({"2":"two","10":"ten",annotations:{z:1,a:2}}),
    '{"10":"ten","2":"two","annotations":{"a":2,"z":1}}');
  assert.equal(canonical({"\uE000":1,"\u{1F600}":2,"a":3}), '{"a":3,"😀":2,"":1}');
});
test("JCS numeric examples and negative zero",()=>{
  assert.equal(canonical([-0,1e30,333333333.33333329,1e-7,0.000001]),"[0,1e+30,333333333.3333333,1e-7,0.000001]");
  assert.throws(()=>canonical(NaN));assert.throws(()=>canonical(Infinity));
});
test("JCS rejects duplicate keys, escaped aliases, lone surrogates and invalid UTF8",()=>{
  for(const s of ['{"x":1,"x":2}','{"x":1,"\\u0078":2}','"\\ud800"','-0','1e30','{}\n','\ufeff{}'])
    assert.throws(()=>parseCanonical(Buffer.from(s)));
  assert.throws(()=>parseCanonical(Buffer.from([0xc0,0xaf])));
});
test("bounded JSON depth, values and record size",()=>{
  assert.throws(()=>parseCanonical(Buffer.from("[".repeat(33)+"0"+"]".repeat(33))));
  assert.throws(()=>canonical(Array(100001).fill(0)));
  const e=entriesFor(fixture());
  assert.throws(()=>validateBundle(repack(e,"records/events.ndjson",Buffer.from('"'+ "x".repeat(65536)+'"\n'))));
});
test("excessive record count rejected before presenting rows",()=>{
  const e=entriesFor(fixture()),line=Buffer.from(canonical(fixture().events[0])+"\n");
  assert.throws(()=>validateBundle(repack(e,"records/events.ndjson",Buffer.concat(Array(25001).fill(line)))));
});
test("archive byte cap and missing end",()=>{
  assert.throws(()=>readZip(Buffer.alloc(LIMITS.archive+1)));
  assert.throws(()=>readZip(bundleFor(fixture()).subarray(0,-1)));
});
test("zip duplicate paths and noncanonical names",()=>{
  const duplicated=[["a.bin",Buffer.from("a")],["a.bin",Buffer.from("b")]];duplicated.size=2;
  assert.throws(()=>readZip(writeZip(duplicated)));
  for(const p of ["/root","../a","a/../b","a\\b","A.bin","a/","é.bin","a.bin\n"])
    assert.throws(()=>readZip(writeZip(new Map([[p,Buffer.from("a")]]))));
});
test("record token strings reject regex end-anchor control aliases",()=>{
  const f=fixture();f.manifest.recording.identity.value+="\n";
  assert.throws(()=>validateBundle(bundleFor(f)));
});
test("reject zip flags, type, offset, central/local discrepancies, CRC",()=>{
  const original=writeZip(new Map([["a.bin",Buffer.from("data")]]));
  const central=original.readUInt32LE(original.length-6);
  const changes=[
    b=>b.writeUInt16LE(8,6), // descriptor
    b=>b.writeUInt16LE(1,central+8), // encrypted
    b=>b.writeUInt32LE(0xa1ff0000,central+38), // symlink
    b=>b.writeUInt32LE(1,central+42), // overlapping/prefix
    b=>b.writeUInt16LE(45,central+6), // ZIP64
    b=>b.writeUInt16LE(1,central+30), // extra
    b=>b.writeUInt16LE(1,central+34), // disk
    b=>b.writeUInt32LE(1,14), // CRC mismatch
    b=>b[30]=98, // local name disagreement
    b=>b[35]^=1 // content corruption
  ];
  for(const change of changes) {const b=Buffer.from(original);change(b);assert.throws(()=>readZip(b));}
});
test("bounded actual inflate rejects undersized declaration on compressible input",()=>{
  const b=writeZip(new Map([["a.bin",Buffer.alloc(2*1024*1024)]]),true);
  const c=b.readUInt32LE(b.length-6);b.writeUInt32LE(1,22);b.writeUInt32LE(1,c+24);
  assert.throws(()=>readZip(b),/zip_inflate_limit_or_data/);
});
test("inflation total cap and advertised per-entry cap",()=>{
  assert.throws(()=>readZip(writeZip(new Map([["a.bin",Buffer.alloc(LIMITS.entry+1)]]),true)));
  const e=new Map(["a.bin","b.bin","c.bin"].map(n=>[n,Buffer.alloc(12*1024*1024)]));
  assert.throws(()=>readZip(writeZip(e,true)));
});
test("NDJSON final LF, blanks, CRLF and later tampering are rejected",()=>{
  for(const b of [Buffer.from("{}"),Buffer.from("\n"),Buffer.from("{}\r\n")])
    assert.throws(()=>validateBundle(repack(entriesFor(fixture()),"records/events.ndjson",b)));
});
