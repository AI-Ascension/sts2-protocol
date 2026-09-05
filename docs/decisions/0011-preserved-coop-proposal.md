# ADR 0011: preserve the unadmitted co-op proposal

- Status: Proposed; admission blocked
- Date: 2026-09-05

## Scope and provenance

This separate branch restores the original hand-authored MIT co-op source schema, Rust prototype,
provisional exports, and deterministic tests split from gameplay by
[ADR 0010](0010-split-unadmitted-coop-proposal.md). The source originated at
`aaf64272d0536fb6a05c36319b43e01510635894` and was rebased at
`85ba19ed9008f4d5942bb9684144ebfb3b8bf882`. The schema remains unchanged with SHA-256 digest
`85e0028c1ae20e49542791da165eeabaaea0cc2023626b5094b6660ebcc0cc81`.
No frozen or gameplay artifact bytes change. This restoration preserves reviewable work; it does
not admit a contract, publish an artifact, or establish compatibility with a live consumer.

## Structural predicates

`CoopSynchronization::is_complete_synchronization` checks bounded synchronization metadata.
`CoopGameplayMessage::is_synchronized_action_request` checks message validity and synchronized
local-action request shape. They replace the authority-suggesting `mutation_allowed` method names
without changing wire bytes or validation behavior. Neither authenticates a peer, authorizes a
mutation, proves game legality, or establishes an effect. Host and gateway authority remain with
their owning consumers. Other consumers have no compile-time dependency on these Rust prototypes.

## Admission blocker and required evidence

The review found only one actual wire consumer: MCP's synchronization response projection.
Gateway, game-mod, and harness local co-op state models do not establish additional wire consumers;
the requested gateway synchronization route has no reviewed producer implementation.
Do not merge this proposal until a canonical ownership/admission record identifies at least two
actual named consumers and records namespace and lifetime, version/profile, serialization,
unknown-value behavior, compatibility classification, provenance, and deterministic conformance.
Producer/consumer mappings must validate exact metadata, semantic relations, and uncertainty
without treating a structural predicate as authority.

Prototype schema hashes and deterministic tests are source/contract evidence only. Authenticated
peer voting, host effects, transport integration, and live cooperative gameplay remain unverified.
