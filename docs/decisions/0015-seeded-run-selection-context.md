# ADR 0015: Seeded-run native selection context

- Status: Accepted for the `seeded-run-v1` artifact; consumer and live host compatibility remain unverified
- Date: 2026-09-09
- Owner: `sts2-protocol`
- Consumers: `sts2-game-mod`, `sts2-gateway`, `sts2-harness`, `sts2-mcp-server`

## Context

The explicit seeded-run profile preserves the caller's requested seed, while only the game host can
establish the canonical seed and a run-start effect. A seed alone does not identify the native lobby
configuration that made a run reproducible. The recovered profile therefore needs one bounded,
transport-neutral context manifest that can be carried through the start operation and read back by
the host.

The context is a selected native configuration, not game rules or a host object. Its lifetime is the
start operation and its receipts. A reconciliation request is read-only and has no launch context.

## Decision

Extend `seeded-run-v1` with a required `selected_context` on every start request and lifecycle result,
plus a matching top-level `context_digest`. The manifest contains the standard game mode, Ironclad,
ascension, a sorted unique modifier set, an ordered unique act sequence, selection policy, profile
baseline, save policy, separate game and mod compatibility identities, and its SHA-256 digest. The
digest covers compact canonical JSON of every manifest field except `context_digest`; the top-level
digest and the host observation digest must match it.

Act order is semantic native campaign order and is preserved in the digest. Modifiers are a
canonicalized sorted set. Unknown members and enum values fail closed. The protocol validates bounds,
ASCII identity/text alphabets, digest shape, list uniqueness, and the content-addressed digest, but it
does not select a character, access a save, dispatch a host call, or decide game legality.

The existing lifecycle distinction remains unchanged: `accepted` is admission only, `settled`
requires a fresh host observation and `run_started` witness, and `unknown` requires read-only
reconciliation by the same operation identity. Runtime-v1, Runtime-v2, Runtime-v3, and all other
protocol artifact bytes remain separate and unchanged.

## Compatibility and evidence

This is an additive protocol profile change relative to the unpublished recovered candidate. Its exact
schema digest, manifest, copied schema, goldens, conformance case, and checksum inventory must be
accepted together; a consumer must reject a different digest or provenance. The Rust representation
requires nullable wire members to be explicitly present, matching the closed Draft 2020-12 schema.

The checked-in vectors prove schema, serialization, bounds, digest binding, and lifecycle shape only.
They do not prove that the named consumers compile against this artifact, that a licensed game host
accepts the context, that a profile or save is safe, or that a live run settles.
