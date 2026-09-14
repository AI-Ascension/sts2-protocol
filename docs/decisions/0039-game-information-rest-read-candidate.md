# ADR 0039: separate game-information rest-read candidate

- Status: proposed candidate; no admitted consumers or runtime compatibility
- Date: 2026-09-14
- Canonical owner: `sts2-protocol`
- Prospective producer: `sts2-game-mod`
- Prospective consumers: `sts2-gateway`, `sts2-mcp-server`

## Contract and migration

`game-information-query-v2` is an independent, closed, inert successor profile.
All v1 schema, artifact, golden and implementation bytes remain unchanged.
Consumers must select the new profile and exact schema digest before decoding it.
The standalone v2 schema derives only from the repository's own pinned v1 schema
and the explicit additions below. Original fixtures use `source.kind: synthetic`.
The schema and original code are MIT; no host material is included.

Add `rest_option`, fields `rest_action`, `rest_eligible`, `rest_mode`,
`rest_option_id`, `rest_selector`, and two closed structured reference types.
The seven immediate options are clone, cook, dig, hatch, heal, kindle and lift.
Smith is card selection; mend is player selection. The contract projects these
source classifications and does not calculate their effects.

Available rest context binds one owned room snapshot and parent observation.
Live list repeats that context and returns opaque rest-option occurrences.
Detail repeats a returned definition and occurrence within the same room fence.
Occurrences differ from action, definition and room identities. The producer
owns membership in the exact captured registry; protocol validates only supplied
inert evidence and never establishes authenticity.

Rest queries support standard projection/detail only. Static list/search use
null targets; static get/availability use a registered definition. Live list uses
the room target; live detail/availability use a registered occurrence. Static
detail and live get/search are unsupported. Live filters are empty. Static fields
are description, display_name, rest_mode and rest_option_id; live adds the three
other new fields. Empty fields selects all allowed fields for that mode.

Requested fields occur exactly once, ascending by ASCII name. Classification
fields must be available when requested. Every available rest field has unit
`none`, non-null typed value and null reason. Nonavailable fields have null value
and unit, with a bounded reason. Non-rest entities cannot use new names or kinds.
Rest selectors bind smith/card or mend/player; counts are 0..256, required count
is positive, and selected plus remaining equals required. Selected count must
also equal the unique selected-choice count supplied by the owning capture.

Eligibility needs an explicit authoritative source; action presence does not
prove it. Initial production eligibility is unsupported. Text likewise requires
actual owned manifest/locale-qualified text. IDs cannot substitute for text.
Oversized existing action/selector IDs produce explicit nonavailability; never
truncate or replace them. Unknown/duplicate source option mappings reject.

## Serialization, lifetime and conformance

Canonical accounting uses compact UTF-8 JSON with sorted object keys and
preserved array order. Payload bytes encode the items array, including two bytes
for an empty array. Item bytes are the maximum encoded item size. Page bytes
encode the page without accounting. Text bytes sum only available text/text-list
values. Schema maxima remain unchanged; the first adoption uses request 16384,
response 262144, page 65536, item/text 4096, items 32 and cursor 512 byte ceilings.

Rest pages order by ascending definition_ref with identity_bytes. Definitions
and occurrences are unique. Complete coverage requires final traversal, a
classified source domain and all requested fields available. Other nonempty
pages are partial. Unavailable/not-observable pages are empty, final, total
unknown and cursor-free. Final traversal alone never proves full information.

One current snapshot/generation is retained initially. The six existing
invalidation events plus relevant room/selector/generation changes invalidate
the occurrence set. No wall-clock promise is made. Cursor bindings include
profile, exact digest, authority, normalized query, snapshot and occurrence
revision; normalization removes only cursor. At most 64 bindings are retained.

Deterministic conformance covers all nine IDs, static reads, two-page room
containment, exact detail targets, selector counts, classification, availability,
closure, duplicate members, identity fences, cursor isolation and byte bounds.
Validation of inert producer evidence is not host membership, authority or
read-only execution proof. Independent protocol schema and semantic checks plus
two actual consumer validators are required before admission.

## Ownership and pending gates

Protocol owns data/schema/serialization validation only. Gateway owns selected
profile headers, routes, authentication, leases and HTTP error behavior. MCP owns
its selected profile catalog and decoder. Mod owns capture, registry membership,
actual source facts, main-thread observation and absence of mutation. Harness
owns recording/replay. No transport, registry persistence or host calls are added.

Old clients remain on v1. Upgraded owners must reject profile/digest mismatch
before querying, preserve unavailable context and unknown coverage, and never
fall back by reinterpreting v2 as v1. Consumer adoption records must pin real
commits and matching artifact bytes; prospective names are not adoption.

This candidate does not close the rest-information issue, qualify native
behavior, add previews or candidate lists, grant dispatch authority, or establish
a release. Consumer confirmations, production adapters, loopback, harness replay
and separately authorized exact-host evidence remain pending.

## Validator dependency

The existing locked `jsonschema = 0.55.0`, with default features disabled, moves
from test-only to production use in the existing neutral crate. No package
version, resolver, installation or lockfile graph changes are intended. The
validator compiles only embedded standalone repository-owned schema bytes.
It accepts no caller schema and performs no remote or filesystem resolution.
Its capture and cursor inputs are caller-owned inert evidence: the authenticated
producer remains responsible for their truth, freshness and registration.
The validator rejects unregistered output or invented field availability against
that supplied evidence; passing does not authenticate the evidence itself.

## Unsupported request diagnostics

The normative full schema remains the acceptance boundary. A structurally valid
current-profile rest `query_request` that fails only supported-selector rules is
still rejected, with this fixed priority: `unsupported_projection` (including
well-shaped live get/search or static detail mode combinations), then
`unsupported_field`, then `unsupported_filter`.

The validator derives a request-only diagnostic schema once from the embedded
schema by removing only its identified rest-query conditional. All inherited
envelope, target, binding, provenance, required-member, unknown-member, type and
bound checks remain. Classification also requires an in-memory diagnostic copy,
with only unsupported selectors replaced by supported equivalents, to pass the
unchanged full schema. The actual request is never accepted, rewritten or sent.
This prevents unrelated malformed rest targets or scope from being masked.

Unknown members, missing required members, wrong types and unrelated invalid
rest shapes remain `malformed`. The diagnostic path does not apply to responses;
an unsupported selector in a `query_response` remains malformed. Original
response fixtures are preserved, and separate request fixtures exercise both
the fixed rejection codes and malformed-plus-unsupported controls. This is
neutral rejection classification, not gateway HTTP or routing implementation.
