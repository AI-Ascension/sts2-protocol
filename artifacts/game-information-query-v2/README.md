# Game-information v2 rest-read candidate

This bundle is a **candidate**, not an admitted profile or native compatibility
claim. It preserves v1 bytes and carries original synthetic evidence only.
The mod is the prospective producer; gateway and MCP are prospective consumers.
The manifest intentionally lists no adopted consumers or invented consumer pins.

The standalone schema is normative for shape. The production Rust
`game_information_v2::Validator` rejects duplicate JSON members, validates the
embedded schema and checks supplied capture/cursor evidence. It never obtains
host facts, authenticates evidence, owns a registry or performs transport calls.
Available fields must match the supplied captured values and action catalog.
Selector counts bind the unique selected-choice evidence, and live occurrences
remain contained by the room snapshot.

Canonical JSON sorts object keys and preserves arrays. Empty items encode as
`[]`, including unavailable pages. The accounting witness independently checks
these bytes; it is not a consumer implementation or a host test.

Regenerate only the separate candidate with:

```sh
node tools/game-information-v2/schema.mjs
node tools/game-information-v2/fixtures.mjs
cargo test --locked --offline -p sts2-protocol --test game_information_query_v2_conformance
node --test tools/game-information-v2/witness.test.mjs
(cd artifacts/game-information-query-v2 && sha256sum -c SHA256SUMS)
```

Run repository foundation policy, formatting, Clippy and workspace tests before
claiming candidate conformance. Two real consumer validators, profile negotiation,
production read-port spies, MCP structured output, harness replay and exact-host
thread/locale/generation behavior remain separate pending gates.

See [ADR 0039](../../docs/decisions/0039-game-information-rest-read-candidate.md).

The PR #50 numeric review repair accepts exact mathematical integer spellings such as
`1.0` and `1e0` at the raw boundary. A lexical normalization pass preserves strings
and rejects fractions and values outside the safe integer range without float rounding;
the unchanged schema then applies each field's narrower bounds. The historical
`fractional-integer-token` vector is corrected from rejection to acceptance, so the
original case inventory is no longer claimed byte-identical. The Rust case runner now
checks the actual raw variant after replacements using an independent Node 24
`JSON.parse` source-reviver and BigInt reference before applying the unchanged schema.
A rounded serde_json floating-point parse is not used as a normative numeric oracle.
All evidence remains synthetic candidate conformance, not live consumer qualification.
