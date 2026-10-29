# Third-Party Notices

No third-party source, game binaries, game assets, host assemblies, or external implementation
artifacts are vendored in this foundation target.

The repository policy tool has a locked Cargo dependency on `toml`, which remains under its own
license. The protocol package uses the locked `jsonschema` version 0.60.1,
with default features disabled, for embedded Draft 2020-12 schema validation in
the game-information v2 candidate and for conformance tests. Its dependency
classification originally changed from development to production; that historical
admission is recorded in ADR 0039. The current manifest and lockfile pin 0.60.1.
No caller schema or remote schema resolver is enabled. Before a
distributed artifact exists, release tooling must generate and review dependency notices from the
exact `Cargo.lock` used for that artifact. Future imported or generated schemas and fixtures require
an explicit source, license, generator, and digest record.

Local host installations, proprietary files, personal saves, credentials, and build output are not
project source and must not enter release archives.
