# ADR 0032: initialize the additive Runtime-v4 expert rest-action candidate

- Status: accepted for protocol contract initialization; downstream adoption pending
- Date: 2026-09-08
- Owner: `sts2-protocol`

## Context

The accepted `runtime-v4-expert` observation exposes player-visible rest-site options. The
separately admitted `runtime-v4-expert-action` profile is potion-only and settles only a
`potion_use_settled` transition. Rest options therefore need a separately versioned action profile
whose selector and effect evidence cannot change the old potion meaning.

## Decision

Initialize the candidate `runtime-v4-expert-rest-action-v1` profile with these fixed identities:

| Field | Value |
| --- | --- |
| Protocol version | `runtime-v4-expert-rest-action-v1` |
| Artifact | `sts2-protocol/runtime-v4-expert-rest-action` |
| Profile | `expert-rest-action` |
| Schema source | `schemas/runtime-v4-expert-rest-action-v1.schema.json` |
| Schema digest | `bb3555fae28eb1f79d08a15e9884696a579e4c20836f5016509f17e0f4c36fbd` |
| Generator | `hand-authored` |
| Witness version | `rest-effect-witness-v1` |

The schema and artifact are additive and transport-neutral. They carry typed `rest_option`,
`select_card`, `select_player`, `confirm_selection`, and `cancel_selection` action references,
generation-fenced transitions, and option-specific effect witnesses. A settled response requires a
fresh nested expert observation and a typed transition. Completed transitions require the same
effect witness at the root and transition; selector transitions require a complete typed catalog,
stable selection identity, and consistent selected and remaining counts. An unavailable or
ambiguous native result is `unknown` and is reconciled by the original operation identity.

The existing `runtime-v4-expert-action` potion profile, schema bytes, artifact bytes, endpoint, and
digest remain unchanged. A consumer must reject an unsupported REST profile before mutation and
must not reinterpret it as the potion profile or Runtime-v3.

## Ownership and adoption boundary

Protocol owns only the inert schema, artifact copy, provenance, checksums, synthetic fixtures, and
implementation-neutral conformance. The named prospective owners are `sts2-game-mod`,
`sts2-gateway`, `sts2-mcp-server`, and `sts2-harness`; the candidate manifest intentionally has an
empty admitted-consumer list. Adoption requires each owner to pin the exact digest and pass its own
serialized producer or consumer checks. The checked-in protocol evidence does not prove any of
those integrations, host compatibility, or live settlement.

The intended gateway assignment envelope is `POST /v4/instances/{instance_id}/expert-rest-action`
for dispatch and `GET /v4/instances/{instance_id}/expert-rest-actions/{operation_id}` for
reconciliation. JSON content type, authorization, and lease fencing remain gateway concerns. The
protocol crate implements no HTTP route, lifecycle, authentication, persistence, or mutation.

## Evidence and migration

The conformance case covers all 16 synthetic request/response goldens, 22 independent mutation
fixtures, and two serialized Smith/Mend producer-shaped lifecycles. Schema-valid mutations are rejected by
cross-field checks for action identity, transition context, selector counts/catalogs, visible legal
choices, prior selector admission, and witness identity/evidence. A completed selector may have a
final `rest` observation, so a stateful consumer retains the admission catalog from earlier
responses. Structural mutations remain schema-invalid. These checks establish candidate contract
closure only; they do not authorize a release or consumer claim.

The mod owner must first produce the exact serialized envelope from managed native evidence,
including Smith and Mend selector state. Gateway, MCP, and harness owners then validate round trips
against this artifact. A non-author verifier must rerun the protocol checks, old-potion byte
preservation, and downstream checks before this candidate can become admitted.
