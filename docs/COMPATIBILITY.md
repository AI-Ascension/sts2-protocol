# Compatibility Policy

## Independent dimensions

The unpublished gameplay proposals require the explicit
[conformance correction and digest migration](decisions/0009-proposed-contract-conformance-corrections.md).
Schema validation alone is insufficient; that decision lists mandatory semantic checks and the
incompatible earlier proposal. The unadmitted six-family co-op prototype remains preserved in
Git history. [ADR 0013](decisions/0013-coop-synchronization-admission.md) replaces its provisional
exports with `coop-synchronization-v1`, consumed completely by gateway serialization and MCP
projection. Its profile and digest are distinct; the old `coop-gameplay-v1` wire is rejected.
This admits coordinator-reported synchronization metadata, not co-op actuation or host effects.

Protocol, schema/profile, repository, consumer, game-host, loader/ABI, gateway, MCP, harness,
provider/model, and artifact versions are independent. A matching number or field name does not
establish compatibility. This target can describe a neutral artifact; it cannot claim host, service,
MCP, model, provider, or end-to-end compatibility.

## Current status

The accepted sixth-target decision permits this repository to be prepared and implemented. The
`poc-v1` schema, release-like bundle, and golden/conformance files provide target-local
source-derived/serialization evidence. As of 2026-09-08, merged protocol main commit
`b3d3034f32e68d70c9e681f906ee37d74db153c4` also contains the `runtime-map-v1` schema, release-like
artifact, checksum inventory, goldens, typed decoder, and conformance case at schema digest
`ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b`; these provide source and
serialization evidence only. Consumer PRs verify copied artifact metadata and mappings, but no
public release or complete runtime integration is established. Separate current-head gateway/MCP
component evidence covers selected map artifact-copy and bounded synthetic exchange checks; those
records are outside this protocol-only result. The STS2 host baseline used elsewhere in planning is
not protocol runtime evidence. Host extraction, visualizer, native map, navigation, gameplay, and
release boundaries remain unverified.

## Future compatibility classes

- **Contract-compatible:** preserves accepted requirements and fixtures.
- **Additive-compatible:** adds optional data without changing valid existing behavior.
- **Deprecated-compatible:** preserves behavior during a documented migration window.
- **Safety correction:** changes dangerous behavior with explicit impact and migration notes.
- **Breaking:** changes meaning, required data, canonical encoding, or removal semantics and requires
  an approved major/profile migration.

Every change identifies the affected requirement, owner, consumer set, profile/version, fixture,
digest, and evidence level. A schema parser pass or successful Rust build is not semantic
compatibility evidence.

## Serialization and migration rules

The parser closure correction preserves all normative schemas, artifact digests, and golden bytes.
It rejects previously tolerated schema-invalid input: omitted required nullable members in neutral
metadata and Runtime-v2, unknown neutral object members, and ambiguous duplicate POC object keys.
Neutral provenance licenses also retain the schema's narrower alphanumeric/underscore/dot/hyphen
alphabet rather than the identity alphabet, which additionally permits colon and slash.
Consumers must send explicit `null` for required nullable fields and unique, recognized members.
This restores the existing contract rather than adding a profile or changing a valid message.

The same closure applies to `runtime-v1` through the typed `RuntimeMessage` envelope
([ADR 0011](decisions/0011-runtime-v1-wire-closure.md)): omitted nullable members, unknown
members, and duplicate keys are rejected, and the typed validator enforces exactly the schema's
metadata, identity, bound, and kind/status shape rules. The `runtime-v1` schema digest, manifest,
checksum inventory, and golden bytes are unchanged; the change is additive-compatible and Rust API
only.

Canonical bytes, field names, enum spellings, optionality, ordering, bounds, and unknown-value
handling are part of a future artifact's contract. A consumer must be able to reject an unsupported
profile before mutation or authorization. A digest change is release-visible. During a migration,
the old owner profile remains readable only for the declared window; adapters preserve rejected,
cancelled, accepted, settled, and unknown outcomes rather than collapsing them.

## Evidence levels

Use `confirmed`, `source-derived`, `inferred`, `proposed`, `unverified`, and `unsupported`
consistently. Protocol-only evidence can establish static artifact properties. It cannot establish a
live game load, host-thread behavior, gateway lifecycle, MCP handshake, harness experiment, provider
call, package installation, or release verification.

## Runtime profile matrix

| Profile | Consumers | Current evidence | Unverified boundary |
| --- | --- | --- | --- |
| `runtime-v1` | game-mod, gateway, harness, MCP | Schema, artifact bytes, goldens, typed envelope round-trip, wire closure, and conformance are confirmed | Host callback, network route, disposable profile, and game compatibility |
| `runtime-v2` | game-mod, gateway, harness, MCP | Separate schema, artifact bytes, lifecycle goldens, and conformance are confirmed | Consumer mapping, operation ledger, host settlement, reconciliation, and game compatibility |
| `runtime-v3-gameplay` | game-mod, gateway, harness, MCP | Source/package schema, sanitized goldens, manifest, digest inventory, and local conformance are confirmed | Consumer mappings, fair-play host projection, gateway/MCP transport, Exo execution, and live full-run compatibility |
| `runtime-map-v1` | game-mod, gateway, harness, MCP, map visualizer | At merged protocol main `b3d3034f32e68d70c9e681f906ee37d74db153c4`, schema digest `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b` and the schema/artifact/checksum/golden/conformance set are confirmed by source and serialization checks | Protocol checks do not attest consumer behavior; host extraction, visualizer validation, native map visibility, navigation settlement, gameplay, and release remain separate boundaries |

`runtime-v1` is contract-compatible only when the exact schema digest, provenance, bounds, and
unknown-field behavior are preserved. Its accepted action is a host-visible probe; it is not a
compatibility claim for gameplay mutation.

`runtime-v2` is a separate profile and must not be negotiated as `runtime-v1`. Its exact schema
digest, provenance, bounded observation, fixed `end_turn` action, outcome semantics, and
`operation_id` replay/reconciliation rules must be preserved. A schema or artifact pass remains
protocol-only evidence; it does not establish a live gameplay mutation or settlement.

`runtime-v3-gameplay` is also a separate profile and must not reinterpret Runtime-v2 messages. Its
exact digest, provenance, player-visible field set, typed action variants, current-generation
binding, and explicit unknown/recovery semantics are contract requirements. The profile's presence
does not establish that a host can produce a complete catalog or settle any action.

`runtime-map-v1` is an additive profile with exact schema digest
`ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b` and an independent snapshot
identity of `visible-map-v1`. Consumers must preserve its bounded UTF-8 text, graph, freshness,
generation, and action-binding rules, and must reject an unsupported digest before any host action.
The profile describes a host-owned read-only projection; protocol schema and artifact checks do not
establish map extraction, route compatibility, native map visibility, navigation settlement,
gameplay, or release readiness. Current consumer component evidence remains governed by each
consumer's exact source head and checks.

The harness's Exo request and decision envelope is intentionally provider-owned. No separate
`runtime-v3-gameplay-llm` or `agent-decision-v1` artifact is admitted without an independent
cross-repository producer, consumer, and conformance requirement; this keeps model policy out of
the neutral protocol owner.

The campaign-continuation candidate in ADR 0012 adds `proceed`, `confirm_selection`, and
`cancel_selection` under digest
`8e99cea36b7ede97532348fd8efe302ca79260895265a7bf14ddf7e006d8ff63`.
It requires coordinated migration of game-mod, gateway, MCP and harness. Existing sessions
using the previous digest must finish before the replacement stack starts; mixed revisions
are rejected. The added vocabulary does not establish host support or campaign completion.
