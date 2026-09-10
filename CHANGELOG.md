# Changelog

All notable user-visible changes to this project will be documented here. The project follows
Semantic Versioning once a protocol artifact or repository release exists.

## Unreleased

### Added

- (2026-09-09) Add the accepted `seeded-run-v1` explicit seeded-launch contract, including its
  selected native context manifest, content-addressed context digest, lifecycle goldens, copied
  artifact, and deterministic conformance case. These checks establish protocol source and
  serialization evidence only; consumer compilation, licensed-host selection, save/profile
  safety, live run settlement, and release compatibility remain unverified. See ADR 0015.
- (2026-09-08) Initialize the candidate `runtime-v4-expert-rest-action-v1` additive profile with
  its hand-authored schema, release-like artifact, provenance, 16 synthetic goldens, 22 independent
  mutation fixtures, two serialized Smith/Mend producer-shaped lifecycles, checksum inventory, exported
  identity constants, and strict conformance case. Selector completion checks retain prior admission
  context when the final observation returns to `rest`. No downstream consumer is admitted; managed
  native producer, gateway, MCP, harness, host, and live settlement evidence remain pending, while
  the existing potion action artifact is unchanged.

- (2026-09-08) Add the `runtime-map-v1` additive, host-owned read-only map projection with its
  schema, release-like artifact, checksum inventory, three golden messages, typed decoder, and
  implementation-neutral conformance case. The merged protocol main source at `b3d3034` has the
  schema digest `ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b`; these checks
  establish source and serialization evidence only. These protocol checks do not attest consumer
  parsing, artifact migration, or route registration; separate current-head component evidence is
  required for those claims. Native map visibility, navigation settlement, gameplay, and release
  readiness remain unverified.

- Add the consumed `coop-synchronization-v1` response artifact, full conformance vectors and
  closed Rust mirror for gateway peer-report serialization and executable MCP projection.
  Retire the unadmitted action/vote/effect prototype exports under ADR 0013; their original
  source remains in Git history. Gateway/MCP executable verification is limited to coordinator
  reports with zero downstream game connections; this adds no mutation authority or multiplayer
  gameplay claim.

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
- Within this protocol-only change, no transport, host, game, gateway, MCP, model, provider,
  storage, or harness behavior was added; consumer mappings use copied release-like files rather
  than protocol implementation dependencies. The protocol package continues to own inert
  Runtime-v1, Runtime-v2, Runtime-v3, and `runtime-map-v1` contracts; named consumers and live
  gameplay compatibility are outside this target and remain unverified here.

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
