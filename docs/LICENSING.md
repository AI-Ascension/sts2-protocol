# Licensing Policy

## Decision

Original repository code and documentation are licensed under MIT. Contributors license accepted
contributions under the same terms. This is an engineering policy, not legal advice.

## Boundary and provenance

MIT covers only material that this project can license. It does not grant rights to STS2 binaries,
game data, art, music, names, trademarks, platform components, host assemblies, or external
dependencies. This repository must not retain proprietary host bytes, saves, credentials, private
data, bulk decompilation, or copied implementation source.

Future schemas and fixtures must identify their source, owner, generator or hand-authored status,
license, revision/profile, and digest. An imported artifact with unknown or incompatible terms is
blocked. A generated consumer binding is derived output and must not become a second semantic source.

## Dependencies and releases

The governance tool's Cargo dependencies are locked in `Cargo.lock`. Before a protocol package is
released, notices must be generated and reviewed from that exact lockfile and the exact allowlisted
artifact contents. New dependencies require a focused review of maintenance, security, license,
reproducibility, and language-neutrality impact.

Every release allowlist excludes host files, game assets, saves, credentials, absolute machine paths,
target output, and unrelated implementation source. Publication stops when provenance or licensing
is ambiguous.
