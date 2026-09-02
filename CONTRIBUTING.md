# Contributing

Thank you for helping build the STS2 protocol foundation. This target values narrow ownership,
language-neutral artifacts, explicit provenance, deterministic validation, and honest compatibility
evidence.

## Start here

Read [`AGENTS.md`](AGENTS.md), [`docs/PRODUCT.md`](docs/PRODUCT.md),
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), [`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md),
[`docs/TESTING.md`](docs/TESTING.md), [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md),
[`docs/LICENSING.md`](docs/LICENSING.md), [`docs/WORKFLOWS.md`](docs/WORKFLOWS.md),
[`RELEASING.md`](RELEASING.md), and [`SECURITY.md`](SECURITY.md).

The initial package is now initialized. Do not add product behavior, empty product crates, copied
implementation source, or a broad common-code module. A future shared item needs an accepted owner,
at least two named consumers, explicit version/profile and serialization rules, provenance, and a
deterministic conformance case before it enters the protocol package.

## Discuss first

Open a design discussion before changing a public contract, schema, identifier namespace, error or
envelope mapping, compatibility/version rule, generated artifact, package boundary, dependency
direction, or release process. Boundary-specific behavior remains with the game core, game-mod,
gateway, MCP server, or harness owner.

## Development workflow

1. Inspect the current target and preserve unrelated work.
2. Identify the owner, named consumers, compatibility classification, and evidence needed.
3. Make the smallest cohesive change with original source and reviewed provenance.
4. Run the target-local policy, formatting, lint, and test commands.
5. Update affected documentation, decision records, and `CHANGELOG.md`.
6. Describe unverified runtime, consumer, packaging, and release claims explicitly.

All product and repository-tool source is Rust. The target has no managed product boundary. New Rust
source uses the MIT SPDX header and follows the file/function budgets in
[`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md).

## Contribution license

By submitting a contribution, you represent that you have the right to provide it and license it
under the repository's [MIT License](LICENSE). Identify generated or adapted material and retain
applicable notices. Do not submit game files, proprietary host data, credentials, saves, or fixtures
whose redistribution rights are unknown.

## Pull requests

Explain the outcome, affected contract IDs or decision records, named consumers, compatibility and
security impact, exact commands/results, provenance, documentation changes, and remaining risks.
A passing local command is not proof of runtime interoperability or release readiness.
