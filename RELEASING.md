# Release Policy and Procedure

This target has an initialized but unreleased neutral protocol package. Any future release of a
neutral schema or Rust contract artifact must be deliberate, immutable, and evidence-backed.
Repository foundation, package publication, and consumer/runtime verification are separate states.

## Authority and scope

Only an explicitly authorized maintainer may publish. Agents may prepare or verify a candidate but
must not push release branches, create or move tags, publish artifacts, upload packages, or deploy.
Protocol releases do not grant compatibility with the game host, gateway, MCP server, harness,
provider, or any proprietary material.

## Version and artifact rules

Use Semantic Versioning for a repository or package release. Keep repository, protocol, schema/profile,
consumer, game-host, and toolchain versions independent. A compatible field addition, deprecation,
safety correction, and breaking semantic change each require an explicit classification and fixture
review. Do not infer compatibility from matching version numbers.

Any published bundle must be reproducible from repository-relative inputs and include its exact
source revision, profile/version, sorted file inventory, SHA-256 digest, generator identity, license
and provenance records, supported consumers, and package allowlist. It must exclude absolute paths,
credentials, saves, proprietary host files, build output, and unrelated implementation source.

## Readiness gates

A protocol candidate is release-ready only when the exact approved commit is identified, required
reviews are complete, policy/format/lint/test/conformance checks pass, schemas and golden fixtures
agree, consumer compatibility evidence is recorded, dependency notices are generated from the exact
lockfile, and package bytes are inspected from a clean build. No current target satisfies this state.

Run the local foundation checks before any future packaging:

```bash
cargo metadata --locked --no-deps --format-version 1
cargo run --locked --offline --package repo-policy -- --strict
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --workspace --all-targets --all-features
```

## Publication and rollback

Publication requires explicit approval, immutable tags/artifacts, checksums, and post-publication
verification from freshly retrieved bytes. Do not rewrite a tag or silently replace a bundle. A
defective release is corrected with a new version while consumers pin the last known-good profile
and digest. Rollback does not delete accepted operations or mutate game state.
