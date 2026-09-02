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

Warnings become failures under `--strict`. The checker does not claim schema semantics, live consumer
compatibility, host behavior, or release readiness.

## Configuration changes

Change `policy.toml` and the checker in one focused review. Every new exemption must name one
repository-relative path and a durable provenance/regeneration reason. Do not weaken a rule to make an
unrelated check pass. Validate policy changes with the full local command set in
[`TESTING.md`](TESTING.md).
