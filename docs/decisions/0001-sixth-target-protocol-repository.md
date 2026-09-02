# ADR 0001: Accepted Sixth-Target Protocol Repository

- Status: Accepted for the current build-completion run; contract publication remains gated
- Date: 2026-09-02

## Context

The earlier protocol investigation proposed no independent repository because it found no released
neutral consumer, owner, or cadence. The current build-completion orchestration explicitly supersedes
that disposition for this run and names `sts2-protocol` as the original sixth accepted target.

The superseding instruction is an implementation-scope decision, not evidence that a shared contract
already exists or that a runtime consumer is compatible. The target must remain narrow and must not
become a generic common-code destination.

## Decision

Prepare `sts2-protocol` as a real target with repository foundation, local governance, and a future
contract-artifact boundary. It may publish only language-neutral, transport-neutral contracts with a
canonical owner, at least two named consumers, explicit version/profile and compatibility rules,
provenance, and deterministic conformance. An item that fails those gates remains owner-local or is
recorded as blocked.

This decision does not authorize product behavior, a product crate, a release, a host integration, a
transport, a game API, a gateway lifecycle model, an MCP catalog, or model/provider behavior.

## Alternatives

1. Retain the old no-repository disposition: rejected for this run because the caller explicitly
   accepts the sixth target.
2. Use a generic shared crate: rejected because it would blur ownership and invite dependency cycles.
3. Put every boundary schema here: rejected because meaning and authority remain owner-local.
4. Prepare this narrow target and gate each contract: accepted because it preserves the superseding
   scope while making unresolved consumer/release evidence visible.

## Consequences and revisit trigger

The target receives independent foundation and policy checks, but no compatibility claim follows from
its existence. The decision should be revisited if maintainers withdraw the sixth target, or if
independent consumers cannot be named and no neutral artifact survives the ownership test.
