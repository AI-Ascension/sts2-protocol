# ADR 0033: accept the native co-op source component contract

- Status: Accepted for component integration; live native gate pending
- Date: 2026-09-10

## Context

The source owner now emits a closed `coop-native-v1` envelope with a host-backed legal catalog,
local action and shared vote requests, peer rejoin, effects, operation receipts, and same-operation
recovery. The producer was rebuilt against the exact checked-in schema and freshly captured with a
synthetic port. Gateway, MCP, and harness remain separate owners of routing, framing, and
coordination.

## Decision

Accept `sts2.protocol/coop-native-v1` as a component contract at schema digest
`2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629`. The artifact registers
`sts2-game-mod`, `sts2-gateway`, `sts2-mcp-server`, and `sts2-harness` as named boundaries and
binds the exact source commit/tree, producer declaration, capture projections, and strict golden
vectors. Each consumer copies and validates the artifact at its own boundary; no consumer imports
protocol implementation code.

The envelope is strict UTF-8 JSON with explicit nullable members, closed nested objects, unique
object keys, bounded identities and generations, and operation identity retained across unknown
outcomes. The host owns legality and settlement, the gateway owns leases and routing, MCP owns
framing and projection, and the harness owns coordination and provider decisions. The catalog is a
read path and does not grant mutation authority.

## Compatibility and live gate

This is an additive component profile relative to existing protocol artifacts and a replacement for
the earlier unadmitted native co-op candidate bytes. A consumer must reject an unsupported digest,
profile, or envelope shape before any mutation. The component status does not claim a live native
session. `live_status` remains `unverified` until a disposable two-peer STS2 run proves distinct
peer identities, host/client loading, model action settlement, supported event and treasure/relic
vote settlement, matching native checksums, disconnect/rejoin recovery, and peer convergence.
