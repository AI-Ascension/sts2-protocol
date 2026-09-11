# ADR 0031: Additive `runtime-v4-expert` fair-play profile

- Status: Proposed for coordinated consumer implementation
- Date: 2026-09-06

## Context

The accepted `runtime-v3-gameplay` profile is deliberately small. It carries HP, energy, gold,
basic card fields, current state, reachable map options, and a compact enemy intent. Strategic
play also needs ordinary context that is visible in the host UI: the selected character and run
location, block, powers/statuses, relics, potions and capacity, richer card metadata, enemy
status and rendered targets, the visible map graph, and the domain of a contextual choice.
Rest-site options and potion use also require parameters that cannot be smuggled into a v3 ID.

## Decision

The protocol target owns a new `runtime-v4-expert` profile with its own schema, digest, artifact,
manifest, and deterministic conformance case. Runtime-v3 remains byte-for-byte unchanged and is
never upgraded in place. The four named consumers are the managed mod producer, gateway forwarder,
harness consumer, and MCP adapter; each must pin this profile's digest before accepting it.

The profile carries bounded owned values only. Every member is required at the profile boundary.
`null` means the value was unavailable or not visible on the current ordinary UI surface; `[]`
means that the surface was observed and empty. Unknown fields and enum values are rejected. The
profile contains no host objects, reflection paths, process commands, saves, credentials, hidden
RNG, future outcomes, unrevealed map content, draw order, or private teammate information.

The player model adds block, powers/statuses, relics, potions, slot capacity, and card type,
rarity, target, and description. The combat model adds enemy block, powers/statuses, and the
currently rendered intent target set. The map model carries only the visible graph and reachable
flags. Choice domains are explicit. Actions add `use_potion`, `rest_option`, `select_character`,
and selection identifiers while retaining the v3 action meanings. Potion targets and rendered
intent targets are identities supplied by the current legal surface; legality remains host-owned.

## Ownership and compatibility

The mod owns native extraction, visibility checks, action dispatch, and postconditions. The
gateway forwards the profile without interpreting game rules. MCP maps the profile into its
versioned catalog. The harness validates the copied artifact and records profile/digest lineage.
The protocol target owns only the language-neutral schema and conformance vectors. No consumer may
fall back to v3 after a v4 digest or field failure; it must report an unsupported profile and use
the separately admitted v3 path when configured.

## Evidence boundary

The schema, artifact copy, golden observation, and offline tests prove inert contract closure and
serialization. They do not prove that the installed target exposes every field, that the visible
map is complete, that gateway/MCP mappings are implemented, or that a live game can complete a
run. Those claims require independent source/build checks and root-reserved native evidence.
