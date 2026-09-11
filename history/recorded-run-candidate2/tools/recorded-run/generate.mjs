// SPDX-License-Identifier: MIT
// Regenerates only the explicit synthetic candidate artifact directory.
import {mkdirSync, writeFileSync, readFileSync, readdirSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {join} from "node:path";
import {positives, negatives} from "./cases.mjs";
import {fixture, entriesFor} from "./fixtures.mjs";
import {canonical} from "./jcs.mjs";
import {schemaDigest, sha256, validateBundle} from "./validate.mjs";
const root=fileURLToPath(new URL("../../artifacts/recorded-run-bundle-v1/",import.meta.url));
const repository=fileURLToPath(new URL("../../",import.meta.url));
for(const dir of ["golden","invalid"]) mkdirSync(join(root,dir),{recursive:true});
const positive=positives(),negative=negatives(),cases=[];
for(const [name,data] of Object.entries(positive)) {
  validateBundle(data);writeFileSync(join(root,"golden",name+".zip"),data);
  cases.push({path:"golden/"+name+".zip",valid:true,sha256:sha256(data)});
}
for(const [name,data] of Object.entries(negative)) {
  let rejected=false;try {validateBundle(data);}catch {rejected=true;}
  if(!rejected) throw new Error("invalid_vector_accepted");
  writeFileSync(join(root,"invalid",name+".zip"),data);
  cases.push({path:"invalid/"+name+".zip",valid:false,sha256:sha256(data)});
}
const entries=entriesFor(fixture());
for(const [name,data] of entries) {
  if(name==="manifest.json") writeFileSync(join(root,"golden","manifest.json"),data);
  if(name==="reports/omissions.json") writeFileSync(join(root,"golden","omissions.json"),data);
  if(name.endsWith(".ndjson")) writeFileSync(join(root,"golden",name.split("/").at(-1)),data);
}
const caseDocument={profile:"recorded-run-bundle-v1",version:"1.0.0-candidate.2",schema_sha256:schemaDigest,
  fixture_provenance:"synthetic",generator:"tools/recorded-run/generate.mjs",cases};
writeFileSync(join(root,"conformance.json"),JSON.stringify(caseDocument,null,2)+"\n");
const manifest={artifact:"recorded-run-bundle-v1",status:"candidate",version:"1.0.0-candidate.2",
  base_revision:"0bc689eabc5542ede2b09b030d9ea32daa8a73e7",schema_sha256:schemaDigest,
  owner:"sts2-protocol",producer:"sts2-harness",candidate_consumers:["ascension-workflow-studio","ai-agent-observability"],
  admitted_consumers:[],license:"MIT",provenance:"original hand-authored schema and synthetic fixtures"};
writeFileSync(join(root,"manifest.json"),canonical(manifest));
const toolingPaths=readdirSync(join(repository,"tools/recorded-run")).filter(p=>p.endsWith(".mjs"))
  .map(p=>"tools/recorded-run/"+p).concat(["conformance/cases/recorded-run-bundle-v1.json",
    "crates/protocol/tests/recorded_run_conformance.rs","docs/decisions/0034-recorded-run-bundle-candidate.md"]).sort();
writeFileSync(join(root,"tooling.json"),canonical({node:"24.16.0",
  files:toolingPaths.map(path=>({path,sha256:sha256(readFileSync(join(repository,path)))}))}));
const paths=["README.md","schema.json","manifest.json","conformance.json","tooling.json",
  ...["golden","invalid"].flatMap(dir=>readdirSync(join(root,dir)).map(n=>dir+"/"+n))].sort();
writeFileSync(join(root,"SHA256SUMS"),paths.map(p=>sha256(readFileSync(join(root,p)))+"  "+p+"\n").join(""));
console.log(JSON.stringify({schema_sha256:schemaDigest,inventory_sha256:sha256(readFileSync(join(root,"SHA256SUMS"))),
  valid_cases:Object.keys(positive).length,invalid_cases:Object.keys(negative).length}));
