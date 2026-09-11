# Compatibility Policy

## Independent dimensions

The admitted co-op profile is `coop-synchronization-v1`: gateway serialization produces
coordinator-reported synchronization metadata and MCP reads it under the declared profile. This
profile carries no action, vote, shared-effect, or host-game authority. The preserved six-family
`coop-gameplay-v1` prototype and other gameplay proposals remain unadmitted; their earlier wire is
rejected. See [ADR 0013](decisions/0013-coop-synchronization-admission.md) and the explicit
[conformance correction and digest migration](decisions/0009-proposed-contract-conformance-corrections.md)
for the distinct profiles and required semantic checks.

The `coop-native-v1` artifact is accepted for source and serialized component integration under
[ADR 0033](decisions/0033-native-coop-component-acceptance.md). It is a separate actuation profile
with a host-backed legal catalog, generation and identity fencing, explicit receipts, and
same-operation recovery. Its component status does not establish native host compatibility or live
multiplayer settlement; `live_status` remains `unverified` pending the owning boundaries' two-peer
runtime evidence.

Protocol, schema/profile, repository, consumer, game-host, loader/ABI, gateway, MCP, harness,
provider/model, and artifact versions are independent. A matching number or field name does not
establish compatibility. This target can describe a neutral artifact; it cannot claim host, service,
MCP, model, provider, or end-to-end compatibility.

## Current status

The accepted sixth-target decision permits this repository to be prepared and implemented. The
`poc-v1` schema, release-like bundle, and golden/conformance files provide target-local
source-derived/serialization evidence. Current protocol main commit
`d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404` also contains the `runtime-map-v1` and `seeded-run-v1`
schemas, release-like artifacts, checksum inventories, goldens, and conformance cases. The map
artifact has schema digest `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b`;
the seeded-run artifact binds its selected-context digest vector and lifecycle shape. These provide
source and serialization evidence only. Consumer PRs verify copied artifact metadata and mappings,
but no public release or complete runtime integration is established. Separate current-head
gateway/MCP component evidence covers selected map artifact-copy and bounded synthetic exchange
checks; those records are outside this protocol-only result. The STS2 host baseline used elsewhere
in planning is not protocol runtime evidence. Host extraction, visualizer, native map, navigation,
seeded-run selection, gameplay, and release boundaries remain unverified.

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
| `runtime-v4-expert` / `runtime-v4-expert-action` | game-mod, gateway, harness, MCP | At current protocol main `d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404`, source schemas, copied artifacts, manifests, goldens, checksum inventories, typed validators, and conformance cases are confirmed | Consumer mapping, host legality, settled effects, provider execution, deployment, release, and live compatibility |
| `runtime-map-v1` | game-mod, gateway, harness, MCP, map visualizer | At current protocol main `d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404`, schema digest `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b` and the schema/artifact/checksum/golden/conformance set are confirmed by source and serialization checks | Protocol checks do not attest consumer behavior; host extraction, visualizer validation, native map visibility, navigation settlement, gameplay, and release remain separate boundaries |
| `seeded-run-v1` | game-mod, gateway, harness, MCP | At current protocol main `d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404`, schema, selected-context digest vector, lifecycle goldens, manifest, checksum inventory, and conformance are confirmed by source and serialization checks | Consumer mapping, licensed-host selection, profile/save safety, live run settlement, gameplay, and release compatibility |
| `runtime-v4-expert-rest-action-v1` | none admitted; prospective game-mod, gateway, MCP, harness | Candidate schema/artifact bytes, provenance, 16 goldens, 22 mutation fixtures, two serialized producer-shaped lifecycles, checksums, and strict conformance at digest `bb3555fae28eb1f79d08a15e9884696a579e4c20836f5016509f17e0f4c36fbd` | Managed native producer, route, consumer mappings, host compatibility, selector settlement, gameplay, and release remain pending |

| `coop-native-v1` | game-mod, gateway, harness, MCP | Source producer, exact schema/artifact bytes, legal-catalog/action/vote/rejoin/receipt goldens, and component boundary records are accepted at digest `2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629` | Native two-peer launch, host/client settlement, native checksum agreement, model/provider execution, disconnect/rejoin convergence, and release compatibility |

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

`runtime-v4-expert-rest-action-v1` is additive-compatible with the existing potion profile because
it has an independent protocol version, schema, artifact, action vocabulary, and digest. The
candidate's empty consumer list is intentional. Prospective owners must pin the exact digest and
prove serialized producer or consumer round trips before adoption; protocol conformance does not
promote the profile or establish native rest-option or selector settlement.

The REST conformance correction selects Mend and Lift evidence validation by the evidence kind.
This is contract-compatible: it restores validation of existing native-completion alternatives
without requiring fields from an unrelated numeric evidence variant. Schema, artifact, digest,
golden bytes, and candidate consumer status remain unchanged. Consumer and native execution
verification remain separate requirements.

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
