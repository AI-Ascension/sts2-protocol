# Product Boundary

## Purpose and status

`sts2-protocol` is the publication boundary for a small set of genuinely shared, language-neutral,
transport-neutral STS2 contracts. Wave 2 initializes one focused metadata package with schema, golden,
and conformance coverage. It adds no transport, host, or product behavior.

The current build-completion instruction accepts the target repository as the sixth target. It
accepts only the initial neutral metadata seam recorded in [ADR 0003](decisions/0003-initial-neutral-metadata-package.md).
Any future contract enters only after its canonical owner, at least two named consumers, compatibility
profile, serialization, provenance, and conformance oracle are accepted.

## Allowed scope

The initial scope is neutral identity/correlation/lineage and sequence metadata, independently
versioned profile and digest descriptors, selected lifecycle/deadline/cancellation metadata, neutral
error-envelope metadata, and a contract/schema manifest with provenance. Owner-local semantics may be
projected but are not transferred without an explicit decision.

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

Named prospective consumers are `sts2-game-core`, `sts2-gateway`, `sts2-mcp-server`, and
`sts2-harness`. The game-mod remains the host authority and is not a direct protocol consumer unless
that need is separately accepted. No live consumer, host, gateway, MCP peer, harness run, provider,
package publication, or release has been exercised; those claims remain unverified.
