# ADR 0037: public checkpoint-reference contract

- Status: accepted for protocol contract initialization; consumer adoption pending
- Date: 2026-09-12
- Owner: `sts2-protocol`

## Context

ADR 0036 defines exact-state identity and its checkpoint manifest, which are privileged: a raw
full-state digest can act as a dictionary oracle for low-entropy hidden choices. Ordinary agents,
public transcripts, logs, dashboards, and the map visualizer still need to name a checkpoint, show
its assurance, and correlate it with a branch or occurrence without receiving any digest or hidden
payload. Without a closed shared shape, a consumer could pass a privileged digest through a
public-looking surface.

## Decision

Initialize the inert `exact-checkpoint-reference-v1` envelope. It carries only:

| Field | Meaning |
| --- | --- |
| `schema` | `ascension.exact_checkpoint_reference.v1` |
| `reference_version` | `exact-checkpoint-reference-v1` |
| `handle` | Keyed opaque handle matching `^ckpt-h1:[0-9a-f]{64}$` |
| `occurrence` | Bounded occurrence identifier |
| `boundary_kind`, `boundary_phase` | Permitted capture-location labels |
| `assurance` | One of `public_observation_only`, `capture_only`, `restore_supported`, `restore_verified`, `continuation_certified` |
| `restore_verified` | Whether a destination actually recaptured the state |

The object is closed (`additionalProperties: false`). There is deliberately no member for an
exact-state, checkpoint, blob, or compatibility digest, so a privileged member is a validation error
rather than silently ignored data. An unsupported `reference_version` is rejected without lossy
coercion. Schema validity proves nothing about coverage, restorability, or honest handle issuance;
the trusted producer, the coverage contract, and a verified restore establish those facts.

## Ownership and adoption boundary

Protocol owns the envelope, its artifact copy, checksums, and deterministic conformance. The named
prospective consumers are `sts2-harness` (produces public summaries from its keyed projection),
`sts2-mcp-server` (exposes references to tools without hidden payloads), and
`ascension-map-visualizer` (displays references and lineage). Adoption is unverified until each
consumer pins the schema digest and round-trips a reference. The handle's key and issuance policy
stay with the trusted producer; the protocol must not invent one.

## Consequences

- Public surfaces gain one closed, versioned reference instead of ad hoc digest strings.
- A digest-bearing reference is rejected by construction, which removes a whole class of accidental
  disclosure.
- The envelope cannot be used to compare states, locate the same state across runs, or claim
  restore evidence by itself.
