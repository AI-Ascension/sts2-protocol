# ADR 0036: initialize the inert exact-state identity contract

- Status: accepted for protocol contract initialization; engine and consumer adoption pending
- Date: 2026-09-12
- Owner: `sts2-protocol`

## Context

Different agents, replays, and policies need a comparable identity for one exact point in a game
run so that equal starting conditions can be recognized and divergences located. Existing
identities do not provide that: the mod `StateId` and Runtime-v3 fingerprint identify public
observation/action changes, the native co-op checkpoint ordinal is a process-local occurrence
coordinate, the combat SHA-256 covers serialized combat bytes, and the co-op shared-state digest is
an explicitly incomplete subset. Hashing a projection does not upgrade its coverage, and a runtime
lease or execution epoch is a fence, not content identity.

## Decision

Initialize the additive, inert `asc-jcs-state-v1` identity contract in this target. It defines
canonical bytes and domain-separated SHA-256 identities only; it carries no game rules, host state,
persistence, or mutation authority.

| Field | Value |
| --- | --- |
| Canonical profile | `asc-jcs-state-v1` |
| Exact-state payload schema | `ascension.exact_state.v1` |
| Checkpoint manifest schema | `ascension.checkpoint_manifest.v1` |
| State digest namespace | `asc-state:v1:sha256:<64 lowercase hex>` |
| Checkpoint ID namespace | `asc-checkpoint:v1:sha256:<64 lowercase hex>` |
| Blob digest namespace | `sha256:<64 lowercase hex>` |
| State domain separator | `AI-ASCENSION/EXACT-STATE/v1` + `0x00` |
| Manifest domain separator | `AI-ASCENSION/CHECKPOINT/v1` + `0x00` |
| Reference bytes | restricted RFC 8785 JSON canonicalization |
| Source | `crates/protocol/src/exact_state.rs`, `crates/protocol/src/exact_state/` |
| Conformance case | `conformance/cases/exact-state-v1.json` |

Exact state and checkpoint identity are separate from existing public identities. The existing
Runtime-v3/v4 `state_id`, observation fingerprints, native checkpoint ordinals, raw legal-action
catalog byte digests, and live execution fences keep their present meanings. A consumer must use
`exact_state_digest`/`exact_checkpoint_id` at exact boundaries and must not reinterpret an existing
identifier as exact state. A matching exact digest supports a same-start claim only together with
complete declared coverage, enforced compatibility, verified restore, and controlled external
inputs; a bare hash proves neither completeness nor restorability.

The restricted profile narrows RFC 8785 to ASCII schema field names, integers inside the IEEE-754
safe range with tagged `uint64`/`float64_bits` objects for wider values, and explicit binary
encodings. Floats, exponents, lexical negative zero, duplicate keys, lone surrogates, BOMs, and
unknown extension behavior are rejected: absent, null, empty, and explicit unknown stay distinct,
and a required unknown value forbids an exactness claim. Object keys sort ordinally, which equals
JCS UTF-16 ordering for ASCII names; arrays and engine-meaningful collection order are preserved.
Bounds are 16 MiB raw input, 16 MiB canonical output, and 64 nested containers.

## Ownership and adoption boundary

Protocol owns the canonical byte function, typed identity wrappers, bounds, domain separators, and
the deterministic conformance case. Compatibility classification is additive and versioned: a
profile or domain change yields a new namespace rather than reinterpreting existing values.
Provenance of the supplied synthetic vectors is recorded in the conformance case; they are contract
fixtures, not recorded game artifacts.

The named prospective consumers are `sts2-game-mod` as the authoritative capture/restore producer,
`sts2-harness` as the durable-artifact and replay coordinator, and `sts2-gateway` as the transport
and lease-fence owner. Until those owners pin the exact digest and provide source plus serialized
round-trip evidence, adoption is unverified and no live compatibility is claimed.

Game-owned per-phase payload schemas are intentionally not defined here. The state object remains
open until the game-side field and RNG inventory is complete against a real build, because an
incomplete closed schema or an untyped bag would silently weaken coverage. The coverage contract and
adapter identity, not this envelope, carry the completeness claim.

## Independent witness

`tools/exact-state/canonical.mjs` is an original, bounded JavaScript implementation of the same
profile with its own strict parser and emitter. `node --test tools/exact-state/canonical.test.mjs`
reproduces every recorded positive vector, checkpoint identifier, blob digest, equivalence and
distinction pair, and raw rejection, so byte agreement is checked by two in-repository
implementations rather than only by the package fixtures. The authoritative C#/native witness does
not exist yet; writing the mapping twice does not by itself certify the game adapter.

## Consequences

- Callers get exact canonical bytes and non-interchangeable identities without new game authority.
- The restricted profile must be implemented identically in any independent validator language;
  byte equality, not hash equality, is the comparison.
- The engine capture/restore work and the live supported-boundary matrix remain open gates.
