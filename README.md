# sts2-protocol

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
