# Architecture

## Purpose and status

`sts2-protocol` is an inert contract-artifact boundary. It may publish a shared language-neutral,
transport-neutral description only when the item has a canonical owner, at least two named consumers,
an explicit compatibility policy, and deterministic conformance. The initial package is metadata-only;
it has no runtime or transport behavior.

## Ownership map

| Boundary | Owns | Does not own |
| --- | --- | --- |
| `sts2-protocol` | Accepted neutral schemas, shared metadata, provenance, and conformance descriptions | Game rules, host objects, transports, processes, lifecycle authority, tools, models, providers |
| `sts2-game-core` | Host-independent game/domain semantics and legality | HTTP, MCP, host, process, filesystem, provider, and training behavior |
| `sts2-game-mod` | Host loader/ABI, main-thread access, authoritative game HTTP, and mutation settlement | MCP framing, gateway admission, model artifacts, and neutral ownership decisions |
| `sts2-gateway` | Instance lifecycle, leases, routing, admission, health, and gateway authorization | Game rules, host state, MCP semantics, and trajectories |
| `sts2-mcp-server` | MCP framing, catalog, session/request lifecycle, and mapping | Host objects, game authority, gateway leases, and model/provider policy |
| `sts2-harness` | Coordination, runs, experiments, trajectories, scoring, providers, and artifacts | Host access, game authority, gateway lifecycle, and a second MCP server |

The protocol target is not an authority merely because a value is shared in conversation. The owner
of meaning remains the boundary that defines the behavior. A protocol artifact may describe a stable
projection without moving that authority.

## Separate graphs

Runtime communication is distinct from compile-time consumption:

```text
Runtime:
sts2-harness -> sts2-mcp-server -> sts2-gateway -> sts2-game-mod -> game host

Compile-time or artifact consumption, only after acceptance:
sts2-game-core  -> released poc-v1 artifact
sts2-game-mod   -> released poc-v1 artifact
sts2-gateway    -> released poc-v1 artifact
sts2-mcp-server -> released poc-v1 artifact
sts2-harness    -> released poc-v1 artifact
```

The game-mod does not receive host authority from this repository. It remains the host boundary and
may consume the release-like POC artifact for metadata. Runtime message arrows do not authorize Cargo
path dependencies.

## Initial package seam

`crates/protocol` owns the typed `poc-v1` message mapping and its compact JSON serialization. The
schema under `schemas/poc-v1.schema.json` is normative source; `artifacts/poc-v1/` is a checked-in
release-like copy consumed by the five target PRs. `poc_conformance.rs` verifies source/artifact
equivalence, golden bytes, invalid input, metadata, and named consumers. The package has no
cross-repository path dependency.

## Boundary invariants

- Protocol artifacts contain no sockets, HTTP/MCP types, host references, processes, filesystem access,
  clock access, provider calls, credentials, or mutation authority.
- A shared item has one normative source; consumer mappings are local and consume only release-like
  artifact files, never protocol implementation internals.
- Authentication and authorization remain with the boundary making the security decision.
- Owner-local lifecycle, error, timing, privacy, and domain semantics are not silently generalized.
- Unknown, stale, rejected, cancelled, and uncertain outcomes remain distinguishable where a shared
  representation is accepted.
- Cycles and outward dependencies into implementation boundaries are prohibited.

## Change control

An ownership or dependency change requires a decision record. A public contract change requires a
requirement, version/profile classification, canonical serialization rule, golden fixture, conformance
case, migration note, and provenance review. If the consumer or owner evidence is incomplete, record
the item as blocked and leave the protocol package unchanged.

## Runtime profile and authority

ADR 0005 admits `runtime-v1` as a second, bounded neutral artifact. Its source is
[`schemas/runtime-v1.schema.json`](../schemas/runtime-v1.schema.json); the release-like copy is
[`artifacts/runtime-v1/`](../artifacts/runtime-v1/). The profile carries independent instance,
session, lease, epoch, correlation, and generation values, plus a bounded host observation and one
fixed host-visible action. Unknown fields are rejected by the schema, and the checked-in fixtures
bind canonical serialization, provenance, digest, accepted witness, and stale rejection.

The protocol describes data; it does not authenticate callers, issue leases, access the host, choose
the main thread, or settle game rules. The mod remains the host authority, the gateway remains the
lease/fence and route authority, MCP remains the adapter, and the harness remains the coordinator.
The runtime profile is contract-confirmed locally; live execution and gameplay semantics are
unverified.

## Runtime-v2 operation profile

