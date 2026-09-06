# ADR 0011: Runtime-v1 typed wire envelope and closure

- Status: Accepted for the frozen `runtime-v1` artifact; consumer and live host compatibility remain unverified
- Date: 2026-09-05

## Context

The historical wire closure (PR #9) added required-present nullable members, closed objects, and
duplicate-key rejection for the neutral metadata, `poc-v1`, and `runtime-v2` profiles. It did not
cover `runtime-v1`, one of the three frozen profiles. The crate exported `RuntimeObservation`,
`RuntimeAction`, `RuntimeEffectWitness`, `RuntimeMessageKind`, and `RuntimeStatus` but no message
envelope, so no Rust type could decode a `runtime-v1` golden, no test called `decode_json`,
`canonical_json`, or `validate()` for the profile, and `RUNTIME_MAX_GENERATION` and
`RUNTIME_MAX_ACTION_COUNT` had no exercising test. Four `RuntimeValidationError` variants
(`Metadata`, `Provenance`, `Identity`, `GenerationBounds`) had no producer. Meanwhile
[`schemas/runtime-v1.schema.json`](../../schemas/runtime-v1.schema.json) requires `observation`,
`action`, `status`, `error_code`, and `effect_witness` to be present on every message kind, with
`unevaluatedProperties: false` on each kind. The profile's closure guarantee was schema-only.

## Decision

Add a `RuntimeMessage` envelope under `crates/protocol/src/runtime/message.rs`, mirroring
`runtime_v2/message.rs`, and a `RuntimeProvenance` object, both exported from the crate root.
The envelope carries the schema's base members (`protocol_version`, `schema_digest`, `provenance`,
`correlation_id`, `instance_id`, `session_id`, `lease_id`, `lease_epoch`, `generation`, `kind`)
and the five nullable members. Every nullable member uses the existing `required_option`
deserializer, so an omitted member is a decode error rather than `None`; `deny_unknown_fields`
closes the envelope and provenance objects; serde's duplicate-field rejection applies at the
envelope and inside every nested object.

`RuntimeMessage::validate()` enforces exactly the schema's constraints and nothing more: fixed
protocol version, digest, and provenance; the identity alphabet and length for the four identities
and `error_code`; the generation bound on `lease_epoch` and `generation`; the existing observation,
action, and witness validators; and the kind/status member shape (`state_request` all null,
`state_response` observation only, `action_request` action only, `action_response` observation and
action with `accepted` requiring a witness and null `error_code`, `rejected` requiring an
`error_code` and null witness). A new `RuntimeValidationError::ResultShape` variant names the shape
failure. No cross-field freshness rule (witness generation equal to envelope generation) is added:
the schema does not express one, and witness binding remains a consumer/host check as in ADR 0009.

Tests decode each of the five checked-in goldens, validate them, and re-encode to byte-identical
canonical JSON; reject omission of each nullable member in every golden; reject duplicate keys at
the envelope and inside nested objects; reject unknown members at every object boundary; and check
each bound at and above its limit with the schema and the typed validator agreeing.

## Compatibility

Classification: additive-compatible, Rust API only. No byte under `artifacts/`, `schemas/`, or
`conformance/` changes; the `runtime-v1` schema digest, manifest, checksum inventory, and golden
bytes are unchanged, verified by an empty `git diff --stat origin/main -- artifacts schemas
conformance` and `sha256sum --check` on all four profile inventories. A message the schema accepts
is accepted by `RuntimeMessage`; a message the schema rejects is rejected either at decode or by
`validate()`. The only input Rust rejects that the schema accepts is a well-formed digest that is
not the frozen `runtime-v1` digest, which the schema cannot pin and which every consumer must
already reject before mutation.

Consumers that emit `runtime-v1` messages through a copied artifact are unaffected. A Rust caller
matching `RuntimeValidationError` exhaustively gains one variant. No in-tree consumer depends on
this crate; the four named consumers use copied release-like files. Local schema, serialization,
and policy evidence is `confirmed`; host, gateway, MCP, harness, and live probe behavior remain
`unverified`.
