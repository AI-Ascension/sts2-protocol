# Policy as Code

## Enforcement entrypoint

Run from the target root:

```bash
cargo run --locked --package repo-policy -- --strict
```

The implementation is target-local under `tools/repo-policy`. It is a governance tool, not a
protocol or product crate. CI runs the same command after testing the tool.

## Enforced rules

`policy.toml` defines the exact required foundation/package/fixture files, ignored generated/editor
directories, Rust/C#/workflow/Markdown budgets, and path-specific exemptions. The checker enforces:

- required files and valid exact-path exemption entries;
- rejection of Python source and Python package metadata;
- nonblank line budgets by artifact category;
- explicit workflow permissions and rejection of `pull_request_target`, unconditional success, and
  mutable action references;
- MIT root license, Rust source SPDX headers, and Cargo license inheritance;
- local Markdown link targets; and
- a Cargo workspace, required lockfile, workspace package metadata, inherited lint policy, and
  toolchain/MSRV agreement.

Policy version 2 gives every finding a mandatory default and marks preferred `SIZE001` limits
advisory, so those warnings remain visible without failing `--strict`. Version 1 remains supported
for legacy callers and promotes every warning in strict mode. The checker does not claim schema
semantics, live consumer compatibility, host behavior, or release readiness.

## Configuration changes

Change `policy.toml` and the checker in one focused review. Every new exemption must name one
repository-relative path and a durable provenance/regeneration reason. Do not weaken a rule to make an
unrelated check pass. Validate policy changes with the full local command set in
[`TESTING.md`](TESTING.md).

## Production lint scope

The production Clippy lane selects workspace libraries and binaries and forbids
unwrap, expect, panic, todo and unimplemented on the compiler command line.
A source-level allowance cannot override that lane. The existing all-target lane
still checks tests with their scoped allowances. `production_lints` runs real
compiler fixtures for forbidden constructs, an attempted blanket allowance,
and valid comments/test-only code. Missing Clippy or an unrelated compiler
failure cannot satisfy a negative case: its diagnostic must identify the rule.
