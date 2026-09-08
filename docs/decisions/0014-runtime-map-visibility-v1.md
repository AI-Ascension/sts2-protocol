# ADR 0014: Additive runtime map visibility v1

- Status: accepted for protocol conformance
- Date: 2026-09-06
- Owners: `sts2-protocol`
- Consumers: `sts2-game-mod`, `sts2-gateway`, `sts2-mcp-server`, `sts2-harness`,
  `ascension-map-visualizer`

## Decision

The protocol repository owns `runtime-map-v1`, an additive transport-neutral contract for a
host-owned read-only map projection. It has its own schema, artifact manifest, SHA-256 inventory,
conformance case, and fixed snapshot request/response envelope. Runtime-v1, runtime-v2,
`runtime-v3-gameplay`, and `coop-synchronization-v1` remain unchanged.

Every snapshot carries the independent schema identity `visible-map-v1` and a
`projection_version` owned by the producer. The snapshot records source state and generation,
game/mod provenance, nullable map identity only when availability is not `available`, explicit
availability/completeness/freshness labels, bounded reason text, stable node identity, logical
coordinates, visible room category (including `unknown`), visited state, directed edges,
pre-start/current/unavailable position, visible history, terminal IDs, and generation-bound
bindings. A binding keeps the stable graph node ID, exact host action ID, and serialized
`select_map_node` action-option ID as separate fields. The action-option ID is an opaque,
independently bounded value consumed by the host action; it is not required to equal or resolve
as a graph node ID. Each namespace remains unique within its own binding set.

The profile is bounded at 256 nodes, 1024 edges, 256 bindings, 256 visible history/terminal IDs,
and 256 KiB per complete message. IDs are limited to 128 UTF-8 bytes, host action IDs to 512
bytes, and JSON-safe generation/lease values to `2^53 - 1`. Coordinates have an explicit signed
16-bit logical range. A typed decoder checks the byte bound, UTF-8, duplicate JSON keys, nesting,
closed objects, and semantic graph invariants before returning an owned value. Cycles, duplicate
IDs/endpoints/bindings, dangling references, malformed bindings, stale envelope snapshot
generations, and self-bindings are rejected. Coordinates may overlap and a complete projection
may contain disconnected visible components; the profile preserves those host facts without
inferring game topology. A complete empty projection is valid when the host truthfully reports
`pre_start` or `unavailable` position.

Canonical map bytes are compact UTF-8 JSON with sorted object keys and normalized set-like
collections. The full snapshot digest and content, topology, and navigation digests are separate
helpers with documented input sections. The protocol does not expose hidden future nodes,
randomness, host objects, provider output, harness IDs, or mutation authority.

## Compatibility and evidence boundary

This is an additive profile with a new protocol version and schema digest. Existing profiles and
consumer artifact copies remain byte-for-byte unchanged. The checked-in schema and synthetic
vectors prove contract consistency only. Consumer parsing, gateway/MCP route registration, host
map extraction, visualizer bundle validation, licensed game compatibility, and live navigation
settlement require independent consumer and runtime evidence and are not implied by this ADR.

## Consequences

### 2026-09-08 conformance correction

Map text excludes Unicode control characters, including DEL and C1, in both the schema and
typed decoder. Build/version text permits at most 128 UTF-8 bytes and reason text at most 256.
JSON Schema `maxLength` counts Unicode characters; consumers must enforce the additional byte
bounds and semantic graph invariants before accepting a message. Schema validity alone is not
complete protocol validity. Boundary tests retain valid multibyte text and reject one-byte-over
values and C0/DEL/C1 controls through the appropriate validation boundary.

This correction changes the unpublished map schema digest. Consumer artifact copies, pins, and
envelope fixtures must migrate together; previous digests remain rejected. Existing gameplay
profiles are unchanged.

Consumers can adopt map visibility behind their own profile and allowlist gates without changing
the existing gameplay observation. They must preserve the request envelope's instance/session/
lease/epoch/correlation metadata and treat unavailable, incomplete, unknown, historical, and
unsupported states as explicit values. Digest equality is an identity check for the stated
canonical projection; it is not authorization or proof that a host action settled.
