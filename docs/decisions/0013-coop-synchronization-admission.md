# ADR 0013: admit the consumed co-op synchronization boundary

- Status: Accepted for coordinated integration; exact-head CI required before merge
- Date: 2026-09-06

## Decision and scope

The MCP proposal is a read-only synchronization tool. Its six-family `coop-gameplay-v1`
prototype has no consumers for action, vote, effect, or ally-target payloads. Follow the
narrowing option in MCP ADR 0012: replace the unpublished exported prototype with the
separate `coop-synchronization-v1` profile that the actual producer and reader consume.
Do not add unused consumers or actuation tools merely to admit the prototype.

The original complete proposal remains in Git at protocol `75f2e0d00b550a12a92c8fd99a182684a5c7255f`
and MCP `717973552aee877a7913aa80c4efd07fee9d3c62`. This decision does not admit or implement
its local actions, peer voting, shared host effects, or multiplayer gameplay. Frozen POC,
Runtime-v1/v2/v3 artifacts are unchanged. The unpublished proposal's profile and digest are
rejected; no backward compatibility or negotiation with it is claimed.

## Owner and actual serialized consumers

Protocol owns the inert response schema, metadata, semantic relationships, canonical JSON,
goldens, and conformance cases. Gateway's attached runtime serializes this response from
its bounded peer-report ledger. MCP's executable `coop-synchronization-v1` profile parses,
validates, and projects that complete response into `sts2.coop_synchronization` results.
Neither consumes protocol Rust implementation source; each consumes the copied artifact.
The producer is gateway `crates/gateway/src/bin/runtime_support/service_coop.rs`, backed by
`coop_reports.rs`. The reader is MCP `projection_coop_synchronization.rs`, reached through
`mapping_coop_synchronization.rs` and the executable `runtime_support/profiles.rs` selector.
Both consume the identical artifact digest
`d410858cabbd38612345120c2196423130c7b21d788fd2b0d775cd82887087ec`.
The coordinated PRs retain exact tested Git heads; executable evidence is in MCP
`docs/evidence/coop-synchronization-20260906.md`.

Gateway owns roster configuration, caller authentication, report ingestion, lease fencing,
monotonic generation, expiry, and routing. Reports come from the configured coordinator
credential with control scope, not independently authenticated remote peers. The constant
`source: gateway_peer_reports` labels this limitation on every response. Synchronized means
the configured roster's recent reports agree; it never authorizes or proves a host mutation.

## Namespace, lifetime, and wire rules

Instance, gateway session, lease and epoch bind the attached gateway lifetime. MCP session
and correlation retain separate namespaces; MCP translates only its configured session
binding and requires all authority fields to match, without fallback identity injection.
Roster peer IDs are unique within that lease, with one local peer and one to three allies.
Generation is a coordinator-reported monotonic convergence sequence, not a native game's
independently allocated local observation counter. Reports cannot lower a peer's generation.
Releasing the lease prevents further reads/reports. Process restart starts all peers missing.

JSON is UTF-8 with exact field sets, bounded ASCII identities, integer tokens in the safe
53-bit range, and the exact schema digest/provenance. Unknown members, kinds, enum values,
profiles, duplicate keys, and mismatched identities are errors. There is no mutation payload
or nullable extension slot. Peer count equals roster length; missing IDs are unique known
peers. Disconnected requires a nonempty missing set; other statuses require an empty set.
Gateway expires reports before constructing the response; protocol contains no clock.

## Verification and compatibility classification

This is an additive, explicitly selected profile relative to main, and a breaking replacement
of an unadmitted proposal. Required deterministic evidence covers all statuses, every closed
object, metadata/digest/identity mismatches, duplicate and unknown peers, integer bounds,
scope/lease refusal, stale reports, regression prevention, and no downstream game access.
An executable MCP-to-gateway exchange must reproduce convergence, disagreement, disconnect,
and recovery. These are coordination-component results, not live multiplayer evidence.
All source is original MIT work; no host assets, credentials, profiles, or generated binaries
belong in the artifact.

Confirmed component verification on 2026-09-06: protocol's 56 tests, gateway's 125 tests,
MCP's 114 default tests, and the separately invoked actual gateway/MCP executable test all
passed. The last check observed every reported-state transition with separate read/control
credentials, stale-lease and report-scope refusal, and zero downstream game connections.
All 27 conformance vectors and each eight-entry artifact inventory passed. Existing POC and
Runtime-v1/v2/v3 artifact bytes remained identical to current main. This satisfies the
read-only synchronization admission gate; it does not admit the preserved actuation proposal.
