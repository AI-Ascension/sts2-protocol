# Compatibility Policy

## Independent dimensions

The unpublished gameplay proposals require the explicit
[conformance correction and digest migration](decisions/0009-proposed-contract-conformance-corrections.md).
Schema validation alone is insufficient; that decision lists mandatory semantic checks and the
incompatible earlier proposal. Co-op remains a source-only prototype, not an admitted artifact.

Protocol, schema/profile, repository, consumer, game-host, loader/ABI, gateway, MCP, harness,
provider/model, and artifact versions are independent. A matching number or field name does not
establish compatibility. This target can describe a neutral artifact; it cannot claim host, service,
MCP, model, provider, or end-to-end compatibility.

## Current status

The accepted sixth-target decision permits this repository to be prepared and implemented. The
`poc-v1` schema, release-like bundle, and golden/conformance files provide target-local
source-derived/serialization evidence. Consumer PRs verify copied artifact metadata and mappings, but no
public release or runtime integration is established. The STS2 host baseline used elsewhere in
planning is not protocol runtime evidence. Host, gateway, MCP, harness, provider, and release
boundaries remain unverified.

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
| `runtime-v1` | game-mod, gateway, harness, MCP | Schema, artifact bytes, goldens, and conformance are confirmed | Host callback, network route, disposable profile, and game compatibility |
| `runtime-v2` | game-mod, gateway, harness, MCP | Separate schema, artifact bytes, lifecycle goldens, and conformance are confirmed | Consumer mapping, operation ledger, host settlement, reconciliation, and game compatibility |
| `runtime-v3-gameplay` | game-mod, gateway, harness, MCP | Source/package schema, sanitized goldens, manifest, digest inventory, and local conformance are confirmed | Consumer mappings, fair-play host projection, gateway/MCP transport, Exo execution, and live full-run compatibility |

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

The harness's Exo request and decision envelope is intentionally provider-owned. No separate
`runtime-v3-gameplay-llm` or `agent-decision-v1` artifact is admitted without an independent
cross-repository producer, consumer, and conformance requirement; this keeps model policy out of
the neutral protocol owner.
