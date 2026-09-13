# ADR 0038: game-information query contract ownership and candidate vocabulary

- Status: proposed; not admitted and no consumer has pinned it
- Date: 2026-09-13
- Owner: `sts2-protocol`

## Context

The existing runtime profiles are closed observation and action contracts.  They cannot safely
gain arbitrary game-content fields: doing so would change their schema digests and leave strict
readers unable to distinguish unavailable information from an empty value.  Consumers nevertheless
need a common way to request bounded static definitions and coherent live details without turning
display names, process-local IDs, or a page from a different turn into authoritative identity.

This decision records the owner boundary and the candidate vocabulary that the game-mod producer,
gateway, MCP adapter, and harness must jointly adopt before a schema is frozen.  It deliberately
does not define an HTTP route, registry, game rule, host adapter, persistence mechanism, or tool.

## Decision

Reserve the additive candidate profile `game-information-query-v1`.  Its canonical owner is
`sts2-protocol`; its named prospective consumers are `sts2-game-mod` (authoritative extraction),
`sts2-gateway` (authentication, instance/profile selection, and fences), `sts2-mcp-server`
(bounded tool mapping), and `sts2-harness` (agent projection, artifacts, and replay).  A consumer
must explicitly negotiate the accepted profile and exact schema digest.  Existing runtime-v1,
runtime-v2, runtime-v3-gameplay, runtime-v4, map, and checkpoint profiles remain byte-for-byte
unchanged.

### Identity and scope

Static definitions use a `definition_ref` tuple:

`(content_manifest_id, entity_kind, namespaced_id, variant)`.

`content_manifest_id` identifies the immutable content revision, `entity_kind` is a closed
negotiated vocabulary, `namespaced_id` is an opaque producer identifier, and `variant` is an
explicit nullable discriminator.  It is never a localized title.  A producer may display equal
titles for distinct references and may display one reference in multiple locales.

Live entities use an `instance_ref` tuple:

`(instance_id, run_id, epoch, entity_kind, entity_id)`.

`instance_id`, `run_id`, and `epoch` are authority fences, while `entity_id` names one occurrence
inside that fenced run.  It is distinct from both `definition_ref.namespaced_id` and every action
identifier.  A consumer must not substitute one namespace for another.

Each request also carries an explicit `locale` and `visibility_scope`.  The latter names the
authorized projection rather than asserting what the caller may infer.  A field that is hidden,
not extracted, unsupported, redacted, or too expensive is represented by typed availability; it is
never invented as zero, an empty string, an empty array, or a successful complete response.

### Revisions, snapshots, and cursors

Static results bind to `(content_manifest_id, locale, visibility_scope)`.  Live results additionally
bind to `snapshot_ref`, which contains the `instance_ref` fence, an opaque snapshot identifier, and
a monotonically scoped state generation.  A list page and a detail response may be combined only
when their bindings are equal.  A server rejects a request when a retained snapshot has expired or
was invalidated by restart, restore, profile change, content change, run change, or epoch change.

The future schema must make the producer declare snapshot lifetime, maximum retained snapshots, and
the maximum page byte/text/item limits.  It must not promise retention based on wall-clock behavior
that protocol does not own.

A cursor is opaque and bound to all of: negotiated profile and schema digest, query kind,
normalized filters, projection/detail level, content revision, locale, visibility scope, and (for
live queries) the snapshot reference.  It is
not reusable across any changed binding.  Ordering is a producer-declared deterministic ordering
for the bound revision/snapshot; a response declares whether it is final and whether a total count
is known.  An empty page is therefore distinguishable from unavailable coverage and from a
non-final page.

### Candidate operations and result semantics

The frozen profile will define separately versioned request/result shapes for `list`, `search`,
`get`, `detail`, and `availability`.  Every request names its query kind, filters, projection or
detail level, limits, locale, visibility scope, and the required static/live binding.  Every result
returns its exact binding, deterministic order declaration, bounded payload accounting, coverage
state, and per-field availability.

