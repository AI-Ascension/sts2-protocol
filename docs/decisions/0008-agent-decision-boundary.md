# ADR 0008: no Exo-specific neutral decision contract

- Status: Accepted
- Date: 2026-09-04

## Context

The harness needs a pinned provider adapter and a bounded structured decision response. That need
does not by itself create a cross-repository game contract. The accepted inputs do not contain a
separate `runtime-v3-gameplay-llm` producer, consumer, version, or conformance requirement.

## Decision

Do not add an `agent-decision-v1` schema or protocol artifact in this wave. The neutral protocol
boundary remains `runtime-v3-gameplay`; the harness-owned Exo envelope is an adapter concern and
uses the neutral fair-play observation, current host-generated action IDs, and explicit wait,
reobserve, recovery, or action outcomes. Its strict parser rejects unknown fields, private
reasoning-shaped fields, malformed values, and action IDs absent from the current catalog.

If a future independent producer and consumer require a durable decision envelope, the protocol
owner must add a bounded versioned schema, artifact manifest, digest, golden/conformance cases,
consumer mappings, and a new compatibility decision together. No Exo SDK type becomes protocol
authority.

## Evidence

The decision is source-derived from the current repository graph and the Runtime-v3 contract. Exo
connectivity, provider behavior, target-game legality, and end-to-end settlement remain unverified.
