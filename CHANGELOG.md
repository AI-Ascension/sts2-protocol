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

### Changed

- No transport, host, game, gateway, MCP, model, provider, storage, or harness behavior was added;
  consumer mappings use copied release-like files rather than protocol implementation dependencies.
  Runtime-v1 and Runtime-v2 remain inert contracts; consumer and live gameplay compatibility are
  unverified.

### Deprecated

- Nothing.

### Removed

- Nothing.

### Fixed

- Nothing.

### Security

- No credentials, host files, saves, provider data, or runtime authority were introduced.
