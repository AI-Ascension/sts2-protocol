# Game-information lookup-binding discovery and observation contract

- Contract identifier: `sts2.protocol/game-information-lookup-binding-v1`
- Profile: `game-information-lookup-binding-v1`
- Status: proposed candidate; no admitted consumer and no route decision
- Canonical owner: `sts2-protocol`
- Named consumers: `sts2-harness`, `sts2-gateway`, `sts2-mcp-server`
- Decision record: `docs/decisions/0040-game-information-lookup-binding-discovery-and-observation.md`

This is the normative section for the initial discovery and observation-binding behaviour of the
game-information lookup transport.  It defines how a consumer learns which lookup binding is legal
at episode start, how an observation is bound to that binding, and what a consumer must reject.  The
machine-checkable witness is the artifact
`artifacts/game-information-lookup-binding-v1` with schema
`schemas/game-information-lookup-binding-v1.schema.json` and conformance case
`conformance/cases/game-information-lookup-binding-v1.json`.

The key words MUST, MUST NOT, SHOULD, and MAY are to be read as described in RFC 2119.

## Non-goals

This contract is inert and read-only.  It defines no route, transport, endpoint, method, host
adapter, registry, clock, credential, lifecycle, persistence mechanism, or tool.  It does not
authorize a consumer to reach a host directly, and it establishes no gateway-to-harness dependency.
Harness-side integration wiring and native exact-build acceptance are owned by their own issues.

## The stable binding identifier

A lookup binding is identified by `binding_id`, the lowercase hexadecimal SHA-256 digest of the
canonical identity input.

The identity input is exactly these eight members, which are the members the lookup owner already
compares when it decides whether two bindings share an owner:

`project_id`, `run_id`, `episode_id`, `agent_id`, `game_profile`, `content_manifest_id`, `locale`,
`authority_epoch`.

The digest input is the compact UTF-8 JSON encoding of that object with its members sorted
lexicographically:

```
{"agent_id":"agent-3","authority_epoch":7,"content_manifest_id":"content-1","episode_id":"episode-7","game_profile":"sts2-native-v1","locale":"en-US","project_id":"proj-1","run_id":"run-42"}
```

That input yields
`58fea90991138ea6fb635df1f5eadd08973ec63eba456d135578677ffee61cfc`.  Compact sorted-key JSON is
required because a sorted-map serializer produces exactly this encoding in each named consumer
language, so three independent implementations recompute the same digest without a shared library.

`binding_id` is stable for a given identity input and changes when any of the eight members changes.
It is never a display name, a process-local handle, or a game-state digest.

## Owning boundary of each member

| Member | Owning boundary |
| --- | --- |
| `project_id`, `run_id`, `episode_id`, `agent_id` | `sts2-harness` scope |
| `authority_epoch` | `sts2-harness` |
| `game_profile`, `content_manifest_id` | `sts2-game-mod` content revision |
| `locale` | `sts2-gateway` |
| `instance_id` | `sts2-gateway` instance lease; not part of the digest |

## Envelope and shapes

Every document is an envelope with `protocol_version`, `schema_digest`, `provenance`,
`correlation_id`, `kind`, `binding`, `discovery`, `observation`, and `error`.  The `kind` member is
one of `lookup_binding_discovery_response`, `lookup_binding_observation_response`, or
`error_response`.

`binding` carries `binding_id`, `scope`, `game_profile`, `content_manifest_id`, `locale`,
`authority_epoch`, `instance_id`, and the three-member `authority` declaration.

`discovery` carries `observation_state` (one of `not_yet_observed`, `observed`,
`reobserve_required`, `reobserve_exhausted`), `required_capabilities` (profile and schema digest),
and `reobserve` (`attempts` and `supersedes_observation_id`, or null).

`observation` carries `observation_id`, `binding_id`, `snapshot_id`, and `state_generation`.

`error` carries a closed `code`, a nullable `field`, and a nullable `reason`.

## Normative rules

### LBR-1: Version and profile fail explicit

A consumer MUST reject a document whose `protocol_version` is not the negotiated profile or whose
`schema_digest` is not the negotiated digest, with `unsupported_version`.  A consumer MUST NOT
coerce an unknown version, unknown state, unknown kind, unknown error code, or unknown member to a
known value.

### LBR-2: Discovery precedes observation

A consumer MUST be able to discover a binding before it may observe it.  A discovery response
reports `not_yet_observed`, the required capability, and no observation.  A consumer MUST NOT
observe a binding that it did not first discover.

### LBR-3: The binding identifier is defined, not negotiated

`binding_id` MUST be the digest defined in this section.  Two documents share a binding identifier
if and only if they declare the same canonical identity input.

### LBR-4: `binding_id` is recomputed, never trusted

