# `runtime-v3-gameplay` release-like artifact

This profile adds exactly one gameplay mutation, `play_card`, without changing the frozen
`runtime-v2` contract. Card indexes and target identities are bounded, and a settled result
requires a fresh collection/energy observation plus a `play_card_settled` witness.

The normative source is [`schemas/runtime-v3-gameplay.schema.json`](../../schemas/runtime-v3-gameplay.schema.json).
The artifact is a checked-in conformance bundle; it is not evidence that a particular host build
supports the action or that a live profile has been exercised.
