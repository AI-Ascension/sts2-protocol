# ADR 0010: split the unadmitted co-op prototype

- Status: Accepted for the Exo gameplay review pass
- Date: 2026-09-05

## Decision and provenance

Remove the co-op Rust module, public exports, source schema, and prototype tests from the Exo
gameplay change. Preserve them as a separate proposal, including the schema provenance and
conformance corrections recorded in [ADR 0009](0009-proposed-contract-conformance-corrections.md).
The original proposal is retained at `aaf64272d0536fb6a05c36319b43e01510635894`; its rebased source
is `85ba19ed9008f4d5942bb9684144ebfb3b8bf882`, retained by `review/coop-prototype-preserved`.
Neither revision establishes an admitted co-op artifact or live integration.

The separate proposal remains blocked until a canonical ownership/admission record identifies at
least two actual named consumers, version and identity lifetime, serialization and unknown-value
rules, compatibility classification, provenance, and deterministic producer/consumer conformance.
The preserved source is original hand-authored MIT material; no host or proprietary files are needed.

## Evidence and ownership

Source review identified one actual wire consumer: `sts2-mcp-server` at
`3785f0e9b6c9f2ff026c7f1d8825d63fce227b02`, whose
`crates/mcp-server/src/projection_coop_gameplay.rs` validates and projects a co-op synchronization
response. Its mapping requests a gateway synchronization route that has no implementation in the
reviewed gateway source. Gateway's `coop_session.rs`, game-mod's `CoopProjection.cs` and
`CoopSynchronization.cs`, and harness's `episode/coop.rs` are local state models rather than complete
wire producers or consumers of the co-op profile. Similar field names do not prove consumption.

These are source-derived findings. Authenticated peer voting, transport integration, authoritative
host effects, and live cooperative gameplay remain unverified. Protocol may validate structural
facts but does not grant mutation authority; the separate proposal must use structural predicate
names and preserve host/gateway ownership.

## Scope and validation

Only checks and policy entries for the removed co-op files move with the proposal. Gameplay and
frozen-profile CI gates remain intact. No admitted artifact, schema digest, or frozen profile bytes
change. The Exo gameplay artifact retains its existing digest and must pass the complete repository
validation matrix before admission. This split does not complete or release the co-op capability.
