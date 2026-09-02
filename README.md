# sts2-protocol

Status: deterministic POC contract owner for the accepted sixth STS2 build target. The `poc-v1`
artifact is release-like and local-only; no public release or runtime compatibility is claimed.

## Ownership and consumers

Protocol maintainers own only genuinely shared, language-neutral, transport-neutral contract
artifacts. Named prospective consumers are `sts2-game-core`, `sts2-gateway`, `sts2-mcp-server`,
and `sts2-harness`, each consuming an explicitly accepted artifact rather than importing another
boundary's implementation. `sts2-game-mod` remains authoritative for host access and game
mutation; it may consume a published neutral artifact only after a separate ownership decision.

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
