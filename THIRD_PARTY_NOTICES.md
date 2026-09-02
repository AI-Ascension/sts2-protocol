# Third-Party Notices

No third-party source, game binaries, game assets, host assemblies, or external implementation
artifacts are vendored in this foundation target.

The repository policy tool has a locked Cargo dependency on `toml`, which remains under its own
license. Before a distributed artifact exists, release tooling must generate and review dependency
notices from the exact `Cargo.lock` used for that artifact. Future imported or generated schemas and
fixtures require an explicit source, license, generator, and digest record.

Local host installations, proprietary files, personal saves, credentials, and build output are not
project source and must not enter release archives.
