# Coding Standards

## Target rules

This target uses Rust `1.97.1` and edition 2024 for governance tooling and future contract code.
`Cargo.lock` is checked in. The workspace owns dependency versions and lints. The repository has no
managed product project; `Directory.Build.props` is retained as a harmless foundation-parity file.

Write original, focused modules. Do not copy or transliterate another harness, use another
implementation's symbols as requirements, or add a broad `common`, `utils`, or `helpers` module.
Name modules for the bounded contract concept they own. A schema is not an implementation and a
generated binding is not a second normative source.

## Formatting and budgets

Use `rustfmt` with a 100-column maximum and LF text endings. Clippy runs with warnings denied. The
policy checker measures nonblank physical lines:

| Artifact | Preferred | Hard maximum |
| --- | ---: | ---: |
| Production Rust | 300 | 400 |
| Rust tests | 400 | 600 |
| Workflow | 160 | 200 |
| Markdown | 500 | 700 |

Refactor by responsibility before reaching a preferred limit. Functions should be at or below 40
lines; split beyond 60 and document any exceptional need beyond 80. Large generated or snapshot
files require an exact-path exemption with a provenance and regeneration reason.

## API and contract rules

- Prefer typed identifiers, namespaces, versions, profiles, outcomes, and enums over bare strings.
- State units, bounds, ordering, lifetime, and clock semantics explicitly.
- Use explicit serialized names and deliberate missing/null/empty/default behavior.
- Define unknown fields and unknown enum values conservatively; never silently reinterpret them.
- Preserve error origin, retryability, acceptance, settlement, cancellation, and uncertainty.
- Keep canonical serialization deterministic and independent of transport implementation.
- Validate untrusted input at the boundary and return structured errors without panic text or paths.

## Safety and dependencies

Unsafe code is forbidden in this target. No module may access a network, host assembly, process,
filesystem, provider, credential, or game state. Avoid global mutable state and undeclared ambient
configuration. Dependencies must have a documented purpose, locked version, MIT-compatible notice,
and no change to the neutral boundary without review.

All new Rust files begin with `SPDX-License-Identifier: MIT`. Python source and package metadata are
prohibited. Test doubles and fixtures must be deterministic, minimal, sanitized, and clearly marked
as test or release artifacts.

## Aggregate naming authority

Use the aggregate NAMING_CONVENTIONS.md and its naming-registry.yaml for shared
casing, identity namespaces, lifecycle vocabulary, evidence states, and protected external names.
Protocol-owned wire names remain under this target's compatibility review; do not normalize a
standard or existing contract member without a version/profile decision.
