# ADR 0040: game-information lookup-binding discovery and observation contract

- Status: proposed candidate; no admitted consumer, no route decision, no runtime compatibility
- Date: 2026-09-16
- Owner: `sts2-protocol`
- Prospective producer: `sts2-harness` (authority), `sts2-gateway` (instance lease), `sts2-game-mod` (content revision)
- Prospective consumers: `sts2-harness`, `sts2-gateway`, `sts2-mcp-server`

## Context

The game-information query profiles define what a consumer may ask and how a result is fenced, but
they do not define how a consumer first discovers the lookup binding it is allowed to use, nor how
an observation is bound to that binding identity across staleness and re-observation.  Without a
single authoritative answer, each consumer would invent its own owner comparison, its own staleness
rule, and its own behaviour when a binding is no longer observable.  Those choices change what
"the same lookup" means, so they cannot be left to an individual adapter.

The observable evidence is the harness implementation.  `LookupBinding::same_owner` already decides
owner equality over the scope, `game_profile`, `content_manifest_id`, `locale`, and
`authority_epoch`; the MCP context already fences `instance_id` separately and reports
`LookupError::Scope` on a mismatch; and `LookupError::Reobserve` already exists as the re-observation
signal.  This decision names that existing comparison as the normative binding identity rather than
inventing a second one.

This decision deliberately does not define a route, transport, endpoint, method, host adapter,
registry, clock, credential, persistence mechanism, or tool.  It is an inert, read-only contract.

## Decision

Reserve the additive candidate profile `game-information-lookup-binding-v1` under the contract
identifier `sts2.protocol/game-information-lookup-binding-v1`.  Its canonical owner is
`sts2-protocol`.  A consumer must negotiate the accepted profile and the exact schema digest before
it decodes a document.  All existing runtime, map, checkpoint, and game-information-query profiles
remain byte-for-byte unchanged.

The normative rules are `LBR-1` through `LBR-12` in the contract section
[`game-information-lookup-binding-contract.md`](../game-information-lookup-binding-contract.md).
The machine-checkable witness is the artifact
[`artifacts/game-information-lookup-binding-v1`](../../artifacts/game-information-lookup-binding-v1/README.md)
with schema [`schemas/game-information-lookup-binding-v1.schema.json`](../../schemas/game-information-lookup-binding-v1.schema.json)
and conformance case
[`conformance/cases/game-information-lookup-binding-v1.json`](../../conformance/cases/game-information-lookup-binding-v1.json).

### Stable binding identifier

A binding is identified by `binding_id`: the lowercase SHA-256 hex digest of the canonical identity
input.  The identity input is exactly the tuple the owner comparison already uses:
`project_id`, `run_id`, `episode_id`, `agent_id`, `game_profile`, `content_manifest_id`, `locale`,
and `authority_epoch`.

The digest input is compact UTF-8 JSON with object members sorted lexicographically.  That
canonicalization is chosen because it is the behaviour a sorted-map serializer produces in each
named consumer language, so three implementations recompute the same digest without a shared
library.  `instance_id` is deliberately excluded from the digest: it fences the session or instance
lease, not the binding identity, so a lease change must not silently produce a new binding.

### Discovery precedes observation

Discovery reports the pre-observation state, the required capability (profile and schema digest),
and no observation.  A consumer must be able to discover a binding before it may observe it.  A
re-observation never re-discovers: a changed identity input requires a fresh discovery, because a
re-observation that quietly rebinds would let a caller observe a different owner under the old
correlation.

### Observation binding, staleness, and re-observation

An observation is bound to the same `binding_id` under which it was observed.  A retained
observation whose `state_generation` falls behind the generation the context retains is stale; the
consumer discards it and re-observes under the same `binding_id` with a new `observation_id`.
Staleness is therefore context-relative and declared, not inferred from wall-clock time.

When re-observation is impossible the binding fails closed: the state is `reobserve_exhausted`, at
least one attempt is recorded, and the terminal code is `reobserve_unavailable`.  An exhausted
binding must never fall back to a stale, partial, or fabricated observation, and its `observation`
member must be null.  This is the single place where the contract prefers an explicit terminal
refusal over a plausible-looking answer.

### Closed vocabulary

The kinds are `lookup_binding_discovery_response`, `lookup_binding_observation_response`, and
`error_response`.  The discovery states are `not_yet_observed`, `observed`, `reobserve_required`,
and `reobserve_exhausted`.  The error codes are `unsupported_version`, `invalid_identity`,
`denied_scope`, `missing_capability`, `stale_snapshot`, `mixed_binding`, `reobserve_unavailable`,
and `malformed`.  Unknown versions, schema digests, enum values, and members fail explicitly; a
conforming reader must not coerce them to a known value.

