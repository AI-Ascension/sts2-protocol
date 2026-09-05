# Development and Automation Workflows

## Lifecycle

```text
decision or issue -> focused change -> local policy/tests -> pull request -> review
  -> authorized merge -> candidate package -> authorized publication -> fresh-byte verification
```

Foundation, implementation, consumer compatibility, runtime verification, and release publication
are separate states. Green CI does not prove a live game, gateway, MCP, harness, provider, or release.

## Current automation

- `policy.yml` tests the Rust repository-policy package and enforces `policy.toml`.
- `ci.yml` runs metadata, verifies each release-like artifact bundle's `SHA256SUMS` inventory
  against actual bytes, then runs formatting, Clippy, and workspace tests.

Both workflows use `pull_request` and pushes to `main`, top-level `contents: read`, explicit timeouts,
pull-request concurrency cancellation, checkout with credentials disabled, and full immutable action
commit pins. Neither workflow has write tokens, secrets, host access, provider access, publication
authority, path filters, or success suppression.

## Authoring rules

Keep workflows focused below 200 nonblank lines and prefer below 160. Pin every third-party action to
a full commit SHA with a release comment. Do not use `pull_request_target`, `continue-on-error`,
`|| true`, blanket retries, arbitrary refs, or unbounded artifacts. Add a workflow only when its
command and evidence surface are real.

Future schema/conformance, security, compatibility, and release workflows must remain read-only for
untrusted pull requests. Release publication requires a protected environment and explicit maintainer
approval; it is not present in this foundation wave.

## Review and local entrypoint

Workflow changes require a review of events, permissions, checkout ref, caches, artifacts, and any
authority expansion. Run the exact commands in [`TESTING.md`](TESTING.md), especially:

```bash
cargo run --locked --offline --package repo-policy -- --strict
```

If a workflow-specific tool is unavailable, report it as unverified rather than hiding the result.
