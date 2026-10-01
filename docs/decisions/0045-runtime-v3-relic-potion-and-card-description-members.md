# ADR 0045: let a runtime-v3 observation carry relics, potions, and card descriptions

- Status: accepted
- Date: 2026-10-01
- Owner: `sts2-protocol`
- Refs: AI-Ascension/sts2-protocol#76, AI-Ascension/sts2-game-mod#171, AI-Ascension/sts2-harness#316

## Context

`runtime-v3-gameplay` could describe the *offered* set in detail (PR #74, issue #72) but had no way
to describe what the player already holds. `#/$defs/player` carried exactly eight members — `hp`,
`max_hp`, `energy`, `gold`, `hand`, `deck`, `discard`, `exhaust` — and `#/$defs/card` carried no
`description`. `use_potion` and `discard_potion` were absent from the admitted action kinds.

The consequence was not a gap in convenience. Every affected object declares
`additionalProperties: false`, so a host that knew its relics and potions could not say so: the
**whole observation was refused**, not trimmed. A relic changes the rules for the run and a potion
usable this turn is exactly the fact a decision turns on, so the model was reasoning about a
different game from the one being played.

The consumer half already existed. `sts2-harness#316` (PR #314, merged 2026-09-19) admitted
`relics`, `potions`, `potion_slots`, `max_potion_slots`, `Relic`, `Potion`, `Card.description` and
`Action.potion_id` in `exo/sandbox/schema.rs`. The harness therefore accepted an observation the
wire contract refused. This is the same producer/consumer asymmetry that #799 records on the
Studio publication path, on the protocol side.

## Decision

Extend `#/$defs/player`, `#/$defs/card`, and `#/$defs/action_payload` additively.

- `#/$defs/player` gains `relics`, `potions`, `potion_slots` and `max_potion_slots`, each
  independently omittable.
- New `#/$defs/relic` = `{relic_id, name, description?}`; new `#/$defs/potion` =
  `{potion_id, name, slot?, usable?, target_mode?, description?}`.
- `#/$defs/card` gains an optional `description`.
- `use_potion` (`kind`, `potion_id`, `target_id`) and `discard_potion` (`kind`, `potion_id`) are
  admitted action kinds. `target_id` is required-but-nullable on the wire, matching `play_card`, so
  a potion with no target says so with `null` rather than omitting the member.
- `target_mode` reuses the runtime-v4 enum — `self`, `any_enemy`, `all_enemies`, `none`,
  `unknown` — so the two protocols describe the same things the same way.

### The rule that makes the widening safe

**Absence is never a plausible value.** Every new member is optional and none is defaulted.

- Omitted `relics` means the host disclosed none. `[]` means the host disclosed that it has none.
  Collapsing them would let a reader mistake silence for a fact, so the parser keeps them
  distinct as `Option<Vec<_>>` and a conformance case pins both.
- An undisclosed `Card.description` stays `None`; it never becomes `""`.
- `target_mode: "unknown"` is an *explicit* statement of ignorance, which is a different claim from
  omitting the potion. A host that cannot answer may leave the potion out instead.

This is why the change is additive and safe to ship against producers that know none of it: a host
sending none of the new members is byte-valid, and its serialized form does not grow invented empty
collections.

### Why `use_potion` carries `target_id` and `discard_potion` does not

`play_card` already required a nullable `target_id`; `use_potion` is the same decision about a
different object and follows the same rule rather than inventing a second one. `discard_potion`
names the potion being thrown away and has no target, so requiring a nullable member there would
be a field no producer could fill with a value.

## Consequences

- `RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST` moves from
  `843e2e546116c8011f378d271406ac2fb4ec0e4c2dedd32dee46cc1500315ad5` to
  `03816c3ef8bdba036dd65dd1063bfaf1128cb331f4d0e9267b378c54bf5a1959`. Every producer and consumer
  pin moves with it: the `sts2-game-mod` mirror, `sts2-harness` `runtime_v3_parse.rs` and
  `runtime_v3_wire.rs`, the `negotiated-capabilities-v2` artifact (whose own digest also moves),
  the runtime-v3 goldens, `SHA256SUMS`, and `docs/COMPATIBILITY.md`. A reader pinned to the old
  digest fails closed on the metadata check, which is the intended direction.
- A new golden, `golden/state-response-described-belt.json`, exercises relics, potions, all five
  target modes, a card description, and both potion actions, and round-trips byte-for-byte.
- Populating these members from the host is `sts2-game-mod#171`'s; consuming them is
  `sts2-harness#318`/`#319`'s. This repository owns only the contract shape and its validation.
- Generation weights, reroll behaviour, and undisclosed outcomes stay excluded, per
  AI-Ascension/sts2-game-core#20. Nothing here makes an unrevealed result representable.