### Initial conformance vectors

The artifact carries canonical valid goldens and invalid vectors with the stable case identifiers
below.  They are requirements for this artifact, not evidence that a live runtime produces them.

| Case | Expected outcome |
| --- | --- |
| `LBR-VALID-DISCOVERY-INITIAL` | A discovery response reports `not_yet_observed`, the required capability, and no observation. |
| `LBR-VALID-DISCOVERY-OBSERVED` | An observation is bound to the same `binding_id` and its generation satisfies the retained generation. |
| `LBR-VALID-REOBSERVE-REQUIRED` | A stale retained observation is superseded under the same `binding_id` with no observation. |
| `LBR-VALID-REOBSERVE-EXHAUSTED` | The terminal fail-closed state reports at least one attempt and no observation. |
| `LBR-VALID-REOBSERVED` | A re-observation keeps `binding_id` and issues a new `observation_id`. |
| `LBR-VALID-BINDING-IDENTITY-INPUT` | The declared identity input recomputes to the declared `binding_id`. |
| `LBR-VALID-BINDING-IDENTITY-EPOCH` | A changed `authority_epoch` yields a different `binding_id`. |
| `LBR-INVALID-WRONG-SCOPE` | A self-consistent binding under another run fails as `denied_scope`. |
| `LBR-INVALID-WRONG-INSTANCE` | A lease from another instance fails as `denied_scope` while `binding_id` is unchanged. |
| `LBR-INVALID-FORGED-BINDING-ID` | A `binding_id` that does not recompute fails as `invalid_identity`. |
| `LBR-INVALID-MIXED-BINDING` | An observation bound to another identity fails as `mixed_binding`. |
| `LBR-INVALID-STALE-OBSERVATION` | A generation behind the retained generation fails as `stale_snapshot`. |
| `LBR-INVALID-MISSING-CAPABILITY` | An unnegotiated required capability fails as `missing_capability`. |
| `LBR-INVALID-SUPERSEDES-MISMATCH` | Re-observe naming an unretained observation fails as `malformed`. |
| `LBR-INVALID-UNKNOWN-VERSION` | An unknown profile version fails as `unsupported_version`. |
| `LBR-INVALID-REOBSERVE-UNAVAILABLE-SHAPE` | The terminal code against a non-terminal state fails as `malformed`. |
| `LBR-INVALID-EXHAUSTED-WITH-OBSERVATION` | An exhausted binding carrying an observation fails as `malformed`. |
| `LBR-INVALID-STATE-SHAPE` | An `observed` state without an observation fails as `malformed`. |
| `LBR-INVALID-DUPLICATE-KEY` | A duplicate object member fails as `malformed`. |

At least two named consumers must validate byte-identical vectors before this candidate can be
admitted, and a reviewer other than the author must approve the final artifact digest.

## Open decisions

The transport that serves discovery and observation is **not** decided here and must not be
invented by an implementation.  Whether discovery is served by the gateway, by the MCP adapter, or
by a dedicated channel is an open question for the owners of
[`sts2-harness#127`](https://github.com/AI-Ascension/sts2-harness/issues/127) and the gateway; this
contract is transport-neutral and only fixes the document shapes, the binding identity, and the
failure behaviour.  Until that route is recorded, this profile is a serialization contract only.

## Ownership and compatibility

Protocol owns only the inert schema, the canonicalization and digest rule, the artifact digest, the
synthetic vectors, and the conformance description.  Harness owns the authority over scope and
`authority_epoch`; the gateway owns the instance lease and locale selection; the game-mod owns the
content revision.  No owner may use this decision to add a direct harness-to-host bypass or a
gateway-to-harness dependency.

This is an additive candidate profile.  Adding a kind, enum member, error code, binding member, or
state after acceptance requires the compatibility classification and negotiated profile/digest
specified by the admitted schema.  Existing closed profiles cannot silently absorb these members.

## Consequences

- Consumers have one explicit, recomputable binding identity and one explicit fail-closed terminal
  state instead of three private interpretations of "the same lookup".
- The identity tuple is the existing owner comparison, so adopting this contract does not require a
  consumer to change what it already treats as the same owner.
- This decision establishes no route, transport, registry, host support, consumer implementation,
  or end-to-end game-information tool; those remain unverified until the named owners pin the digest
  and record the route decision.
