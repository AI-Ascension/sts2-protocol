# ADR 0014: watchdog-recovery-v1 neutral sideband

- Status: accepted for a release-like neutral artifact; consumer implementation remains unverified.
- Date: 2026-09-06
- Owners: protocol maintainers for the inert artifact; gateway, host-mod, harness, MCP, and watchdog
  maintain their boundary-specific implementations.
- Compatibility: new additive profile; no existing runtime-v1, runtime-v2, co-op, or frozen
  runtime-v3-gameplay bytes change.

## Decision

Publish the closed, language-neutral `watchdog-recovery-v1` frame and conformance artifact under
`artifacts/watchdog-recovery-v1/`. Its source is `schemas/watchdog-recovery-v1.schema.json`; the
release-like `schema.json` copy must remain byte-identical. The approved schema digest is
`fb934d3157485aaf6e13e6ebbb213ec8a14c7fc6f5eeebc06b7a22c1f0009217`.

Named prospective consumers are `ascension-watchdog`, `sts2-game-mod`, `sts2-gateway`,
`sts2-harness`, and `sts2-mcp-server`. Naming a consumer does not claim that it parses, authenticates,
persists, fences, dispatches, settles, or recovers this profile. Each owner must add its own mapping,
semantic tests, and runtime evidence before asserting compatibility.

The artifact carries no transport type, authentication decision, lease authority, persistence logic,
host object, process control, MCP framing, provider behavior, or game mutation. Those meanings remain
with their owning boundaries. Its fields describe authority and operation relationships only so that
owners can exchange a stable, bounded recovery record.

## Canonical and compatibility rules

The sideband uses bounded `RCJ-1` canonical action bytes: the frozen runtime-v3 `legal_action` shape,
ASCII identity/enum strings, `null` where the frozen schema permits it, sorted ASCII member names,
no whitespace, no escapes, no Unicode, no arrays, and no numeric or floating-point action values.
Artifact digests hash exact approved immutable bytes. Consumers must not replace either rule with a
default Rust or C# serializer. Any future input outside this subset requires a new profile/version and
cross-language golden vectors.

Every frame carries the sideband schema digest and a frozen runtime-v3 schema digest. Mixed digests,
unknown fields, unsupported versions, or incompatible release/config/profile bytes are rejected before
consumer-specific authorization or mutation. Runtime-v3 remains separately validated against its
unchanged owner-published schema.

Operation records distinguish durable intent, possible dispatch, acceptance, settlement, rejection,
unknown outcome, and reconciliation. Historical lookup is read-only. The sideband does not promise
exactly-once effects across a host mutation/receipt-persistence crash window.

## Evidence and follow-up

The 18 valid frames cover all nine request and nine response kinds. Invalid shape fixtures cover
unknown fields, stale contract version, and the action bound. RCJ vectors cover all frozen action
variants and malformed duplicate, Unicode, float, and escape inputs. Rust conformance checks compare
source/artifact bytes, Draft 2020-12 schema validation, fixture coverage, and RCJ vectors.

This ADR establishes a protocol artifact only. Gateway durable boot/lease authority, host fence and
ticket enforcement, harness resume/provider accounting, watchdog supervision, consumer release
coordination, and live/reboot/soak verification are separate gates and remain unverified here.
