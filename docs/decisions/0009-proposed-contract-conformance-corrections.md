# ADR 0009: correct unpublished gameplay conformance

- Status: Accepted for the proposed, unpublished contracts; consumer migration required
- Date: 2026-09-05

## Defects and decision

The initial Runtime-v3 schema checked field types but admitted contradictory request/result
payloads that the Rust validator rejected. Conversely, Rust accepted omitted required-nullable
members and ignored extra fields in tagged enum objects. Empty enum variants require closed
struct-shaped wire variants: `deny_unknown_fields` alone does not close Serde unit variants.
These are safety corrections, not additional gameplay capabilities.

The schema now expresses the existing kind/status/wait/recovery payload constraints and excludes
C1 control characters consistently with Rust. Rust requires every schema-required nullable member
and rejects unknown fields in all state, action, and enemy-intent variants.

The broad Runtime-v3 proposal's schema digest changes from
`fbfb18279b0c7ebb350ef0ce0d56547fa11e83985b13380cb2b0f1dba4cb56e9` to
`b37c80f583aeaf4f81ede2083bcfb4129196baf5eb092470e8738173c4b7226c`.
No published release, tag, or immutable artifact is replaced. This is a correction to an unmerged
proposal. All four named consumers must explicitly migrate their copied artifact, digest pin,
parser, fixtures, and negative cases together; old digests must fail before mutation.

The earlier bounded card-play proposal (PR #7, digest
`c961bbde893f0422f80233d14ea9ae8b648ee9032136e5370aa5f6b949f6575e`) uses the same profile name
but a different action, observation, and reconciliation contract. It is not an ancestor or a
compatible wire profile. Do not merge both proposals unchanged or infer negotiation from the name.
Choosing or renaming/consolidating those competing proposals remains a maintainer decision.

## Complete validation boundary

JSON Schema establishes structural validity only. Consumers must additionally enforce the exact
schema digest/provenance, UTF-8 text byte limit (512 bytes, not 512 Unicode scalar values or UTF-16
code units), HP not exceeding maximum, unique action IDs, observation state/generation matching
the envelope, and strictly increasing transition generations matching the settled envelope.
These cross-field and byte-length constraints are mandatory, not optional game-rule inference.
Schema numeric values such as `1.0` can be mathematical integers, while the typed wire parser
requires integer tokens; canonical serialization emits integer tokens and explicit null members.

Response correlation and operation/action witness binding require the original request or stored
receipt and therefore remain consumer/host checks. An internally consistent witness is not proof
that the host performed an effect. Wait outcomes remain unchanged: only settled successor/same-state
mutation or unknown timeout/recovery-required is representable. Use explicit operation reconciliation
to retrieve other receipt statuses; do not claim a rejected operation settled.

## Co-op source-only contract

Historically, this proposal included a co-op schema and Rust prototype without an admitted artifact
bundle, named producer/consumer conformance, or a publication decision. They are now retained on a
separate proposal branch under [ADR 0010](0010-split-unadmitted-coop-proposal.md), rather than in the
gameplay source or exported API. They must not be described as a packaged,
integrated co-op release. Corrected source validation requires complete valid messages before the
mutation predicate can return true, explicit nullable members, and kind/peer/synchronization shapes.
Its source digest changes from `2c34d013315fbf2e16de03dbe2bd4c43d4c13c744292548cc46ea960af5e1fa2`
to `85e0028c1ae20e49542791da165eeabaaea0cc2023626b5094b6660ebcc0cc81`.
No transport, authenticated peer voting, host authority, or live co-op capability is established.

## Evidence

Regression tests reproduce the parser/schema disagreements, enumerate request/result shapes,
and check nested enum closure. Co-op rejection tests remain with the separate prototype proposal.
CI verifies actual gameplay artifact bytes with
SHA-256. These checks are inert contract evidence, not host, provider, or end-to-end execution.
