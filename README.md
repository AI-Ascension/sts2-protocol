<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/AI-Ascension/.github/main/profile/assets/banner-dark.svg">
  <img alt="AI-Ascension — Inspect how AI requests to a game get fenced, one Rust contract at a time. Runtime contracts: confirmed. Live compatibility: unverified." src="https://raw.githubusercontent.com/AI-Ascension/.github/main/profile/assets/banner-light.svg" width="100%">
</picture>

# sts2-protocol

> **AI-Ascension · neutral metadata contracts (beside the ascent)** — Shared metadata contracts (identity, versions, error envelopes) in language-neutral schemas with golden test vectors.
>
> **Status:** deterministic Runtime-v1/Runtime-v2/runtime-map-v1/seeded-run-v1/co-op contract tests and the read-only coordinator-synchronization consumer path are `confirmed`; the `coop-native-v1` source and serialized component contract is accepted at its recorded digest · native host compatibility, game settlement, seeded-run selection, and multiplayer gameplay remain `unverified`.
> **Proof:** [45-second browser replay](https://ai-ascension.github.io/proof.html) · [Evidence ledger](https://ai-ascension.github.io/evidence.html) · [This repository on the map](https://ai-ascension.github.io/repositories.html#sts2-protocol)
> **Owner:** Protocol maintainers own only genuinely shared, language-neutral, transport-neutral contract artifacts; consumers accept explicit artifacts rather than importing another boundary's implementation.
> **Contribute:** [Organization guide](https://github.com/AI-Ascension/.github/blob/main/CONTRIBUTING.md) · [First tasks](https://ai-ascension.github.io/contributing.html)
>
> AI-Ascension is an independent project. It is not affiliated with or endorsed by Mega Crit or Valve and grants no rights to game files, assets, or marks.

Status: deterministic contract owner for the accepted sixth STS2 build target. The `poc-v1`,
`runtime-v1`, `runtime-v2`, `runtime-map-v1`, and `seeded-run-v1` artifacts are release-like and
local-only; no public release or native host/game compatibility is claimed. The separately admitted
`coop-synchronization-v1` artifact has a read-only gateway producer and MCP reader; that
coordinator-report path does not authorize game effects or establish multiplayer gameplay.
The accepted `coop-native-v1` component artifact adds a strict host-backed catalog and actuation
envelope for the game-mod, gateway, MCP, and harness boundaries. Its producer capture is synthetic,
and live native settlement remains unverified.

## Ownership and consumers

Protocol maintainers own only genuinely shared, language-neutral, transport-neutral contract
artifacts. The accepted `poc-v1` artifact consumers are `sts2-game-core`, `sts2-game-mod`,
`sts2-gateway`, `sts2-harness`, and `sts2-mcp-server`, each consuming an explicitly accepted artifact
rather than importing another boundary's implementation. `sts2-game-mod` remains authoritative for
host access and game mutation; its artifact consumption is limited to local representation mapping.

The current build-completion instruction accepts this repository as a sixth target. The narrow
`poc-v1` contract is documented in [ADR 0004](docs/decisions/0004-minimal-poc-contract.md). It does
not turn this target into a generic common-code bucket. Future exported items need a canonical owner,
at least two real consumers, an independent compatibility/version policy, provenance, and deterministic
conformance before they are added.

## Scope and exclusions

The POC scope is one versioned JSON message family: protocol version, schema digest, provenance,
correlation ID, instance ID, generation, a bounded observation, one typed `use_budget` action, and
accepted/rejected status with an error code. The Runtime-v2 scope is a separate versioned message
family for one bounded `end_turn` operation, with stable operation identity, explicit lifecycle
outcomes, and receipt reconciliation. The prior neutral metadata package remains as foundation
history; each release-like artifact is independent and consumed only after local acceptance.
This repository must not own game rules, host objects, loader or main-thread behavior, game HTTP
routes, gateway lifecycle or routing, MCP framing or tool catalogs, model or provider behavior,
credentials, persistence, process control, or mutation authority.

The runtime topology remains `sts2-harness -> sts2-mcp-server -> sts2-gateway -> sts2-game-mod ->
game host`. It is separate from compile-time consumption of an inert protocol artifact. The gateway
is the lifecycle/routing control plane, MCP is a thin adapter, and the harness owns coordination,
experiments, and artifacts.

## Foundation status

`crates/protocol` contains the typed POC mapping. `schemas/poc-v1.schema.json`, the release-like
bundle under `artifacts/poc-v1/`, and `crates/protocol/tests/poc_conformance.rs` provide source,
artifact, digest, golden, invalid-fixture, and deterministic conformance evidence. No second product
crate or cross-repository path dependency was added.

The five foundation consumer PRs copy and verify only their release-like artifacts. Separately, the
admitted co-op profile records gateway/MCP consumption of its release-like artifact and an executable
coordinator-report exchange. This repository remains an inert contract owner: native host/game
execution and provider, package-publication, or public-release operations belong to the named consumer
boundaries or remain unverified. Local build, schema, and golden results establish target-local
static/serialization evidence; the co-op exchange cannot establish host compatibility, game
settlement, multiplayer behavior, or release readiness.

## Provenance and validation

This target contains original foundation documentation and Rust governance tooling under the MIT
license. Planning documents were read-only structural or decision inputs; product source,
historical implementation, proprietary game files, saves, credentials, and
generated output are not copied here. Future schemas and fixtures must record source, license,
generator, version/profile, and digest, and must not contain machine paths or private data.

Run the local foundation entrypoint from this directory:

```bash
cargo metadata --locked --no-deps --format-version 1
cargo run --locked --offline --package repo-policy -- --strict
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --workspace --all-targets --all-features
```

See [the architecture](docs/ARCHITECTURE.md), [the product boundary](docs/PRODUCT.md),
[the repository layout](docs/REPOSITORY_LAYOUT.md), and [the policy guide](docs/POLICY_AS_CODE.md)
for the target-local rules.

## `runtime-v1` vertical-slice contract

The checked-in [`runtime-v1` artifact](artifacts/runtime-v1/README.md) is the canonical contract for
the first bounded runtime slice. It binds one state response, one safe host-visible
`show_runtime_probe` action, a fresh observation/effect witness, and a stable stale-generation
rejection. Its named consumers are `sts2-game-mod`, `sts2-gateway`, `sts2-harness`, and
`sts2-mcp-server`; each consumer uses a copied release-like artifact rather than a cross-repository
implementation dependency.

Schema and golden/conformance checks for this profile are confirmed in the protocol repository.
That is contract evidence only: live host state, the managed main-thread callback, network
compatibility, profile behavior, and game-rule mutation remain unverified.

## `runtime-v2` gameplay-operation contract

The checked-in [`runtime-v2` artifact](artifacts/runtime-v2/README.md) defines the bounded `end_turn`
action. Its observation carries `combat_phase`, `turn_index`, `host_ready`, and `generation`; the
action is legal only for `combat/player_turn`. The deterministic contract vector describes generation
4/turn 2 transitioning to generation 5/turn 3 and requires a `turn_end_settled` witness for
authoritative settlement.

Runtime-v2 preserves `accepted`, `settled`, `rejected`, `unknown`, and `cancelled` as distinct
outcomes. `operation_id` is stable across duplicate requests and reconciliation; a conflicting
reuse returns `idempotency_conflict`, and an uncertain result must be reconciled rather than blindly
retried. These are neutral wire semantics. Game legality, host authority, leases, transport,
reconciliation storage, and live compatibility remain with the named consumer boundaries and are
unverified by this repository.

## `seeded-run-v1` launch contract

The checked-in [`seeded-run-v1` artifact](artifacts/seeded-run-v1/README.md) defines a bounded,
explicitly seeded launch operation. The harness supplies the requested seed, while the game host
remains authoritative and must read back the canonical seed and a `run_started` effect before a
result is `settled`. Every start request and lifecycle result carries the selected native context:
standard mode, Ironclad, ascension, modifiers, ordered acts, selection policy, profile baseline,
save policy, and separate game/mod compatibility identities.

`context_digest` is the SHA-256 of those context fields in the profile's specified compact canonical
JSON order. `accepted` means admission only; `unknown` requires read-only reconciliation using the
same operation identity, and a reconciliation request carries no launch context. The checked-in
schema, artifact, manifest, goldens, checksum inventory, digest vector, and conformance case prove
bounded protocol serialization and lifecycle shape only. They do not prove consumer mapping,
licensed-host selection, profile or save safety, live run settlement, or release compatibility; see
[ADR 0015](docs/decisions/0015-seeded-run-selection-context.md).

## `runtime-v4-expert` source/component contract

At current protocol main
[`d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404`](https://github.com/AI-Ascension/sts2-protocol/commit/d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404),
the additive `runtime-v4-expert` observation and `runtime-v4-expert-action` transport artifacts,
manifests, checksum inventories, goldens, typed validators, and conformance cases are present.
Their schema digests are `0ee034d5da83f34e9fa0ba23038738d56ef8cfccb1c6e752af3ab63d212c8e42` and
`393318bda8c3522c0ecbacc78b95471a9f4dc3f825169d2048f4c74a7b7f2929`. This confirms the protocol
source, serialization, and conformance boundary. Consumer mapping, native host legality, settled
effects, provider execution, deployment, release, and live compatibility remain unverified.

## `runtime-map-v1` read-only map profile

The checked-in [`runtime-map-v1` artifact](artifacts/runtime-map-v1/README.md) is the additive,
host-owned read-only map projection. At current protocol main commit
[`d3ab5fc`](https://github.com/AI-Ascension/sts2-protocol/commit/d3ab5fca7d9d74bb31eeb3e5b343d8024ee44404),
its schema digest is `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b`. The
normative schema, release-like artifact copy, seven-entry checksum inventory, three golden files,
typed map decoder, and implementation-neutral conformance case are present and hash-bound. This
is source and serialization evidence in the protocol repository. The manifest names
`sts2-game-mod`, `sts2-gateway`, `sts2-harness`, `sts2-mcp-server`, and
`ascension-map-visualizer` as consumers. The [gateway map PR](https://github.com/AI-Ascension/sts2-gateway/pull/26)
and [MCP map PR](https://github.com/AI-Ascension/sts2-mcp-server/pull/30) are separate consumer
source changes. Protocol checks do not attest consumer behavior; selected current-head artifact-copy
and bounded synthetic gateway/MCP checks are recorded separately. Host extraction, visualizer
validation, native map visibility, navigation settlement, gameplay, and release readiness remain
separate boundaries.

The profile carries only player-visible map projection data and generation-bound read-only action
bindings. It does not expose hidden map state, future outcomes, host objects, provider output, or
mutation authority. Complete cross-repository integration, native map visibility, navigation
settlement, gameplay, and release readiness remain unverified.

## Candidate `runtime-v4-expert-rest-action-v1` profile

The [`runtime-v4-expert-rest-action-v1` artifact](artifacts/runtime-v4-expert-rest-action/README.md)
is a candidate additive transport for native rest-site options and typed Smith/Mend selector
follow-up actions. Its schema digest is
`bb3555fae28eb1f79d08a15e9884696a579e4c20836f5016509f17e0f4c36fbd`; the existing potion action
profile remains unchanged. The artifact includes 16 synthetic goldens, 22 structural and semantic
mutation checks, two serialized Smith/Mend producer-shaped lifecycles, provenance, and an HTTP assignment
envelope owned by the gateway.

This profile has no admitted consumers yet. The game-mod must first emit the exact serialized
envelope from native evidence, then gateway, MCP, and harness owners must independently validate
their integrations. Protocol schema and conformance results establish candidate contract evidence;
they do not establish a producer, route, host compatibility, or live settlement. See
[ADR 0032](docs/decisions/0032-runtime-v4-expert-rest-action-v1.md).

The separate `coop-synchronization-v1` artifact describes the attached gateway's recent
coordinator-reported peer agreement. Its full response is produced by the gateway and read by the
executable MCP profile. The recorded exchange covers coordinator report convergence and fencing
with zero downstream game connections; it has no action, vote, or shared-effect payload. See
[ADR 0013](docs/decisions/0013-coop-synchronization-admission.md) and the [MCP executable
evidence](https://github.com/AI-Ascension/sts2-mcp-server/blob/main/docs/evidence/coop-synchronization-20260906.md)
for identity lifetimes, conformance, and the distinction from multiplayer gameplay.
