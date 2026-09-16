# ADR 0044: initialize the inert exact-restore handoff profile

- Status: accepted for protocol contract initialization; consumer adoption and native restore pending
- Date: 2026-09-16
- Owner: `sts2-protocol`

## Context

`exact-state-v1` defines exact-state identity and the immutable checkpoint manifest, while
`exact-checkpoint-reference-v1` intentionally hides privileged digests from public surfaces.
Neither defines a closed handoff for the Harness-owned manifest and blobs to a selected native
destination. Owner fencing, durable operation identity, bounded transfer, explicit uncertainty,
and a post-restore receipt must be shared without transferring the Harness, Gateway, MCP, or
game-mod's local authority to this protocol repository.

## Decision

Initialize the additive, inert `exact-restore-v1` profile in
[`artifacts/exact-restore-v1/`](../../artifacts/exact-restore-v1/), sourced from
[`schemas/exact-restore-v1.schema.json`](../../schemas/exact-restore-v1.schema.json). The frame
schema defines five request phases, their closed response shapes, and a typed error response.
Frames carry stable message/correlation identities and a durable operation UUID bound to the
selected child and complete current-owner fence. It reuses the existing checkpoint manifest,
canonical payload, and restore-artifact descriptors; it does not alter `exact-state-v1`.

The profile fixes chunk, frame, reference-count, blob, aggregate-closure, unfinished-operation,
and terminal-receipt bounds. It distinguishes `STAGING`, `CLOSURE_VERIFIED`, `COMMIT_INTENT`,
`UNKNOWN`, `RESTORE_VERIFIED`, and `NOT_FOUND`. A repeated begin reports the existing phase without
resetting work. Chunk replay is exact-byte idempotent only at the same offset. Commit intent is
durable before a host effect; uncertain commit is looked up and never blindly retried. A successful
receipt binds the source and recaptured exact-state identity.

The named producer/consumer boundaries are `sts2-harness`, `sts2-game-mod`, `sts2-gateway`, and
`sts2-mcp-server`. Harness owns source-artifact validation and transfer coordination; game-mod owns
native staging, host mutation and recapture; Gateway owns live owner/lease fencing and forwarding;
MCP owns tool framing and mapping. These mappings are descriptive responsibilities, not new
protocol authority. Authentication, route paths, MCP tool names, storage implementation, host
threading and mutation remain outside this repository.

`exact-restore-v1` is a new additive profile. An unsupported version, digest, kind, field or enum
is rejected before an effect. Unknown or unavailable effect outcomes remain distinguishable.
`REJECTED` and `STALE_OWNER` represent a request known not to have started a host effect;
`UNAVAILABLE` carries an explicit `host_effect` value and never certifies settlement.

The provenance is original hand-authored schema and synthetic conformance data mapped only to the
existing `ascension.checkpoint_manifest.v1` fields. No proprietary host bytes, product
implementation, save, credentials, or runtime authority are included.

## Compatibility and evidence

The artifact schema digest and checksum inventory are fixed by the manifest. Deterministic
conformance checks compile the schema, validate every phase golden, reject unknown fields and
state/receipt contradictions, and check canonical frame digests and resource ceilings.

The protocol artifact establishes only a neutral frame contract. Harness artifact transfer,
Gateway forwarding and owner checks, MCP tool mapping, native durable staging, host restore, and
post-restore recapture remain unverified until their exact consumer revisions provide source and
serialized production-boundary evidence. Current native checkpoint capture is not restore support.
