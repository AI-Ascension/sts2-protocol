# ADR 0006: Frozen `runtime-v2` gameplay-operation contract

- Status: Accepted for the protocol artifact; consumer and live gameplay compatibility remain unverified
- Date: 2026-09-02

## Context

The `runtime-v1` profile proves a bounded host-visible probe, but it cannot distinguish admission from
authoritative gameplay settlement or represent uncertainty after a timeout. The next vertical slice
needs one narrow, language-neutral contract that downstream owners can map independently while keeping
host authority, game rules, routing, and coordination outside this repository.

## Decision

`sts2-protocol` owns a separate `runtime-v2` profile under
[`schemas/runtime-v2.schema.json`](../../schemas/runtime-v2.schema.json) and
[`artifacts/runtime-v2/`](../../artifacts/runtime-v2/). Runtime-v1 is unchanged and remains a
separate compatibility profile. The named runtime consumers are `sts2-game-mod`, `sts2-gateway`,
`sts2-harness`, and `sts2-mcp-server`; `sts2-game-core` remains the owner of domain transition
semantics rather than a protocol implementation dependency.

Every message preserves protocol version, exact schema digest, inert provenance, correlation ID,
instance ID, session ID, lease ID, lease epoch, and generation. Action and receipt messages carry a
stable `operation_id`. The only action is `{ "action_id": "end_turn" }`. Observations are bounded to
`combat_phase`, `turn_index`, `host_ready`, and `generation`; the legal phase is `combat/player_turn`.

The implementation-neutral transition vector is generation 4/turn 2 in `combat/player_turn` to
generation 5/turn 3 in the same phase. A settled receipt requires a fresh observation and a
`turn_end_settled` witness. The outcome set is exactly `accepted`, `settled`, `rejected`, `unknown`,
and `cancelled`:

- `accepted` means admission only and carries no settlement witness.
- `settled` is authoritative only with the fresh observation and matching witness.
- `rejected` means no mutation was admitted.
- `unknown` is required when timeout, disconnect, or restart leaves mutation uncertain; it must be
  reconciled with the same operation identity rather than blindly retried.
- `cancelled` is valid only before mutation or with explicit no-mutation confirmation.

An identical request with an existing operation identity replays the original receipt without a second
mutation. Reusing that identity for a conflicting request returns `idempotency_conflict`.

## Consequences and evidence

The release-like artifact contains a byte-identical schema copy, manifest, digest inventory, and
sanitized goldens for legal, stale-generation, outside-combat, enemy-turn, duplicate, idempotency
conflict, cancellation, timeout-to-unknown, and successful-reconciliation cases. The conformance case
binds the frozen transition and lifecycle assertions without depending on a language or transport.

The protocol owns representation, bounds, versioning, provenance, and conformance only. It does not
implement game legality, host calls, main-thread dispatch, queueing, leases, HTTP/MCP routes,
operation storage, cancellation, retry policy, or reconciliation authority. Local schema,
serialization, and policy evidence is `confirmed`; downstream mapping, host settlement, and live
gameplay compatibility are `unverified`.
