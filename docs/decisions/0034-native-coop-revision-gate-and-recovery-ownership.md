# ADR 0034: native co-op revision gate and recovery ownership

## Status

Proposed. Revised after root arbitration and independent review on 2026-09-11.
This record changes no schema, artifact, digest, consumer admission, or runtime
claim.

## Context

`coop-native-v1` is an accepted source/serialized **component** profile at
schema digest `2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629`.
The game-mod producer's deterministic capture covers observation, catalog,
action, vote, unknown/reconciliation, and rejoin/reconciliation. It does not
execute a native STS2 session.

Read-only inspection of the producer shows two relationships that deserve
explicit future conformance coverage:

1. A mutation whose `expected_host_generation` is not the fresh host generation
   is rejected with `stale_host_generation`, without dispatching a native
   mutation.
2. The host resolves the actor against its current local peer binding before
   dispatch. An opaque `peer:` string is not sufficient proof that a peer is
   current or local.

The v1 schema permits the envelope shapes used by those outcomes, but its
projected synthetic capture has no dedicated stale-generation case. The v1
producer also currently rejects client-side action and vote calls with
`native_action_requires_host_authority` and
`native_vote_requires_host_authority`; it therefore cannot yet show an
authenticated model-controlled local client receiving the host's settlement.

The existing envelope can serialize session/lease identity, an original
operation ID, an opaque peer, an expected host generation, a host authority ID,
a process-local authority epoch, a receipt, a host observation, and an optional
settled effect. This is necessary but not sufficient for a safe compatible
implementation. In particular, `authority_id` is cross-peer comparable but
`authority_epoch` is process-local; together they are not a universal host
identity. A v1 digest edit would invalidate the closed-envelope profile that
gateway, MCP, and harness already pin.

The similarly named `watchdog-recovery-v1` is different. Gateway and harness
currently own and pin its sideband frame digest
`fb934d3157485aaf6e13e6ebbb213ec8a14c7fc6f5eeebc06b7a22c1f0009217`.
The protocol target has no copied artifact with named game-mod and MCP recovery
consumers. Moving that lifecycle/control contract here now would duplicate
boundary authority and violate the target ownership rule.

## Decision

Keep `coop-native-v1` byte-for-byte frozen and keep `watchdog-recovery-v1`
owner-local. Do not relabel synthetic capture as native-session evidence.

The minimum required implementation path, if its owning boundaries can prove it
without adding a relay trust scheme, is:

1. Each native peer exposes its own authenticated local producer/session route.
   Gateway must bind that route's authenticated caller/session/lease lineage to
   the local peer; it must not accept a caller-chosen peer identity or
   synthesize a second host relay identity.
2. A model-controlled client sends `local_action_request` or
   `shared_vote_request` to its own local producer with its original
   `operation_id`, local `actor_peer`, and observed host generation.
3. The client producer invokes only the installed first-party peer path. The
   authoritative host decides legality and settlement. A host-produced v1
   `effect_response` must return through the initiating peer route with the
   same operation ID, plus authoritative receipt provenance and a fresh
   all-peer observation. A client-local pending receipt is not settlement.
4. `accepted` and `unknown` retain the original operation identity. Recovery
   through that same client route must reconcile that identity rather than issue
   another mutation. A changed authority is rejected as
   `native_authority_changed`, not treated as recovered settlement.

Gateway and MCP do not currently prove the required actor binding, and current
source inspection did not find an original-operation-correlated host-to-client
settlement return or client-side reconciliation capable of proving remote
checksum convergence. Thus these are implementation gates, not established v1
behavior. MCP must expose only the gateway-verified binding; harness must record
the bound peer and reconcile only the same operation. The game-mod alone issues
host observations, effects, and settlement receipts.

Extra fixture coverage alone is not a reason to migrate. A versioned v2
contract becomes essential only after the game-mod proves one causal native
correlation mechanism: either the original operation identity is propagated on
a supported first-party/mod peer-to-host message and returned on the same
authenticated peer link, or the native queue supplies a unique dispatch ID that
the host returns in its authoritative witness. Matching action contents, actor,
or generation is never correlation proof because separate native or human
requests can share that tuple. No fingerprint-only correlation field is
admitted.

## Required producer-bound v1 fixtures after the implementation gates

The game-mod must produce these compact JSON request/response projections from
its deterministic source capture, then protocol may bind their exact bytes:

| Fixture | Required source condition | Required result |
| --- | --- | --- |
| `stale-local-action` | Request generation differs from fresh host generation | `effect_response`, `rejected` receipt, `after_host_generation: null`, `error_code: stale_host_generation`, and no native dispatch witness |
| `client-local-action-settled` | Authenticated local client sends a v1 action through its own producer route | Same operation ID reaches an authoritative host receipt; response contains host authority/epoch, fresh observation, settled effect, and peer convergence witness |
| `client-local-vote-settled` | Authenticated local client submits a shared vote through its own producer route | Same client-operation/host-settlement lineage and peer convergence properties |
| `client-reconcile-original-operation` | Client transport outcome is unknown | Same client route reconciles the original operation ID and never emits a second mutation request |
| `authority-changed-recovery` | Host authority changes before client recovery | Reconciliation is rejected as `native_authority_changed`, with no settled effect |

These are component fixtures only. Each must record producer commit/tree,
source capture command, schema digest, wrapper hash, projection hash,
originating peer-route binding, authoritative receipt provenance, a
no-second-dispatch oracle, and whether native session execution occurred. None
can satisfy the two-peer native acceptance gate.

## Admission and migration gates

Before choosing v1 refresh versus v2, inspect the exact installed native
assemblies/API metadata to determine whether `RequestEnqueue`/the vote paths
provide a unique queue or message identifier, or expose a supported
peer-to-host/host-to-peer mod-message registration point. If neither exists,
do not infer correlation from a fingerprint: report the concrete missing native
mechanism to root for the next ownership decision.

Once a causal mechanism is proven, game-mod must component-test the first-party
original-operation-correlated host-to-origin-client path; gateway must bind
authenticated caller/session/lease-to-peer and retain the causal token; MCP
must refuse caller-selected peers not established by gateway; and harness must
exercise client request, authoritative host settlement, routed return,
original-operation recovery, and authority-change rejection. If a new causal
token must cross the neutral artifact, it requires a separately versioned v2
schema with a host-issued witness—never a fingerprint-only migration. At least
the game-mod producer and gateway/MCP consumers must independently validate the
exact selected-profile bytes. Live acceptance remains separate and requires the
objective's real two-peer evidence.

Issue #16 remains open until recovery has a genuine neutral artifact with its
canonical owner and named consumers, or its owners explicitly decide to retain
it locally. This ADR does not make either claim.
