# Product Boundary

## Purpose and status

`sts2-protocol` is the publication boundary for a small set of genuinely shared, language-neutral,
transport-neutral STS2 contracts. This run provides one focused `poc-v1` message family with schema,
golden, invalid-fixture, digest, and conformance coverage. It adds no transport, host, or product
behavior.

The current build-completion instruction accepts the target repository as the sixth target. It
accepts only the initial neutral metadata seam recorded in [ADR 0003](decisions/0003-initial-neutral-metadata-package.md).
Any future contract enters only after its canonical owner, at least two named consumers, compatibility
profile, serialization, provenance, and conformance oracle are accepted.

## Allowed scope

The POC scope is protocol version, schema digest, provenance, correlation ID, instance ID, generation,
bounded state observation, one typed action identity/argument, and accepted/rejected status with an
error code. Owner-local legality and settlement semantics are projected but not transferred.

## Non-goals

This target must not own:

- game rules, state extraction, legality, combat, or domain transitions;
- host assemblies, loader metadata, main-thread callbacks, UI, saves, or game files;
- game-mod HTTP routes, gateway lifecycle/leases/routing, or process control;
- MCP framing, initialization, tool descriptions, prompts, or transport behavior;
- model/provider calls, training, scoring, trajectories, datasets, or experiment orchestration;
- credentials, authentication/authorization decisions, persistence, or mutation authority; or
- copied source, historical implementation, proprietary material, or unsanitized private data.

## Consumer and evidence boundary

The accepted `poc-v1` artifact consumers are `sts2-game-core`, `sts2-game-mod`, `sts2-gateway`,
`sts2-harness`, and `sts2-mcp-server`. The game-mod consumes only the inert representation artifact
for its local host translation mapping; it remains the host and mutation authority, and this does not
authorize a Cargo path dependency or runtime compatibility claim. No live consumer, host, gateway,
MCP peer, harness run, provider, package publication, or release has been exercised; those claims
remain unverified.

## `runtime-v1` profile

The first runtime profile is deliberately narrow. It describes an authenticated, bounded state
observation and one host-visible `show_runtime_probe` action. An accepted action must return a fresh
observation and a `status_overlay_visible` effect witness; an old generation must remain rejected
with `sts2.game-mod/stale_generation`. This action is an integration probe, not a gameplay mutation
or a transfer of host authority.

The profile has four named consumers: the game mod, gateway, harness, and MCP server. The canonical
source is [`schemas/runtime-v1.schema.json`](../schemas/runtime-v1.schema.json), with the checked-in
artifact and five deterministic golden messages under [`artifacts/runtime-v1/`](../artifacts/runtime-v1/).
The schema/conformance result is `confirmed` in this repository. Host execution and end-to-end
runtime compatibility remain `unverified` until an authorized disposable game test is run.

## `runtime-v2` contract

Runtime-v2 is a separate, inert contract for the bounded `end_turn` operation. Its observation
contains `combat_phase`, `turn_index`, `host_ready`, and `generation`; the action is legal only in
`combat/player_turn`. The implementation-neutral transition vector is generation 4/turn 2 to
generation 5/turn 3 with a `turn_end_settled` witness.

The profile carries `accepted`, `settled`, `rejected`, `unknown`, and `cancelled` outcomes. A stable
`operation_id` supports receipt replay and reconciliation; conflicting reuse is reported as
`idempotency_conflict`. `unknown` is intentionally not a completion claim, and cancellation is only
valid before mutation or after explicit no-mutation confirmation. The protocol describes these wire
semantics but does not own legality enforcement, host mutation, queueing, leases, transport, or
persistence. Consumer and live gameplay compatibility remain `unverified`.

## `runtime-v3-gameplay` contract

The separate `runtime-v3-gameplay` profile is the neutral fair-play projection for the full-run
vertical slice. It contains ordinary player-visible state, visible seed text, typed host-generated
legal actions, state/generation identity, and bounded observe, dispatch, wait, reobserve, and safe
recovery message shapes. It has no screen coordinates, arbitrary input, host object graph, raw
memory, save, credential, future RNG, or unrevealed outcome field.

The action family covers setup, map, combat card play/end turn, reward, shop, rest, event,
selection, victory, defeat, and save/quit. Every dispatch carries one current state ID, generation,
operation identity, and typed action. The host rechecks catalog membership and legality. Admission
does not imply settlement; a settled result requires a fresh observation, current legal-action set,
and transition witness. Unknown results require recovery or reconciliation and never trigger a
strategic retry. Consumer mappings and target-build behavior remain `unverified`.
