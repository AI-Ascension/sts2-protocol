# ADR 0041: game-information live-observation bootstrap

- Status: proposed candidate; no native host or consumer compatibility claim
- Owner: `sts2-protocol`
- Prospective producer: `sts2-game-mod`
- Prospective consumers: `sts2-gateway`, `sts2-mcp-server`, `sts2-harness`

## Context

The accepted `game-information-query-v1` profile requires an entity-scoped
`instance_ref`, a nested `snapshot_ref`, and a matching `parent_observation` on every live
request. Its capabilities response advertises live support but does not carry those references.
The delivered lookup-binding profile fences an owner binding and a snapshot generation, but its
observation is not an entity reference. Therefore no accepted response can bootstrap the first
entity-scoped live query.

The missing native values and the missing wire bootstrap are separate boundaries. The game-mod
must still obtain native run, occurrence, snapshot, entity, and generation facts from one coherent
host-thread read. This decision defines only an additive, transport-neutral serialization contract
for carrying those facts after a route owner chooses a transport.

## Decision

Add the candidate profile `game-information-live-observation-bootstrap-v1`. A request contains a
definition selector and bounded limits. The selector may omit an exact instance to enumerate a
bounded set of matching visible occurrences, or may name one exact instance. A successful response
contains:

1. a query-v1-shaped `parent_observation`;
2. a bounded `visible_entities` list, with each exact `instance_ref`, its matching `snapshot_ref`,
   and an optional content-manifest `definition_ref`;
3. an authenticated scope binding the gateway instance, native run, manifest, locale, and positive
   harness authority epoch; and
4. explicit owner provenance separating native occurrence epoch, gateway instance fencing, harness
   authority epoch, and transport lease epoch.

An explicit selector that names one instance must return that instance. A definition-only selector
may match multiple instances; every returned occurrence remains distinct by its native
`entity_id`, and the caller must choose one exact reference, including its attested snapshot
reference, before issuing query-v1. A producer
must never invent a wildcard, process-local counter, generated action ID, or transport lease epoch.
The owner-provenance labels describe responsibility; they are not authentication evidence. Gateway
and the existing lookup-binding fence remain the authority for authenticated instance and harness
scope.

Unavailable native identity, stale generation, foreign instance, duplicate occurrence, or
definition/instance mismatch is an explicit failure. The profile carries no route, host access,
credential, lifecycle, persistence, or mutation authority.

## Compatibility

This is a new negotiated profile and schema digest. It does not modify:

- `game-information-query-v1` (`376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9`);
- `game-information-lookup-binding-v1`
  (`f10f9af01d6be1de104069ba842e7971971e88f27553e782e81174ee7aa1cd58`); or
- runtime-v3/v4 observation and transport profiles.

The candidate artifact is serialization evidence only. Gateway, MCP, harness, and native game-mod
owners must separately review and pin it before claiming route, host, or live-agent compatibility.
