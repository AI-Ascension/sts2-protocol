# Repository Layout

## Foundation tree

```text
.
├── .github/                 # read-only policy and CI automation
├── crates/protocol/         # typed POC contract mapping
├── schemas/poc-v1.schema.json # normative POC JSON Schema
├── artifacts/poc-v1/        # release-like schema, manifest, fixtures, checksums
├── conformance/             # hand-authored cases and legacy foundation fixtures
├── docs/decisions/          # accepted ownership and target decisions
├── crates/protocol/tests/   # target-owned deterministic conformance tests
└── tools/repo-policy/       # non-empty Rust governance package
```

The existing scaffold directories are preserved. Wave 2 adds one non-empty package and its focused
fixtures/tests; it does not add an empty crate, generated output, or product behavior. Every future
source directory must have one responsibility, a named consumer, a build/test purpose, and
documentation of its boundary.

## Ownership and dependencies

The initialized protocol package contains only accepted neutral contracts and must not depend on
game-core, game-mod, gateway, MCP, harness, provider, host, process, or transport implementations.
Consumer repositories may consume a versioned artifact; runtime message flow remains a separate
graph. The game core stays free of transports and host processes. The game-mod owns host authority,
the gateway owns lifecycle/routing, MCP is a thin adapter, and the harness owns coordination and
artifacts.

## Naming and artifacts

Use repository-relative POSIX paths, explicit profile/version directories, and stable names. The
current schema and golden fixtures are hand-authored and their manifest records source, license,
generator, and digest. Future generated schemas, bindings, manifests, and fixtures must record their
inputs and digest. Generated output is never silently edited or treated as normative source.

Release staging must exclude `.git`, `target`, editor state, credentials, private paths, saves,
proprietary host files, and unrelated source. Temporary validation output belongs outside the retained
source tree or in ignored directories.

## Naming authority

Shared naming and identity rules are normative in the aggregate NAMING_CONVENTIONS.md and
naming-registry.yaml. This target owns
protocol-local names and compatibility decisions, while standard and consumed wire spellings stay
exact.
