# Changelog

All notable user-visible changes to this project will be documented here. The project follows
Semantic Versioning once a protocol artifact or repository release exists.

## Unreleased

### Added

- Preserved the unadmitted co-op source/schema/test proposal on a separate branch, with structural
  predicate names that confer no mutation authority. Admission remains blocked pending at least
  two actual named consumers and producer/consumer conformance; no co-op release is established.

- Target-local repository governance, policy tooling, and least-privilege CI foundations.
- The accepted-sixth-target decision and the protocol ownership/dependency-direction decision.
- Documentation defining the narrow neutral-contract boundary and named prospective consumers.
- The initial `sts2-protocol` package with typed neutral metadata, schema, golden fixtures, and an
  implementation-neutral conformance test.
- The narrow `poc-v1` contract, release-like artifact directory, invalid action fixture, and
  deterministic conformance tests for downstream artifact consumers.
- The bounded `runtime-v1` contract, release-like artifact, golden messages, fixed
  `show_runtime_probe` action, effect witness, and stale-generation conformance tests.
- The separate `runtime-v2` contract, release-like artifact, bounded `end_turn` action, explicit
  lifecycle outcomes, stable operation identity, reconciliation vectors, and conformance tests.
- The separate `runtime-v3-gameplay` fair-play projection, typed host-generated legal-action
  contract, full-run state/action family, explicit wait/recovery messages, artifact bundle, and
  deterministic conformance tests.

### Changed

- Decomposed Runtime-v3 shape validation and its strictness matrix into bounded functions while
  preserving schema bytes, validation semantics, and existing message/payload mutation coverage.
- No transport, host, game, gateway, MCP, model, provider, storage, or harness behavior was added;
  consumer mappings use copied release-like files rather than protocol implementation dependencies.
  Runtime-v1, Runtime-v2, and Runtime-v3 remain inert contracts; consumer and live gameplay
  compatibility are unverified.

### Deprecated

- Nothing.

### Removed

- Nothing.

### Fixed

- Historical wire closure (PR #9) for the frozen neutral-metadata, `poc-v1`, and `runtime-v2`
  profiles. Required nullable members must now be present with an explicit `null`; omitting the
  member is rejected instead of silently deserializing to `None`. Unknown members of neutral
  objects are rejected, and duplicate keys inside POC nullable objects are rejected before an
  intermediate JSON value can collapse them. Neutral provenance `license` uses the frozen schema's
  narrower alphanumeric/underscore/dot/hyphen alphabet rather than the identity alphabet.
  Consumers that omitted required nullable members, sent duplicate keys, or used `:` or `/` in
  `license` must update their emitters. No schema bytes, artifact bytes, schema digest, or golden
  bytes changed; see `docs/COMPATIBILITY.md` "Serialization and migration rules". CI now verifies
  the checked-in `poc-v1`, `runtime-v1`, and `runtime-v2` checksum inventories against actual
  bytes. `runtime-v1` is not covered by this closure: it has no typed Rust wire envelope, so its
  required-nullable and duplicate-key behavior remains schema-only.

### Security

- No credentials, host files, saves, provider data, or runtime authority were introduced.
