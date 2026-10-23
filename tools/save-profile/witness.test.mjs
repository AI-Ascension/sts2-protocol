// SPDX-License-Identifier: MIT
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
const base = 'artifacts/save-profile-v1/';
const read = path => readFileSync(base + path);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
// Independent encoder: no generator imports; UTF-16 ordering is ASCII ordering for closed keys.
function ordered(value) {
  if (Array.isArray(value)) return '[' + value.map(ordered).join(',') + ']';
  if (value && typeof value === 'object') return '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + ordered(value[key])).join(',') + '}';
  return JSON.stringify(value);
}
const fixed = {
  select: '3ddec2a15f478c9d8039817e083353b1e7155adde07615024b57261e9147ad28',
  create_disposable: 'b70af704f717e83f6bb06ef8fbdba072308bdb65dc8885daa9dcf89089d6c57a',
};
test('independent domain-separated intent preimages and fixed hashes', () => {
  for (const [action, hash] of Object.entries(fixed)) {
    const message = JSON.parse(read(`golden/${action}-request.json`));
    const preimage = 'sts2-save-profile-intent-v1\n' + ordered(message.intent);
    assert.equal(sha(preimage), hash);
    assert.equal(message.intent_digest, hash);
    const changed = structuredClone(message.intent);
    changed.original_lease_id = 'lease-rotated';
    assert.notEqual(sha('sts2-save-profile-intent-v1\n' + ordered(changed)), hash);
  }
  assert.deepEqual(JSON.parse(read('intent-hashes.json')), fixed);
});
test('canonical goldens, exact integer source numbers, schema identity and complete checksums', () => {
  const manifest = JSON.parse(read('manifest.json'));
  assert.equal(manifest.schema_digest, sha(read('schema.json')));
  assert.deepEqual(read('schema.json'), readFileSync('schemas/save-profile-v1.schema.json'));
  assert.deepEqual(manifest.consumers, []);
  const listed = new Set();
  for (const line of read('SHA256SUMS').toString().trim().split('\n')) {
    const [expected, path] = line.split('  ');
    assert(!listed.has(path));
    listed.add(path);
    assert.equal(sha(read(path)), expected, path);
  }
  for (const file of readdirSync(base)) if (file !== 'golden' && file !== 'SHA256SUMS') assert(listed.has(file));
  assert.deepEqual(manifest.golden_files, readdirSync(base + 'golden').sort().map(x => 'golden/' + x));
  for (const path of manifest.golden_files) {
    assert(listed.has(path));
    const bytes = read(path).toString();
    const value = JSON.parse(bytes, (_, v, context) => {
      if (typeof v === 'number') {
        assert(Number.isSafeInteger(v));
        assert.equal(BigInt(context.source), BigInt(v));
      }
      return v;
    });
    assert.equal(bytes, ordered(value));
  }
});
