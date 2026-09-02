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

Status: Wave 2 initialized target for the accepted sixth STS2 build target. The initial package is a
small neutral metadata seam; no transport or product behavior is present.

## Ownership and consumers

Protocol maintainers own only genuinely shared, language-neutral, transport-neutral contract
artifacts. Named prospective consumers are `sts2-game-core`, `sts2-gateway`, `sts2-mcp-server`,
and `sts2-harness`, each consuming an explicitly accepted artifact rather than importing another
boundary's implementation. `sts2-game-mod` remains authoritative for host access and game
mutation; it may consume a published neutral artifact only after a separate ownership decision.

The current build-completion instruction accepts this repository as a sixth target. The initial
package decision accepts only the narrow metadata seam documented in [ADR 0003](docs/decisions/0003-initial-neutral-metadata-package.md).
It does not turn this target into a generic common-code bucket. Future exported items need a canonical
owner, at least two real consumers, an independent compatibility/version policy, provenance, and
deterministic conformance before they are added.

## Scope and exclusions

The initialized scope is limited to shared identity/correlation/lineage metadata, version/profile
and digest descriptors, selected lifecycle/deadline/cancellation/sequence metadata, neutral error
envelope metadata, and a manifest with provenance. This repository must not own game rules, host
objects, loader or main-thread behavior, game HTTP routes, gateway lifecycle or routing, MCP framing
or tool catalogs, model or provider behavior, credentials, persistence, process control, or mutation
authority.

The runtime topology remains `sts2-harness -> sts2-mcp-server -> sts2-gateway -> sts2-game-mod ->
game host`. It is separate from compile-time consumption of an inert protocol artifact. The gateway
is the lifecycle/routing control plane, MCP is a thin adapter, and the harness owns coordination,
experiments, and artifacts.

## Foundation status

`crates/protocol` now contains the target-owned package. `schemas/common`, `conformance`, and
`crates/protocol/tests` contain hand-authored schema/fixture inputs and deterministic tests for this
seam; none is generated output. No second product crate or cross-repository path dependency was added.

No live consumer, host, gateway, MCP peer, harness run, provider, package publication, or release has
been exercised by this repository. Local build, schema, and golden results establish only target-local
static/behavioral evidence; they cannot establish host compatibility, wire integration, end-to-end
behavior, or release readiness.

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
