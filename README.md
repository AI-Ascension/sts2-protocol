<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/AI-Ascension/.github/main/profile/assets/banner-dark.svg">
  <img alt="AI-Ascension — Inspect how AI requests to a game get fenced, one Rust contract at a time. Runtime: unverified. Deterministic tests: confirmed." src="https://raw.githubusercontent.com/AI-Ascension/.github/main/profile/assets/banner-light.svg" width="100%">
</picture>

# sts2-protocol

> **AI-Ascension · neutral metadata contracts (beside the ascent)** — Shared metadata contracts (identity, versions, error envelopes) in language-neutral schemas with golden test vectors.
>
> **Status:** deterministic in-memory tests `confirmed` at the pinned commit · runtime, host, and game compatibility `unverified` · nothing is live.
> **Proof:** [45-second browser replay](https://ai-ascension.github.io/proof.html) · [Evidence ledger](https://ai-ascension.github.io/evidence.html) · [This repository on the map](https://ai-ascension.github.io/repositories.html#sts2-protocol)
> **Owner:** Protocol maintainers own only genuinely shared, language-neutral, transport-neutral contract artifacts; consumers accept explicit artifacts rather than importing another boundary's implementation.
> **Contribute:** [Organization guide](https://github.com/AI-Ascension/.github/blob/main/CONTRIBUTING.md) · [First tasks](https://ai-ascension.github.io/contributing.html)
>
> AI-Ascension is an independent project. It is not affiliated with or endorsed by Mega Crit or Valve and grants no rights to game files, assets, or marks.

Status: deterministic POC contract owner for the accepted sixth STS2 build target. The `poc-v1`
artifact is release-like and local-only; no public release or runtime compatibility is claimed.
The initial package remains a small neutral metadata seam; no transport or product behavior is
present.

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
accepted/rejected status with an error code. The prior neutral metadata package remains as foundation
history, but the POC artifact under `artifacts/poc-v1/` is the only new contract consumed by this run.
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

The five consumer PRs copy and verify only the release-like artifact; this repository has no live
consumer, host, gateway, MCP peer, harness run, provider, package publication, or public release.
Local build, schema, and golden results establish target-local static/serialization evidence; they
cannot establish host compatibility, wire integration, end-to-end behavior, or release readiness.

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
