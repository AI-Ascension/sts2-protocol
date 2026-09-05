# Changelog

All notable user-visible changes to this project will be documented here. The project follows
Semantic Versioning once a protocol artifact or repository release exists.

## Unreleased

### Added

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
- The typed `RuntimeMessage` envelope and `RuntimeProvenance` object for the frozen `runtime-v1`
  profile, with `RuntimeValidationError::ResultShape`, mirroring the Runtime-v2 envelope; see
  `docs/decisions/0011-runtime-v1-wire-closure.md`. Rust API only; no artifact bytes changed.

### Changed

- Split Runtime-v3 action and metadata types into focused modules and removed their two
  handwritten file-budget exemptions, preserving public API, wire bytes, and validation behavior.

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
  bytes. `runtime-v1` was not covered by that closure because it had no typed Rust wire envelope;
  the following entry closes it.
- Runtime-v1 wire closure. `RuntimeMessage` requires every nullable member (`observation`,
  `action`, `status`, `error_code`, `effect_witness`) to be present with an explicit `null`,
  rejects unknown members at the envelope, provenance, observation, action, and witness
  boundaries, rejects duplicate keys before an intermediate JSON value can collapse them, and
  validates metadata, identities, `RUNTIME_MAX_GENERATION`, `RUNTIME_MAX_ACTION_COUNT`, and the
  kind/status member shape. All five `runtime-v1` goldens decode, validate, and re-encode to
  byte-identical canonical JSON. No schema, artifact, digest, or golden bytes changed.

### Security

- No credentials, host files, saves, provider data, or runtime authority were introduced.