A consumer MUST recompute `binding_id` from the declared identity members and MUST reject a document
whose producer-supplied `binding_id` differs from the recomputed value, with `invalid_identity`.  A
producer-supplied `binding_id` is not authority.

### LBR-5: Each member has one owning boundary

Each identity member MUST be supplied by its owning boundary as recorded in the table above.  A
consumer MUST NOT accept a scope, epoch, content revision, locale, or lease that the owning boundary
did not supply.

### LBR-6: Owner equality uses the identity input

Two bindings are the same binding if and only if all eight identity members are equal.  A consumer
MUST compare exactly that tuple; it MUST NOT extend or reduce the comparison with a display name, an
`instance_id`, a snapshot, or a process-local handle.

### LBR-7: Scope and instance fences are enforced

A consumer MUST reject a document whose `binding.scope` differs from the context scope, or whose
`binding.instance_id` differs from the context instance, with `denied_scope`.  `instance_id` is
fenced separately from the digest: a lease change is a scope denial, not a new binding identity.

### LBR-8: Required capability fails closed

A consumer MUST reject a document whose `required_capabilities.profile` was not negotiated with
`missing_capability`.  A consumer MUST NOT proceed on an unnegotiated capability because the
document is otherwise well formed.

### LBR-9: An observation is bound to one binding identifier

When a document carries an observation, `observation.binding_id` MUST equal `binding.binding_id`.  A
consumer MUST reject an observation bound to any other identity with `mixed_binding`; it MUST NOT
merge fields from a differently bound observation.

### LBR-10: State, shape, and error code agree

The `kind`, `observation_state`, `observation`, `reobserve`, and `error` members MUST agree.  A
`discovery_response` reports `not_yet_observed` with no observation.  An `observation_response`
reports `observed` with an observation and no re-observe record, or `reobserve_required` or
`reobserve_exhausted` with no observation and a re-observe record.  A response that disagrees with
its own state, or whose `error.code` contradicts its state, MUST be rejected with `malformed`.  When
the context retains an observation, `reobserve.supersedes_observation_id` MUST name that retained
observation.

### LBR-11: Staleness is context-relative and re-observation preserves identity

A consumer MUST reject a document whose `observation.state_generation` is behind the generation the
context retains, with `stale_snapshot`.  A stale observation MUST be discarded and re-observed under
the same `binding_id` with a new `observation_id`.  Re-observation MUST NOT re-discover: a changed
identity input requires a fresh discovery, and a re-observation MUST NOT silently rebind to a
different owner.

### LBR-12: Re-observation impossible fails closed

When re-observation is impossible, the binding MUST fail closed.  The state MUST be
`reobserve_exhausted`, `reobserve.attempts` MUST be at least one, `observation` MUST be null, and the
terminal code MUST be `reobserve_unavailable`.  A consumer MUST NOT fall back to a stale, partial,
or fabricated observation, and MUST NOT report success.  A document that asserts
`reobserve_unavailable` against a state other than `reobserve_exhausted` MUST be rejected with
`malformed`.

## Deterministic witness precedence

A conforming witness evaluates a document in this order and reports the first failure, so that two
independent implementations agree on the code for a given input:

1. duplicate object member detection, then document schema validity;
2. `protocol_version` and `schema_digest` (LBR-1);
3. `binding_id` recomputation (LBR-3, LBR-4);
4. scope fence (LBR-7);
5. instance fence (LBR-7);
6. required capability (LBR-8);
7. observation binding identity (LBR-9);
8. staleness (LBR-11);
9. state, shape, and error-code agreement (LBR-10, LBR-12).

## Error vocabulary

The closed error codes are `unsupported_version`, `invalid_identity`, `denied_scope`,
`missing_capability`, `stale_snapshot`, `mixed_binding`, `reobserve_unavailable`, and `malformed`.
An error retains the request correlation supplied by the boundary owner and MUST NOT disclose a
hidden identifier merely to explain a denial.

## Conformance

The case `CT-GAME-INFORMATION-LOOKUP-BINDING-V1-001` exercises at least happy-path discovery,
wrong-owner rejection, stale and re-observe, and re-observation-impossible fail-closed.  Two
witnesses, one Rust and one JavaScript, validate the same bytes and vector identifiers.  This is
protocol and serialization evidence only: consumer source adoption, gateway or MCP tool
registration, host extraction, live snapshots, and end-to-end reachability remain unverified until
the named owners pin the artifact digest.

## Open decision

The transport that serves discovery and observation is not decided by this contract and MUST NOT be
invented by an implementation.  Whether discovery is served by the gateway, by the MCP adapter, or
by a dedicated channel is an open question for `sts2-harness#127` E2 and the gateway owners.  This
contract fixes only the document shapes, the binding identifier, and the failure behaviour.
