# ADR 0007: Bounded card-play proposal and competing Runtime-v3 profile

Status: proposed; unreleased; not a compatibility or live-host approval.

## Scope and ownership

This branch proposes one bounded `play_card` contract. The protocol owns its inert wire
representation, schema, deterministic canonical JSON and conformance bundle. Core and the host
boundary own legality; game-mod owns authoritative mutation, receipts and settlement. Named artifact
consumers are core, game-mod, gateway, MCP and harness. No cross-repository Rust dependency is added.

The profile is `runtime-v3-gameplay`, artifact `sts2-protocol/runtime-v3-gameplay`, with schema digest
`c961bbde893f0422f80233d14ea9ae8b648ee9032136e5370aa5f6b949f6575e`. It adds a distinct profile to
frozen runtime-v2; it does not alter runtime-v2 messages. Correlation is request-scoped; instance,
session, lease and epoch fence the operation lifetime. Generation binds observation and action
admission. Operation identity remains stable for receipt replay/reconciliation within that scope.
Card positions and target identities are references into the caller's generation, not durable host
objects or globally reusable gameplay identifiers.

## Serialization and semantic validation

The normative schema requires explicit nullable members; omission is invalid. Unknown members and
enum values are rejected. UTF-8 compact JSON goldens define field order and null representation.
Bounds and identifier syntax are explicit in the schema. Authoritative typed validation additionally
checks exact metadata/digest, unique target identities, envelope/observation generation equality,
and settled witness generation/card/target correspondence with the recorded receipt action.

The parser-only safety corrections reject omitted required nullable fields and mismatched settled
witnesses that were previously accepted by Rust. Canonical valid bytes and the schema digest do not
change. Schema validation alone does not establish cross-field equalities; non-Rust consumers must
apply those semantic checks. A receipt's internal consistency cannot prove host completion or
freshness relative to a previous observation. The owning consumer must retain the admission
observation, verify an independent operation-bound host effect and preserve accepted, settled,
rejected, unknown and cancelled outcomes. Unknown must not trigger blind mutation retry; cancellation
requires no-mutation evidence. Replay and conflict detection are ledger obligations, not parser
behavior. Current goldens demonstrate a generation4 to5 collection/energy transition, not a live
host execution or general proof of those obligations.

## Competing proposal and migration gate

[PR #8](https://github.com/AI-Ascension/sts2-protocol/pull/8) proposes an incompatible semantic
catalog/lifecycle contract under the same profile, artifact path and schema identifier. At
`18a8bd8da3857c826799261054e4a6a9893d7aa9` its schema digest is
`fbfb18279b0c7ebb350ef0ce0d56547fa11e83985b13380cb2b0f1dba4cb56e9`. It replaces positional
`action_request`/`reconcile_request` with catalog-bound dispatch, wait and recovery messages and a
different observation/witness model. These proposals cannot be treated as additive, or admitted
based on matching profile names alone. Exact digest validation must reject the unsupported variant
before mutation. Neither branch is an approved published release.

Before either profile is integrated, the organization must choose and document distinct identity or
explicit replacement, pin producer/consumer revisions and digests, and validate complete consumer
compatibility. Replacing one branch's files during conflict resolution is a breaking migration,
not compatibility evidence. Preserve operation identity, retry/conflict rules, uncertain recovery,
independent settlement and cancellation no-mutation requirements in the selected contract.

## Provenance and verification

Schema, fixtures and Rust mapping are original hand-authored project material under MIT; no game
source or proprietary artifacts are included. The release-like bundle is not a publication.
`runtime_v3_gameplay_conformance.rs` checks canonical fixtures and artifact agreement;
`runtime_v3_gameplay_validation.rs` checks required nullability, witness correspondence and numeric
bounds. Standard repository checks apply. Host, transport, consumer integration and live gameplay
remain unverified.
