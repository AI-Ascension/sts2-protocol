# Repository Instructions for Coding Agents

## Scope and authority

These instructions apply to `sts2-protocol`. Direct user instructions take precedence. The
canonical detailed rules are in:

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PRODUCT.md`](docs/PRODUCT.md)
- [`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md)
- [`docs/TESTING.md`](docs/TESTING.md)
- [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md)
- [`docs/LICENSING.md`](docs/LICENSING.md)
- [`docs/WORKFLOWS.md`](docs/WORKFLOWS.md)
- [`docs/POLICY_AS_CODE.md`](docs/POLICY_AS_CODE.md)
- [`RELEASING.md`](RELEASING.md)

## Target contract

This is the accepted sixth target for the current build-completion run, with an initialized neutral
metadata package. It owns only shared language-neutral, transport-neutral artifacts with named
consumers. Do not infer a public release or consumer integration from a repeated field name,
historical corpus, reference checkout, or successful policy/build command.

Before adding an exported contract, record its canonical owner, at least two named consumers,
namespace and lifetime, version/profile, serialization rules, unknown-value behavior, compatibility
classification, provenance, and deterministic conformance case. A boundary-specific item stays with
its owner.

The protocol target MUST NOT own game rules, host objects, loader metadata, main-thread dispatch,
game HTTP routes, gateway lifecycle/routing, MCP framing/tool catalogs, model/provider behavior,
credentials, persistence, process control, or mutation authority. The game-mod owns host authority;
the gateway owns lifecycle and routing; MCP remains a thin adapter; the harness owns coordination,
experiments, and artifact lineage.

## Safety and provenance

- Keep protocol data inert: no transport, host, process, filesystem, clock, provider, or network access;
  deadline clock values are metadata only.
- Keep runtime communication and compile-time dependency graphs separate.
- Do not copy, vendor, transliterate, or use product/reference implementation source.
- Do not add proprietary host files, saves, credentials, personal paths, or generated build output.
- Keep Python source and package metadata out of the repository.
- Preserve existing target-owned experiments and do not create an empty product crate or a second
  product crate.
- Do not initialize Git, commit, push, publish, install, deploy, launch a game, or call a provider.

## Before editing

Inspect the target tree and relevant planning inputs. Preserve unrelated files. Keep each change
focused and state whether it changes a contract, schema, consumer, version, or decision record.
Architecture and public-contract changes require a decision record under `docs/decisions/`.

## Required local validation

From the target root, run and report:

```bash
cargo metadata --locked --no-deps --format-version 1
cargo run --locked --offline --package repo-policy -- --strict
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --workspace --all-targets --all-features
```

Schema, golden, and implementation-neutral conformance checks become required when a real contract
is initialized. An unavailable runtime or consumer is unverified, never a pass.
