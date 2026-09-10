# `coop-native-v1` component contract

This directory records the accepted source and serialization component contract for native
co-op actuation. It contains the exact schema, producer projections, seventeen strict golden
envelopes, and boundary conformance metadata. The contract covers observation, a host-backed
legal catalog, local actions, shared votes, peer rejoin, effects, receipts, and same-operation
recovery.

The managed producer at source commit
`d23ca838a7be875f32242123955b4a27782bac04` (tree
`23336ca834b5870d15ee6369c101d5c67ff34caf`) declares the exact schema digest
`2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629`. The producer capture is
source-only: `CapturePort` is synthetic and does not load STS2, connect native peers, or prove a
live host outcome. Fresh run `coop-native-source-only-20260910-r7` used the .NET 9.0.317
source-only producer probe and checked in all nine serialized wrapper captures under `capture/`.
The recorded wrapper hashes were recomputed from those files; each wrapper member is checked
against its compact golden projection, including explicit nullable envelope fields and receipt
evidence. The producer is deterministic, so this fresh run produces the same golden bytes as the
previous source capture.

The component contract registers `sts2-gateway`, `sts2-mcp-server`, and `sts2-harness` as
boundary consumers. Each consumer must copy the exact artifact, preserve the closed JSON shape,
and keep host authority, gateway routing, MCP framing, and harness coordination in its owning
repository. Unknown members, duplicate keys, unsupported kinds, stale generations, and
unfenced identities remain errors; an `unknown` outcome is retained for same-operation recovery
and never retried as a new mutation.

The serialized conformance record binds the reviewed source producer (`d23ca83`), gateway
(`c8be3a72`), MCP (`037d10de`), and harness (`d876738`) heads and their exact trees. The harness
refresh is limited to the merged `jsonschema` 0.52.1 to 0.55.0 development-dependency update,
Cargo.lock refresh, and build-manifest source digest update; its native co-op wire and
serialization semantics are unchanged, so the fresh producer capture and projections are reused.
This records
component serialization compatibility for those snapshots; it does not imply that later source
heads or live deployments have been validated.

Component acceptance does not establish live native support. The separate runtime gate still
requires a disposable two-peer native STS2 session with distinct peer identities, a settled
model action and supported shared votes, matching native checksums, disconnect/rejoin recovery,
and peer convergence. Until that evidence exists, `live_status` remains `unverified` and this
artifact must not be described as a live multiplayer release.