The candidate typed error vocabulary is `unknown_kind`, `unknown_id`, `ambiguous_id`,
`unsupported_filter`, `unsupported_projection`, `unsupported_version`, `denied_scope`,
`stale_snapshot`, `stale_cursor`, and `result_limit_exceeded`.  Errors retain the request
correlation supplied by the boundary owner but do not disclose a hidden ID or field merely to
explain a denial.  Unknown enum values and versions fail explicitly; a conforming reader must not
coerce them to a known value.

### Initial conformance vectors

Before admission, the artifact must contain canonical valid and invalid synthetic vectors with at
least the following stable case identifiers.  They are requirements for the future artifact, not
evidence that a runtime can produce them today.

| Case | Expected outcome |
| --- | --- |
| `GIQ-VALID-STATIC-TWO-PAGE` | Two pages share content, locale, scope, query, and cursor binding; the second is final and reports a known total. |
| `GIQ-VALID-LIVE-DETAIL-PINNED` | A detail result uses the same `instance_ref` and `snapshot_ref` as its parent live observation. |
| `GIQ-VALID-UTF8-BOUNDS` | Boundary UTF-8 text is accepted and its byte count, not character count, is reported. |
| `GIQ-VALID-SOURCE-PROVENANCE` | A result identifies its producer-declared source and availability without exposing unavailable source data. |
| `GIQ-INVALID-CROSS-LOCALE-CURSOR` | Reusing a cursor with another locale fails as `stale_cursor`. |
| `GIQ-INVALID-CROSS-EPOCH-CURSOR` | Reusing a live cursor after an epoch change fails as `stale_cursor` or `stale_snapshot`, never with mixed results. |
| `GIQ-INVALID-MIXED-GENERATION` | Combining a page and detail from different state generations is rejected. |
| `GIQ-INVALID-AMBIGUOUS-DISPLAY-NAME` | A display name without a unique definition reference fails as `ambiguous_id`. |
| `GIQ-INVALID-UNKNOWN-ENUM-VERSION` | Unknown entity kind or profile version fails explicitly. |
| `GIQ-INVALID-INTEGER-BOUNDS` | Integer values below or above their declared bounds fail rather than wrapping, rounding, or being treated as missing. |
| `GIQ-INVALID-OVERSIZED` | Item, page, byte, and text limits produce `result_limit_exceeded`; no truncated result is labelled complete. |
| `GIQ-INVALID-MISSING-VS-EMPTY` | Missing, unavailable, null, zero, and empty values remain distinguishable according to the field contract. |

At least two of the named consumers must validate byte-identical vectors, including the two-page
and pinned-live-detail cases, before the candidate can become accepted.  Tests must also cover
cross-run, content-revision, scope, filter, and profile reuse.

## Ownership and compatibility

Protocol owns only the inert schema, serialization rules, artifact digest, synthetic vectors, and
conformance description.  Game-mod owns facts, extraction, and main-thread constraints; gateway
owns routing, authentication, and fences; MCP owns descriptors and fixed gateway mapping; harness
owns agent integration, authorized projections, history, and replay.  No owner may use this
decision to add a direct harness-to-host bypass or a gateway-to-harness dependency.

This is an additive candidate profile.  Adding a field, enum member, filter, projection, ordering,
or limit after acceptance requires the compatibility classification and negotiated profile/digest
specified by the admitted schema.  Existing closed profiles cannot silently absorb candidate
fields.

## Consequences

- Producers and consumers have one explicit boundary for identity, freshness, pagination, and
  honest partial coverage before transport-specific code is written.
- This decision establishes no route, registry, host support, native content extraction, consumer
  implementation, or end-to-end game-information tool.
- The next protocol change is a reviewed schema/artifact/fixture set whose values are validated by
  at least two named consumers; until then this vocabulary is proposed only.
