# `exact-state-v1` identity contract

This directory is the release-like copy of the inert `asc-jcs-state-v1` exact-state identity
contract. It contains the exact-state, checkpoint-manifest, and coverage-contract envelope schemas,
one synthetic end-to-end golden payload with its canonical bytes and computed identifiers, and
deterministic conformance metadata. It carries no game state, host authority, storage, or mutation
behavior.

The canonical profile narrows RFC 8785 to ASCII schema field names, safe-range integers with tagged
`uint64` and `float64_bits` objects, and explicit binary encodings. Floats, exponents, lexical
negative zero, duplicate keys, lone surrogates, BOMs, and ambiguous unknown values are rejected;
absent, null, empty, and explicit unknown stay distinct. Identifiers are domain-separated by
`AI-ASCENSION/EXACT-STATE/v1\0` and `AI-ASCENSION/CHECKPOINT/v1\0`, so an exact-state digest, an
immutable checkpoint identifier, and a raw blob digest are not interchangeable. Existing
Runtime-v3/v4 `state_id`, observation fingerprints, raw legal-action catalog digests, and live
execution fences keep their present meanings.

Every fixture here is synthetic and marked. `golden/state-payload.json` is not a game capture and
its compatibility digests name fixture data. The generic `state` group remains open until the
game-side field and RNG inventory is complete against a real build; the coverage contract and
adapter identity, not this envelope, carry the completeness claim. Schema validity never authorizes
an exact checkpoint or proves restorability.

Prospective consumers are `sts2-game-mod` as the authoritative capture/restore producer,
`sts2-harness` as the durable-artifact and replay coordinator, and `sts2-gateway` as the transport
and lease-fence owner. Adoption is unverified until each consumer pins the schema digest and
provides source plus serialized round-trip evidence. Engine capture/restore and the live
supported-boundary matrix remain open gates. See
[ADR 0036](../../docs/decisions/0036-exact-state-identity-contract.md).
