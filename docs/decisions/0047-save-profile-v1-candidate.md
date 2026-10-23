# ADR 0047: Independent save-profile-v1 candidate

- Status: candidate; no consumer adoption or runtime claim
- Requirement: AI-Ascension/sts2-protocol#97
- Canonical artifact owner: sts2-protocol
- Prospective consumers: sts2-game-mod, sts2-gateway, sts2-mcp-server, sts2-harness
- Compatibility: independent additive profile; every existing artifact stays unchanged
- Provenance: original MIT schema, implementation and synthetic fixtures, no host captures

## Decision and authority

Publish an inert closed `save-profile-v1` candidate. Protocol owns representation and conformance
only. Mod owns logical slot support, authoritative baseline and active-run readback, host-thread
selection and settlement. Gateway owns approved numeric process profile/namespace, retained
disposable allocation, directory safety, process generations, admission and leases. MCP adapts;
Harness coordinates and validates consumer evidence. No transport, route, credential, filesystem,
process, persistence, game rules, host mutation or Harness validation authority is transferred.

The fixed `automation-disposable-v1` alias is not an approved numeric process profile or namespace.
An allocated directory is not a Mod logical save slot. There are no paths or save payloads on this
wire. Gateway must reject unknown existing directories rather than adopt or overwrite them.

## Identity, namespace and lifetime

The owner tuple contains independently scoped `instance_id`, positive safe integer
`approved_process_profile`, `process_namespace`, `allocation_id`, `process_generation`,
`mod_incarnation` and `mod_generation`. None substitutes for another. Allocation IDs are retained
Gateway ledger identities, never filesystem names. Process generations and Mod incarnation/generation
must match exactly; observation generation increases only within that exact owner. Restart is not a
new observation of the old owner. Slot IDs are Mod logical identities within that owner. Baselines
are opaque SHA-256 content identities supplied by Mod, not a hash computed from a host path here.

Discovery requests contain only owner and current lease, not a baseline. Unready discovery can
return null baseline/selection and an empty slot inventory. Native-ready observation requires a
baseline and selected slot in its bounded unique catalog. Active-run state is explicit.

The original intent retains operation ID, permission scope, owner, original lease, expected
observation generation, baseline, selected slot and action. Selection needs a baseline; disposable
creation has no pre-existing baseline and carries a prior allocation distinct from its newly bound
owner allocation. Gateway first reserves and retains that new allocation under the original operation
identity, and binds its exact process/Mod owner before this creation intent is admitted. Reservation
alone, allocation directory creation and process launch are not native profile readiness. Protocol
cannot implement or authenticate that reservation or binding. Absent an exact binding, the consumer
must remain blocked rather than invent identities or claim a creation request was admitted.

## Settlement and recovery

`accepted`, `settled`, `rejected`, `cancelled`, `blocked`, and `unknown` are distinct. Cancellation
requires `not_applied` owner evidence. Only settled results have observation and witness. Selection
and creation settle only with a strictly newer native-ready, inactive-run observation from the exact
admitted owner, the exact selected slot and a witness naming the original operation/intent digest,
allocation, slot and observed generation. A newer unrelated observation is insufficient. Directory
reservation or creation can appear in nonsettled results but never proves native readiness.

`bind_operation` compares complete retained intent and digest to a recovery request or result;
isolated message validation cannot establish origin. Lookup carries a separately current lease,
allowing read-only reconciliation after lease rotation while preserving original lease/scope/intent.
`bind_lookup` additionally joins both found receipts and lookup failures to that lookup's current
lease. Operation results explicitly carry `current_lease_id` (the original lease for an immediate
receipt; the current read lease for a recovered receipt). Lookup failures have explicit `not_found`,
`request_error`, or `uncertain` outcomes. None authorizes a new operation, mutation or allocation ID.
An owner ledger must retain unresolved identity across restart/lost reply and enforce idempotency;
this package supplies no ledger. A current lease is metadata, not authentication or authorization.

## Encoding and bounds

Every envelope names the exact profile and standalone schema digest. All objects are closed,
all members required (nullable only where specified), unknown enums and duplicate decoded keys
are rejected. Tokens are ASCII, 1–96 characters, without paths. Catalogs contain at most 32 slots.
Messages are at most 65,536 UTF-8 bytes; nesting is closed by schema and parser recursion limits.
Integers are exact mathematical values in 0–2^53−1 (process profile starts at 1). Raw integral
exponent/decimal spellings normalize exactly before JSON number parsing; fractional or unsafe
values reject without floating-point rounding.

Canonical JSON is compact UTF-8, recursively ASCII-lexicographic object key order, preserved array
order, integer decimal spelling and no final newline. Intent digest is lowercase SHA-256 over
UTF-8 `sts2-save-profile-intent-v1\n` followed by the canonical **complete intent**. It includes
original lease, operation and scope; excludes the separately current lookup lease and envelope.
Fixed hashes are independently checked by Rust and Node, not regenerated expected values at test time.

## Verification and remaining migration

The Rust conformance gate validates schema copy, all canonical goldens, complete checksums, closure,
raw duplicate/exact-number faults, owner joins, strictly newer readback, preparation-vs-settlement,
original operation recovery and fixed hashes. The independent Node witness also runs in hosted CI.
`tools/save-profile/generate.mjs` regenerates schema and synthetic artifact; inspect all digest changes.
These tests are synthetic candidate evidence, not authenticated authoritative native evidence.

Consumer adoption requires exact-digest agreement, serialized producer/consumer fixtures and
coordinated capability negotiation. Gateway #51 must implement durable allocation/operation identity,
approved numeric process configuration, safe directory handling, fenced routing and lease recovery.
Mod #78 must implement real host-thread operations and authoritative active-run/profile readback,
stale-baseline checks and preservation of all existing profiles. MCP needs negotiated mappings;
Harness #102 needs setup/readback, retained-operation recovery and validation. Studio must keep
save slots distinct from provider/workflow profiles. Authorized disposable native tests must prove
preservation on failed operations and recovery after loss/restart. Neither this component nor its
merge closes these original feature trackers or claims release/native/integration compatibility.
