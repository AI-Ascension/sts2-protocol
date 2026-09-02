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
