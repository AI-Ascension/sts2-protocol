# Repository Layout

## Foundation tree

```text
.
├── .github/                 # read-only policy and CI automation
├── crates/protocol/         # typed POC contract mapping
├── schemas/poc-v1.schema.json # normative POC JSON Schema
├── schemas/runtime-v1.schema.json # normative host-probe JSON Schema
├── schemas/runtime-v2.schema.json # normative gameplay-operation JSON Schema
├── schemas/runtime-v3-gameplay.schema.json # normative fair-play semantic gameplay schema
├── artifacts/poc-v1/        # release-like schema, manifest, fixtures, checksums
├── artifacts/runtime-v1/    # release-like runtime-v1 bundle
├── artifacts/runtime-v2/    # release-like runtime-v2 bundle
├── artifacts/runtime-v3-gameplay/ # release-like fair-play gameplay bundle
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

## Runtime-v3 gameplay modules

The gameplay root retains public reexports and observation/state types. Its private `action`
module owns semantic actions, transition witnesses, and recovery outcomes; `metadata` owns
artifact provenance and request context. `message` retains envelopes, constructors, and validation
entrypoints. These handwritten modules use the ordinary production line budget without exemptions.
The split preserves public paths, wire behavior, schemas, and frozen artifact bytes.
