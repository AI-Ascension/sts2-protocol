# ADR 0014: Native co-op actuation candidate

## Status

Candidate, unadmitted. Recorded 2026-09-09.

## Context

The managed `CoopNativeRuntime` source seam produced deterministic request and response bytes for
observation, local actions, shared votes, rejoin, and same-operation recovery against a synthetic
capture port. The capture is producer evidence only: it does not load STS2, connect native peers,
or prove a live host outcome. Before this record there was no checked-in native co-op schema or
artifact, and the producer's declared digest had no corresponding schema file.

## Decision

Record an implementation-neutral `coop-native-v1` candidate with a closed envelope and bounded
observation, action, vote, effect, and recovery shapes derived from those captured bytes. Keep the
candidate under the protocol repository for strict schema and fixture review, with zero consumers
and an explicit `unadmitted` status. Preserve `accepted`, `settled`, `rejected`, and `unknown`
outcomes and require an operation identity for reconciliation. Keep host authority in the mod,
lease and routing decisions in the gateway, framing and catalog mapping in MCP, and coordination in
the harness.

The canonical owner is the `sts2-protocol` repository artifact namespace
`sts2.protocol/coop-native-v1`; the version/profile is `coop-native-v1`. Named prospective
consumers are `sts2-gateway`, `sts2-mcp-server`, and `sts2-harness`, but none is admitted in the
manifest until it independently validates the exact bytes. Envelope identities and operation
receipts live for one authenticated session/lease lineage; an operation ID is retained across
unknown outcomes for same-operation reconciliation, while peer tokens are opaque for that session.
Serialization is strict UTF-8 JSON with explicit nullable envelope members, integer generations,
closed object boundaries, and duplicate-member rejection. `unknown` is non-settlement evidence and
must be reconciled without retrying the mutation. Compatibility is candidate-only until the
producer declaration equals the candidate digest; the source commit/tree and synthetic-capture
provenance are recorded in the capture manifest, and deterministic cases are bound by the root
conformance file and fifteen projected goldens.

The candidate digest is computed from the checked-in source schema. It intentionally differs from
the current producer declaration; no consumer may negotiate this candidate until the producer is
rebuilt against the exact schema and digest. The schema uses session-scoped `peer:` identities,
the five action parser values (`play_card`, `end_turn`, `select_card`, `choose_reward`, and
`confirm_selection`), the installed action/event/relic witness names alongside the two synthetic
capture names, the installed checksum status vocabulary, and nullable remote checkpoints while a
native checksum is unread. Recovery requests and settled/rejected
recovery responses carry a `reconcile` value; the source's unknown pending-rejoin response may
carry `rejoin`. Only `rejoin_request` is mutation-shaped. The managed source has no cancelled
outcome, so `cancelled` is intentionally not in this candidate status enum.

The exact source projection and wrapper/golden hashes are recorded in
`artifacts/coop-native-v1/producer-capture.json`. The fifteen goldens are projections of eight
source capture calls, not hand-shaped examples.

## Admission gate

Admission requires the rebuilt producer digest, at least two named cross-repository consumers,
source and artifact byte identity, deterministic conformance including duplicate-key rejection,
and live evidence supplied by the owning boundaries. Until then, this record and its vectors are
proposed evidence and do not alter any supported profile.