ADR 0006 admits `runtime-v2` as a separate inert artifact for one bounded `end_turn` operation. Its
observation is limited to `combat_phase`, `turn_index`, `host_ready`, and `generation`; the frozen
domain vector permits the action only in `combat/player_turn` and describes generation 4/turn 2 to
generation 5/turn 3. An authoritative settlement carries the `turn_end_settled` witness.

The profile keeps `accepted`, `settled`, `rejected`, `unknown`, and `cancelled` distinct. A stable
`operation_id` identifies the mutation attempt: an identical request replays its receipt, a
conflicting reuse returns `idempotency_conflict`, and an uncertain result is reconciled by that same
identity without a blind retry. The schema and fixtures describe these facts; they do not implement
the ledger, queue, lease checks, host call, cancellation mechanism, or reconciliation storage.

## Seeded-run launch profile

ADR 0015 admits `seeded-run-v1` as a separate neutral artifact for an explicitly seeded launch. The
harness chooses the requested seed, but only the game host can establish the canonical seed and the
`run_started` effect. Every start request and lifecycle result carries a bounded `selected_context`
manifest for standard mode, Ironclad, ascension, modifiers, ordered acts, selection policy, profile
baseline, save policy, and separate game/mod compatibility identities. Its `context_digest` is
content-addressed using the profile's specified compact canonical JSON member order.

The protocol owns validation, serialization, and digest binding only. `accepted` is admission;
`settled` requires a fresh host observation and `run_started` witness; `unknown` requires read-only
reconciliation by the same operation identity. The profile does not select a character, access a
save, dispatch a host call, decide game legality, or establish consumer or live-host compatibility.

## Runtime-v3 fair-play gameplay profile

ADR 0007 admits `runtime-v3-gameplay` as the next neutral profile. It describes ordinary
player-visible state and a complete host-generated typed `LegalAction` catalog for setup, map,
combat, rewards, shop, events, rest, selections, victory, defeat, and recovery. The profile is a
data boundary, not an authority: the game-mod owns extraction, legality, host-thread mutation, and
effect settlement; the gateway owns instance and lease lifecycle; MCP maps bounded tools; and the
harness coordinates Exo decisions and recovery.

The profile makes privileged data structurally absent. A visible seed is text only; it is not a
source of future outcomes. A dispatch request is bound to state ID, generation, operation identity,
and one typed action. `accepted`, `settled`, `rejected`, `unknown`, and `cancelled` remain distinct;
settlement needs a fresh observation, legal-action catalog, and transition witness. Unknown results
must be reconciled or recovered before another mutation. The exact schema digest and artifact
provenance are required before a consumer maps the profile.

## Coordinator-reported synchronization

ADR 0013 defines the separate `coop-synchronization-v1` artifact. Gateway owns configured
peer membership, authenticated coordinator reports, generation convergence, report expiry,
and lease fencing. MCP consumes the complete closed response through its selected read-only
profile. Protocol owns only its metadata, schema, canonical serialization and relationships.
The required source label distinguishes these reports from native-host or independently
authenticated peer evidence. No action, vote, effect, or mutation predicate is exported.

## Candidate Runtime-v4 rest-action profile

ADR 0032 initializes `runtime-v4-expert-rest-action-v1` as a separate candidate profile. The
protocol target owns its inert schema, artifact copy, provenance, checksums, synthetic goldens,
and semantic conformance vectors. Its typed rest-option and selector messages remain separate from
the existing potion action profile, whose bytes and digest are preserved. The profile carries no
HTTP, host, lifecycle, authentication, persistence, or mutation implementation.

The prospective consumer chain is `sts2-game-mod` as native serialized producer,
`sts2-gateway` as route and lease forwarder, `sts2-mcp-server` as thin mapping adapter, and
`sts2-harness` as artifact-validating coordinator. The candidate manifest claims no consumers until
those owners provide exact-digest source and round-trip evidence. A protocol test pass establishes
candidate contract closure only; it does not promote the profile or establish live compatibility.

## Native co-op component contract

ADR 0033 accepts the separate `coop-native-v1` component contract. It is an inert envelope for
native-host observations, a generation-bound legal catalog, local actions, shared votes, peer
rejoin, effects, receipts, and same-operation recovery. The artifact binds schema digest
`2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629` and registers the game-mod,
gateway, MCP, and harness as boundary consumers.

The game-mod owns native host identity, legality, thread affinity, and settlement. Gateway owns
leases and routing, MCP owns framing and projection, and harness owns coordination and provider
decisions. The catalog is read-only and does not authorize a mutation. Component acceptance covers
source and serialized boundary conformance; a live two-peer native session, model action and vote
settlement, native checksum agreement, and disconnect/rejoin convergence remain a separate pending
gate.
