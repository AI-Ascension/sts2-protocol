# ADR 0002: Protocol Ownership and Dependency Direction

- Status: Accepted for foundation and future contract admission
- Date: 2026-09-02

## Context

The STS2 system has separate game, host, gateway, MCP, and experiment boundaries. Runtime message
arrows do not imply compile-time dependencies, and repeated names do not establish a shared semantic
owner. The protocol target needs an explicit boundary before any package is initialized.

## Decision

`sts2-protocol` owns only inert, shared, language-neutral, transport-neutral contract artifacts. Its
future consumers are named as follows:

| Consumer | Allowed use |
| --- | --- |
| `sts2-game-core` | Neutral identity/version or other accepted shared metadata; game semantics remain core-owned |
| `sts2-gateway` | Accepted neutral lifecycle/correlation metadata; lifecycle authority remains gateway-owned |
| `sts2-mcp-server` | Accepted neutral wire/mapping metadata; MCP framing and catalog remain MCP-owned |
| `sts2-harness` | Accepted neutral experiment/correlation metadata; runs and artifacts remain harness-owned |

`sts2-game-mod` remains authoritative for host objects, main-thread access, game HTTP, and mutation
settlement. It is not a direct protocol consumer by default. The compile-time graph points inward to
the protocol artifact only after a contract admission decision; no protocol crate depends on any
consumer or implementation.

The runtime graph remains:

```text
sts2-harness -> sts2-mcp-server -> sts2-gateway -> sts2-game-mod -> game host
```

Gateway owns lifecycle/routing, MCP is a thin adapter, and harness owns coordination/experiments and
artifact lineage. Authentication, authorization, persistence, provider decisions, and mutation
authority remain at their owning boundaries.

## Admission and consequences

Each exported item needs one canonical source, two or more named consumers, version/profile and digest
policy, explicit serialization and unknown-value rules, provenance/license, and a deterministic
conformance case. Missing evidence blocks that item without making the whole accepted target
not-applicable. A future path dependency must be reproducible from the intended repository boundary
and must not create a cycle.

This decision keeps the current wave behavior-free. It also means owner repositories may continue to
publish owner-local schemas until a row-by-row transfer is accepted.
