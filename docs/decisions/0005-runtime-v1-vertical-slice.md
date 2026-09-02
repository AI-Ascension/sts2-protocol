# ADR 0005: Bounded `runtime-v1` vertical-slice contract

- Status: Accepted for the implementation sprint; live host and release compatibility remain unverified
- Date: 2026-09-02

## Context

The `poc-v1` artifact proves only deterministic in-memory mappings. The next sprint needs one
cross-repository message family that can carry a safe host-visible probe while preserving the
existing ownership boundaries. A shared contract must not become a transport, host, gateway, MCP,
or harness implementation.

## Decision

`sts2-protocol` owns the language-neutral, transport-neutral `runtime-v1` artifact under
`schemas/runtime-v1.schema.json` and `artifacts/runtime-v1/`. Its named consumers are
`sts2-game-mod`, `sts2-gateway`, `sts2-harness`, and `sts2-mcp-server`.

The profile carries protocol metadata, provenance, correlation, instance/session/lease identity,
lease epoch, and bounded generation values. It defines state and action request/response shapes.
The only admitted action is `show_runtime_probe`; an accepted action must include a fresh
`status_overlay_visible` witness, while a stale generation is rejected with a stable error identity.
Unknown fields, unsupported provenance, out-of-bound values, and unsupported action identities are
invalid. The canonical schema digest and golden bytes are checked by deterministic conformance.

This contract does not issue credentials or leases, open sockets, access host objects, select a
thread, define game rules, or settle mutations. The mod owns host authority, the gateway owns
authentication/routing/fencing, MCP owns its mapping, and the harness owns coordination.

## Consequences and evidence

Each consumer copies the release-like artifact and keeps its implementation dependency-local. The
protocol repository can confirm schema and serialization properties without claiming that any
consumer has run. The safe host-visible probe is intentionally weaker than a gameplay action; real
game-state mutation and semantic effect settlement remain a later sprint.

The schema, manifest, fixtures, and conformance case are `confirmed` in this repository. Live host
state, network interoperation, a disposable profile, and exact runtime trace are `unverified` until
their controlled preconditions are reproduced.
