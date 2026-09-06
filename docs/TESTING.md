# Testing and Evidence

## Foundation commands

Run from this target root:

```bash
cargo metadata --locked --no-deps --format-version 1
cargo run --locked --offline --package repo-policy -- --strict
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --workspace --all-targets --all-features
```

The current workspace contains the non-empty `sts2-protocol` package and repository policy tool. These
commands prove the target-local package, governance tool, and deterministic test seam; they do not
prove a live consumer or runtime integration.

## Protocol tests

The `poc-v1` seam has project-owned cases for exact compact JSON round-trips, source/artifact schema
equivalence, named consumers, metadata/digest validation, action/result shape, and a structural
invalid-action fixture reserved for core legality. Future neutral contracts must add cases for:

- exact canonical serialization, optional/null/empty/default behavior, bounds, ordering, and unknowns;
- identifier namespaces, lifetime, collision, version/profile mismatch, and stale references;
- envelope/error origin, retryability, accepted/settled/rejected/cancelled/unknown outcomes;
- deterministic schema or fixture generation, sorted manifests, digests, and package allowlists;
- Rust, managed, and language-neutral consumer parsing where applicable; and
- compatibility migration, licensing, provenance, redaction, and release verification.

Every requirement maps to at least one case and every case maps back to one requirement. The five
consumer mappings use copied release-like artifact files and are tested locally in their own PRs.
A host, gateway, MCP, harness, provider, publication, or game test without its precondition is visibly
skipped or unverified; it is never counted as a pass.

## Runtime profile tests

`runtime_conformance.rs` validates source/artifact byte identity, schema compilation, all five
`runtime-v1` golden messages, unknown-field rejection, manifest consumers, and the fixed action
identity. It also decodes every golden into the typed `RuntimeMessage`, validates it, and re-encodes
it to byte-identical canonical JSON; checks `RUNTIME_MAX_GENERATION` and `RUNTIME_MAX_ACTION_COUNT`
at and above each bound; and rejects metadata, provenance, identity, and kind/status shape drift with
the schema and typed validator agreeing. Run the normal foundation commands from this root; the
runtime-specific evidence is `confirmed` for the inert contract and `unverified` for every live
consumer until its precondition is reproduced. The safe action is `show_runtime_probe`, which witnesses a visible status overlay and
does not claim a game-rule transition.

`runtime_v2_conformance.rs` validates the separate `runtime-v2` schema/artifact byte identity, all
deterministic request and receipt goldens, bounded observation fields, outcome-specific receipt shape,
unknown-field rejection, stable operation identity, duplicate replay, idempotency conflict, and the
unknown-to-settled reconciliation vector. The contract vector is `confirmed` for this inert target;
consumer, host, transport, and live gameplay settlement remain `unverified`.

`runtime_v3_gameplay_conformance.rs` validates the separate fair-play schema/artifact byte identity,
typed state/action goldens, complete-message bounds, duplicate action rejection, unknown-field
rejection, visible-seed representation, and the dispatch settlement witness shape. It is
protocol-only evidence. Host-generated catalog completeness, consumer mapping, Exo decisions,
provider behavior, and live target-build settlement remain `unverified`.

## Test discipline

`runtime_v3_strictness.rs` exercises all twelve message kinds, result statuses, required nullable
members, nested tagged enum closure (including empty variants), and control-character rejection.
It compares schema and Rust results for payload mutations. Cross-field relations and UTF-8 byte
limits still require the typed validator as described in
[ADR 0009](decisions/0009-proposed-contract-conformance-corrections.md).

`wire_closure.rs` checks required nullable members and closed objects against the unchanged
schemas, including nested neutral metadata, Runtime-v1, and Runtime-v2 messages. It also rejects
duplicate members inside POC nullable objects and at the Runtime-v1 envelope and nested objects
before they can be collapsed by an intermediate JSON value.
CI verifies every checked-in POC, Runtime-v1, and Runtime-v2 checksum inventory against actual bytes.

Use deterministic in-memory inputs, bounded sizes, synthetic identifiers, injected clocks where time
metadata is relevant, and no network or provider calls. Never retain credentials, saves, proprietary
host files, personal paths, or unsanitized arbitrary text. A successful schema parse, Cargo build,
or handshake-looking fixture is not semantic or runtime compatibility evidence.

Record command, exit status, toolchain, target revision, fixture/profile identity, and evidence level
in the handoff or release record. Do not convert unavailable tools or live boundaries into passes.

`runtime_v3_continuation` checks the three ADR 0012 action vectors through the schema and
Rust boundary, exact serialization, extra-argument rejection, unknown kinds, and the prior
digest rejection. The Runtime-v3 checksum inventory includes all seven golden envelopes,
the manifest, artifact schema, source schema and conformance case. These synthetic checks
prove contract consistency only; consumer and host verification remain separate.

`coop_synchronization_conformance` runs all 27 shared vectors through schema and Rust
validation, closes every object, rejects missing/duplicate fields and fractional integer
tokens, and checks canonical round trips and manifest consumer identities. CI verifies all
eight checksum entries, including source schema and conformance copies. The separate
executable gateway/MCP evidence is recorded in ADR 0013; contract tests alone do not prove it.

`watchdog_recovery_v1_conformance.rs` validates the new inert sideband's byte-identical source and
artifact schemas, manifest and conformance coverage, all nine request and nine response kinds,
closed-object rejection, bounded invalid fixtures, and RCJ-1 vectors for every frozen action variant.
Its raw malformed vectors cover repeated top-level and nested members, unsorted top-level and nested
keys, Unicode, floating-point values, escapes, and malformed Base64. The test requires exact
canonical JSON bytes and recomputes each vector's SHA-256 payload digest using a test-only `sha2`
dependency. The test does not authenticate callers, issue leases, access a host, persist an
operation, or claim any consumer/runtime compatibility; those remain owner-side gates.
