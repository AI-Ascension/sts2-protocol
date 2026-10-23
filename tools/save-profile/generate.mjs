// SPDX-License-Identifier: MIT
// Original synthetic contract authoring; no host or consumer implementation inputs.
import { mkdirSync, writeFileSync, readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
export const hash = x => createHash('sha256').update(x).digest('hex');
export const canonical = x => JSON.stringify(x, (_, v) => v && !Array.isArray(v) && typeof v === 'object' ? Object.fromEntries(Object.entries(v).sort(([a], [b]) => a.localeCompare(b, 'en'))) : v);
const root = 'artifacts/save-profile-v1';
const ref = name => ({ $ref: `#/$defs/${name}` });
const object = properties => ({ type: 'object', additionalProperties: false, required: Object.keys(properties), properties });
const nullable = x => ({ anyOf: [x, { type: 'null' }] });
const token = { type: 'string', pattern: '^[a-zA-Z0-9][a-zA-Z0-9._:-]{0,95}$' };
const number = { type: 'integer', minimum: 0, maximum: 9007199254740991 };
const digest = { type: 'string', pattern: '^[a-f0-9]{64}$' };
const owner = object({ instance_id: token, approved_process_profile: { ...number, minimum: 1 }, process_namespace: token, allocation_id: token, process_generation: number, mod_incarnation: token, mod_generation: number });
const intent = object({ operation_id: token, action: { enum: ['select', 'create_disposable'] }, scope: { enum: ['select_profile', 'create_disposable_profile'] }, owner: ref('owner'), original_lease_id: token, expected_generation: number, baseline: nullable(digest), slot_id: token, prior_allocation_id: nullable(token) });
const observation = object({ owner: ref('owner'), generation: number, baseline: nullable(digest), selected_slot_id: nullable(token), slots: { type: 'array', maxItems: 32, uniqueItems: true, items: token }, native_ready: { type: 'boolean' }, active_run: { type: 'boolean' } });
const witness = object({ operation_id: token, intent_digest: digest, slot_id: token, allocation_id: token, generation: number });
const common = { protocol_version: { const: 'save-profile-v1' }, schema_digest: digest };
const message = (kind, fields) => object({ ...common, kind: { const: kind }, ...fields });
const operation = { intent: ref('intent'), intent_digest: digest };
const schema = { $schema: 'https://json-schema.org/draft/2020-12/schema', $id: 'sts2-save-profile-v1', title: 'Inert save-profile-v1 candidate', oneOf: [
  message('discover_request', { owner: ref('owner'), current_lease_id: token }),
  message('discover_response', { observation: ref('observation') }),
  message('operation_request', operation),
  message('lookup_request', { ...operation, current_lease_id: token }),
  message('operation_result', { ...operation, current_lease_id: token, status: { enum: ['accepted', 'settled', 'rejected', 'cancelled', 'blocked', 'unknown'] }, allocation_stage: { enum: ['reserved', 'directory_created', 'native_ready'] }, observation: nullable(ref('observation')), witness: nullable(ref('witness')), reason: nullable({ enum: ['active_run', 'stale_baseline', 'unsupported_slot', 'unknown_directory', 'denied', 'conflict', 'unavailable', 'not_applied', 'uncertain'] }) }),
  message('lookup_result', { ...operation, current_lease_id: token, outcome: { enum: ['not_found', 'request_error', 'uncertain'] } }),
], $defs: { owner, intent, observation, witness } };
mkdirSync(`${root}/golden`, { recursive: true });
const pretty = x => JSON.stringify(x, null, 2) + '\n';
writeFileSync('schemas/save-profile-v1.schema.json', pretty(schema));
writeFileSync(`${root}/schema.json`, pretty(schema));
const schema_digest = hash(pretty(schema));
const own = { instance_id: 'synthetic-instance', approved_process_profile: 2, process_namespace: 'automation-isolated', allocation_id: 'allocation-old', process_generation: 3, mod_incarnation: 'mod-synthetic-a', mod_generation: 1 };
const make = (kind, fields) => ({ protocol_version: 'save-profile-v1', schema_digest, kind, ...fields });
const select = { operation_id: 'operation-select', action: 'select', scope: 'select_profile', owner: own, original_lease_id: 'lease-original', expected_generation: 4, baseline: 'a'.repeat(64), slot_id: 'slot-two', prior_allocation_id: null };
const create = { ...select, operation_id: 'operation-create', action: 'create_disposable', scope: 'create_disposable_profile', owner: { ...own, allocation_id: 'allocation-new', process_generation: 4, mod_incarnation: 'mod-synthetic-b' }, baseline: null, prior_allocation_id: 'allocation-old' };
const digestIntent = intent => hash('sts2-save-profile-intent-v1\n' + canonical(intent));
const entries = {};
entries['discover-request'] = make('discover_request', { owner: own, current_lease_id: 'lease-original' });
entries['discover-empty'] = make('discover_response', { observation: { owner: own, generation: 4, baseline: null, selected_slot_id: null, slots: [], native_ready: false, active_run: false } });
for (const intent of [select, create]) {
  const prefix = intent.action;
  const op = { intent, intent_digest: digestIntent(intent) };
  entries[`${prefix}-request`] = make('operation_request', op);
  entries[`${prefix}-lookup`] = make('lookup_request', { ...op, current_lease_id: 'lease-rotated' });
  for (const status of ['accepted', 'settled', 'rejected', 'cancelled', 'blocked', 'unknown']) {
    const settled = status === 'settled';
    const observation = { owner: intent.owner, generation: 5, baseline: 'b'.repeat(64), selected_slot_id: intent.slot_id, slots: [intent.slot_id], native_ready: true, active_run: false };
    entries[`${prefix}-${status}`] = make('operation_result', { ...op, current_lease_id: 'lease-original', status, allocation_stage: settled ? 'native_ready' : 'directory_created', observation: settled ? observation : null, witness: settled ? { operation_id: intent.operation_id, intent_digest: op.intent_digest, slot_id: intent.slot_id, allocation_id: intent.owner.allocation_id, generation: 5 } : null, reason: ({ rejected: 'denied', cancelled: 'not_applied', blocked: 'unavailable', unknown: 'uncertain' })[status] ?? null });
  }
  for (const outcome of ['not_found', 'request_error', 'uncertain']) entries[`${prefix}-${outcome}`] = make('lookup_result', { ...op, current_lease_id: 'lease-rotated', outcome });
}
for (const [name, value] of Object.entries(entries)) writeFileSync(`${root}/golden/${name}.json`, canonical(value));
const manifest = { artifact: 'sts2-protocol/save-profile-v1', owner: 'sts2-protocol', status: 'candidate', consumers: [], prospective_consumers: ['sts2-game-mod', 'sts2-gateway', 'sts2-harness', 'sts2-mcp-server'], schema_digest, provenance: { license: 'MIT', generator: 'tools/save-profile/generate.mjs', source: 'original-synthetic', requirement: 'AI-Ascension/sts2-protocol#97' }, golden_files: Object.keys(entries).sort().map(x => `golden/${x}.json`) };
writeFileSync(`${root}/manifest.json`, pretty(manifest));
writeFileSync(`${root}/intent-hashes.json`, pretty({ select: digestIntent(select), create_disposable: digestIntent(create) }));
const inventory = [...readdirSync(root).filter(x => x !== 'SHA256SUMS' && x !== 'golden'), ...manifest.golden_files, '../../schemas/save-profile-v1.schema.json', '../../conformance/cases/save-profile-v1.json'].sort();
writeFileSync(`${root}/SHA256SUMS`, inventory.map(path => `${hash(readFileSync(`${root}/${path}`))}  ${path}\n`).join(''));
console.log(schema_digest);
