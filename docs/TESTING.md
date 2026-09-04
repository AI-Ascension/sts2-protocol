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
identity. Run the normal foundation commands from this root; the runtime-specific evidence is
`confirmed` for the inert contract and `unverified` for every live consumer until its precondition
is reproduced. The safe action is `show_runtime_probe`, which witnesses a visible status overlay and
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

Use deterministic in-memory inputs, bounded sizes, synthetic identifiers, injected clocks where time
metadata is relevant, and no network or provider calls. Never retain credentials, saves, proprietary
host files, personal paths, or unsanitized arbitrary text. A successful schema parse, Cargo build,
or handshake-looking fixture is not semantic or runtime compatibility evidence.

Record command, exit status, toolchain, target revision, fixture/profile identity, and evidence level
in the handoff or release record. Do not convert unavailable tools or live boundaries into passes.
