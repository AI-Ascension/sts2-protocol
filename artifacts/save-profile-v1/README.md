# save-profile-v1 candidate

Original MIT inert artifact owned by sts2-protocol. Prospective consumers are sts2-game-mod,
sts2-gateway, sts2-mcp-server and sts2-harness; none is admitted by this bundle.

See [ADR 0047](../../docs/decisions/0047-save-profile-v1-candidate.md) for exact identity lifetimes,
canonical encoding, digest domain, operation and read-only recovery semantics, and native work.
Source schema and standalone copy are byte-identical. `manifest.json` pins the source digest;
`SHA256SUMS` covers every distributable file plus source schema and requirement inventory.
`golden/` is synthetic, not a capture of any game, save or consumer. No paths or credentials occur.

Regenerate with `node tools/save-profile/generate.mjs` from repository root, then run the full
Rust gates and `node --test tools/save-profile/*.test.mjs`. Changes to this candidate's closed wire
require new digest pins and coordinated consumer review, not silent adoption.
