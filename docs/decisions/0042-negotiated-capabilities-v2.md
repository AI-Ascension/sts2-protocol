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

A Gateway serves v1 by default. A consumer requests v2 with the exact
`x-sts2-capabilities-version: sts2-gateway-negotiated-capabilities-v2` header
on the capabilities GET. A legacy Gateway that ignores this header returns its
v1 document, which MCP accepts as a strict legacy fallback without the live
operation. A Gateway that understands version negotiation returns HTTP 406 with
`negotiated_capabilities_version_unsupported` for any requested version other
than v1 or v2. MCP accepts the pinned v2 schema only when the v2 document is
returned. V1 remains closed and never silently accepts the new operation.

The live bootstrap route and its authentication remain Gateway-owned. This
artifact records only the negotiated offer and its bounded wire/content units.
