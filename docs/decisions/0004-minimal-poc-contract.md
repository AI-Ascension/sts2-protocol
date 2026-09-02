# ADR 0004: Minimal `poc-v1` Contract

- Status: Accepted for the deterministic fake vertical slice; public release remains unverified
- Date: 2026-09-02

## Context

The build-completion slice needs one small contract that can cross five independent repository
boundaries without importing implementation modules. Existing neutral metadata remains useful
foundation material, but it is too broad to serve as the observable trace for one state read and one
action result.

## Decision

`sts2-protocol` owns one `poc-v1` JSON message family with four shapes: `state_request`,
`state_response`, `action_request`, and `action_response`. Every shape carries protocol version,
schema digest, inert provenance, correlation ID, instance ID, and generation. Responses carry a
bounded observation; action messages carry the `use_budget` identity and bounded `units` argument;
action responses carry `accepted` or `rejected` plus an optional owner-qualified error code.

The canonical schema is `schemas/poc-v1.schema.json`. `artifacts/poc-v1/` is the release-like
bundle, with manifest, byte-identical schema copy, complete message-shape goldens, one invalid
typed-action fixture, and SHA-256 inventory. The accepted artifact consumer set is exactly
`sts2-game-core`, `sts2-game-mod`, `sts2-gateway`, `sts2-harness`, and `sts2-mcp-server`; game-mod
consumption is limited to local host-translation mapping and does not transfer host or mutation
authority. Downstream targets consume copied artifact bytes and implement local mappings; none
imports `sts2-protocol` Rust modules or a generic common crate.

## Ownership and limits

The protocol owns representation, bounds, version, provenance, and conformance only. `sts2-game-core`
owns legality and state transition semantics; `sts2-game-mod` owns host translation and settlement;
the gateway owns instance and lease checks; MCP owns framing and catalog; the harness owns the
episode and artifact trace. The artifact is deterministic and offline; it is not a public release and
does not prove runtime compatibility.

## Evidence

Target-local conformance validates source/artifact byte identity, Draft 2020-12 fixture validity,
exact golden bytes for all four message shapes, metadata, error identity, manifest identity, checksum
inventory, and the structurally valid invalid fixture. Consumer PRs must validate their copied
manifest and digest. Any live host, network, provider, MCP session, gateway process, or publication
remains unverified.
