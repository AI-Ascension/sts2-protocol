# ADR 0007: `runtime-v3-gameplay` fair-play semantic contract

- Status: Accepted for the neutral artifact; consumer, host, Exo, and live-game compatibility remain unverified
- Date: 2026-09-04

## Context

Runtime-v2 proves a single argument-free `end_turn` operation, but it cannot carry the ordinary
player-visible state or complete semantic action catalog needed by a run coordinator. The next
slice needs one neutral representation shared by the game-mod producer and gateway, MCP, and
harness consumers. It must not move legality, host-thread mutation, lifecycle, tool framing, or
model policy into the protocol repository.

## Decision

The protocol owner admits a separate `runtime-v3-gameplay` profile under
[`schemas/runtime-v3-gameplay.schema.json`](../../schemas/runtime-v3-gameplay.schema.json) and
[`artifacts/runtime-v3-gameplay/`](../../artifacts/runtime-v3-gameplay/). Runtime-v1 and Runtime-v2
remain unchanged and are not silently upgraded.

The profile contains only bounded owned values: an ordinary player-visible observation, optional
visible seed text, a tagged run state, player-visible cards/resources/enemy intent, a complete
host-generated typed `LegalAction` catalog, generation and lease identity, and explicit lifecycle
messages for observe, legal-actions, dispatch, wait, reobserve, and recovery. Its action variants
cover setup, map, combat, rewards, shop, rest, events, selections, victory, defeat, and save/quit.
Coordinates, arbitrary input events, reflection paths, process commands, saves, credentials,
future RNG, and unrevealed outcomes have no representation.

Every mutating request carries a state ID, generation, operation identity, and exactly one typed
action. `accepted` is admission only. `settled` requires a fresh observation, current legal-action
catalog, and an explicit transition witness. `unknown` carries no effect witness and requires
reobserve/reconcile/recovery before another mutation. Duplicate action IDs in a catalog are
invalid; catalog membership and authoritative legality remain host-owned checks.

## Consumers and migration

Named artifact consumers are `sts2-game-mod`, `sts2-gateway`, `sts2-harness`, and
`sts2-mcp-server`. They consume copied release-like schema artifacts, not this crate's
implementation internals or a Cargo path dependency. Mappings are additive implementation work in
the next barrier. A consumer must reject the profile if its exact schema digest or provenance does
not match. No Runtime-v2 message is reinterpreted as Runtime-v3.

No `runtime-v3-gameplay-llm` profile exists in the accepted repository inputs. The harness keeps
the Exo request/decision envelope provider-owned and reuses this neutral fair-play profile; no
LLM-specific protocol authority is introduced.

## Evidence and consequences

The source schema, byte-identical package copy, manifest, digest inventory, sanitized goldens, and
offline conformance test are protocol-level evidence. They do not prove host extraction, main-thread
affinity, gateway leases, MCP transport, Exo execution, semantic legality, or a completed STS2
effect. Those gates remain `unverified` until exact build and service inputs are available.
