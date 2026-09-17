# ADR 0042: negotiated capabilities v2 live bootstrap extension

- Status: proposed additive candidate
- Owner: `sts2-protocol`
- Consumers: `sts2-gateway`, `sts2-mcp-server`

## Decision

Publish `sts2-protocol/negotiated-capabilities-v2` as an additive schema over the
Gateway negotiated-capabilities response. It preserves every v1 field, identity,
limit unit, and known operation, and adds only
`game_information.live_observation_bootstrap` with revision
`game-information-live-observation-bootstrap-v1`.

A Gateway serves v1 by default. A consumer that explicitly requests the v2
capabilities version may receive v2; an unsupported request is rejected or
falls back to v1 without advertising the v2 operation. MCP accepts the pinned
v2 schema only when that explicit negotiation succeeds. V1 remains closed and
never silently accepts the new operation.

The live bootstrap route and its authentication remain Gateway-owned. This
artifact records only the negotiated offer and its bounded wire/content units.
