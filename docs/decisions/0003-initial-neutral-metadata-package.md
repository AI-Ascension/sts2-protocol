# ADR 0003: Initial Neutral Metadata Package

- Status: Accepted for Wave 2 initialization; release compatibility remains unverified
- Date: 2026-09-02

## Context

The accepted sixth target now needs a non-empty, useful source and test seam. The seam must prove
that the repository can own a narrow neutral contract without becoming a second game, host, gateway,
MCP, model, provider, storage, or harness authority. No cross-repository path dependency is approved
for this wave.

## Decision

Initialize one package, `sts2-protocol`, under `crates/protocol`. Its public surface contains only:

- namespace-qualified opaque identity, correlation, lineage, and sequence metadata;
- independent version/profile and SHA-256 digest descriptors;
- selected lifecycle, relative deadline, cancellation, and operation-status metadata;
- neutral error origin, code, retryability, safe-message, and operation-status metadata; and
- a contract/schema manifest with named consumers and repository-relative provenance.

The package has no sockets, transports, host objects, game rules, gateway leases, MCP framing or
catalog, provider calls, storage, process lifecycle, or experiment semantics. It has no cross-repo
path dependencies. Validation is explicit and deterministic; canonical JSON uses compact stable
struct field order. A JSON Schema, three golden fixtures, and one implementation-neutral conformance
case are checked by target-owned integration tests.

The manifest names `sts2-game-core`, `sts2-gateway`, `sts2-mcp-server`, and `sts2-harness` as
prospective consumers. The manifest does not grant those boundaries authority and does not claim
that any consumer currently compiles against or runs this package.

## Consequences

The package is buildable and testable offline, while actual consumer integration remains a later
cross-repository decision. Any future field or type must preserve one canonical owner, at least two
real consumers, explicit version/profile and digest policy, provenance, and bidirectional
conformance. A boundary-specific meaning remains in its owning repository.
