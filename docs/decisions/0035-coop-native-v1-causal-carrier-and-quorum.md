# ADR 0035: keep the causal native carrier behind `coop-native-v1`

## Status

Proposed on 2026-09-11, pending owner implementation and independent component
verification. This record changes no schema, artifact bytes, digest, profile
selection, consumer admission, or live-compatibility claim.

## Context

`coop-native-v1` already carries the external identity needed to correlate a
single mutation: `instance_id`, `session_id`, `lease_id`, `lease_epoch`,
`operation_id`, `actor_peer`, and `expected_host_generation`. Its response
shapes preserve that operation ID and carry an authoritative receipt,
observation, and, only for settlement, an effect. Its peer roster is bounded to
two through four session-scoped opaque peers.

The game-mod owner reported `INetGameService.SendMessage<T>` and
`RegisterMessageHandler<T>` candidates for `INetMessage`, plus a mod-subtype
discovery path in `MessageTypes.Initialize`. Its independent review found the
current `Assembly.LoadFrom`/`GetTypes` probe unacceptable for this
metadata-only reservation: it can load target dependencies, may run module
initializers, and can silently report an incomplete type set. Consequently this
is not admitted metadata evidence until game-mod replaces it with complete
`PEReader`/`MetadataReader` inspection. Before the candidate may be called
verified, that replacement must emit every relevant registration/send method
definition and overload with decoded signature, visibility, static/interface
context, and explicit unresolved-signature diagnostics; a synthetic managed
fixture with a module initializer, non-public candidate, and overloaded
registration methods must prove the probe neither loads nor executes its target
while producing the complete inventory. Even then it identifies only a
first-party native-message candidate; serializer shape, type-ID allocation,
authenticated delivery, ordering, and host-to-origin-client return remain
separate gates.

The existing envelope has no place for a native message ID, and one is not
needed merely because the internal carrier exists. Adding one now would change
the closed profile without evidence that it crosses a boundary whose owner
needs protocol representation.

## Decision

Keep `coop-native-v1` frozen. The game-mod may implement an **owner-local**
`INetMessage` carrier only if it copies the existing v1 request identities
without changing their spelling or meaning. The carrier is not a gateway
envelope, protocol artifact, or second trust authority.

For a local action or shared vote, the carrier must retain the exact external
`operation_id`, the source `actor_peer`, and the request's generation/session
fence. The host must bind the native sender to its current local peer before
recording that operation, reject a mismatched or stale carrier before native
mutation, and return a host-issued witness to that same authenticated native
peer route. The receiving local producer then emits the existing v1
`effect_response` or `recovery_response`: envelope, receipt, and effect
operation IDs must all equal the original request operation ID. A matched action
fingerprint, actor, or generation is never an alternative correlation method.
Carrier session and lease values are correlation/fencing data only at the host;
gateway remains the sole authority that authenticates, issues, and validates
that route's session and lease.

Gateway authenticates and binds a local producer route to the actor peer before
forwarding the v1 request. MCP may expose that gateway-bound peer but must not
accept a caller-selected replacement. Harness retains the original operation
against its local route and never creates a second mutation during recovery.
The host remains the only source of legality, native sender identity, receipt,
all-peer observation, and settlement.

No v2 profile is authorized by this decision. A versioned artifact is required
only if a reviewed carrier needs a new value to cross this neutral boundary or
if v1 cannot preserve the original operation through the host-issued witness.

## Negotiated harness roster, session, and settlement quorum

This is an implementation agreement among owners, not a new v1 payload.
Before scheduling a mutation, harness must obtain a current v1 observation from
each selected local route and establish an owner-local cohort record containing:

| Item | Required rule |
| --- | --- |
| Profile | Every route reports `coop-native-v1` and the exact accepted digest. Unsupported profile or digest stops the cohort before mutation. |
| Participants | Two through four routes; each route has one gateway-authentication **route token** and one canonical v1 `peer:…` identity. The route token is credential material only: it is not a v1 peer value and is never copied into an envelope, catalog, observation, receipt, effect, or harness artifact. Gateway binds it to exactly one canonical peer identity. That bound identity must equal the route observation's sole `role: local` `peer_token`, and the union of each observation's peer tokens is exactly the same roster. No duplicate peer identity, missing selected participant, or extra peer is schedulable. |
| Session fence | Each participant record retains its own `instance_id`, `session_id`, `lease_id`, and `lease_epoch`. These values are never substituted from another route, and a released/changed lease or route-token rotation/rebinding invalidates its pending operation rather than remapping it. |
| Native run and authority | All observations must agree on `run_id` and cross-peer comparable `authority_id`. `host_authority_epoch`/`authority_epoch` is process-local and may fence its own route but must not be used as a globally comparable host identity. |
| Ready roster | Every roster entry is connected, not loading, not divergent, and has a known compatible checkpoint/state identity under the host's current observation. Otherwise scheduling stops or reports unavailable; the harness does not invent a quorum. |
| Actor admission | The selected action/vote is present in that local peer's current legal catalog; its `actor_peer` equals the gateway binding's canonical peer identity and the observation's sole local peer token. It must never equal or derive from the route-token credential. Its expected generation equals the fresh catalog/observation generation. |

"Quorum" is deliberately not a model-side vote count. Vote policy and action
legality remain host-owned. For a result to be surfaced as `settled`, the
host-issued response must carry the original operation ID, a fresh observation,
a settled receipt/effect, and all current roster peers must be connected,
non-loading, non-divergent, with the host's matching native-checksum/convergence
predicate satisfied. If that evidence is unavailable, the result remains
`accepted` or `unknown`; it is not upgraded by matching harness observations.

Recovery is valid only on the original participant route and exact
`(instance_id, session_id, lease_id, lease_epoch, operation_id)` binding. A
changed authority, route, lease, roster, or missing carrier witness is rejected
or unknown according to the owner response; harness must not retry the action
as new work. Rejoin refreshes the affected participant's observation and the
whole cohort's ready-roster check before scheduling resumes.

## Required implementation evidence and handoff

1. **Game-mod**: define the mod message's serializer/type-ID behavior, bind its
   sender to the local native peer, treat carrier session/lease values as
   non-authorizing correlation data, preserve the original v1 operation ID, and
   return a host witness to the origin route. Component tests need mismatch,
   stale-generation, duplicate, same-operation recovery, and host-authority
   change cases. The native message itself is not copied into this repository.
2. **Gateway**: bind an authenticated local route credential plus its
   session/lease fence to one canonical v1 peer identity; retain a pending
   original-operation record; invalidate that record on credential rotation or
   rebinding; reject caller-provided peer substitution; and forward only a
   return matching the original credential, fence, and canonical peer binding.
3. **MCP**: obtain peer identity from gateway binding, preserve the original
   operation through tool mapping, and expose `accepted`/`unknown` without
   manufacturing settlement.
4. **Harness**: enforce the cohort table above, schedule only bound legal
   actions, serialize recovery on the original route, and record the exact
   request/return/receipt/effect identities and quorum evidence.

Only after those component paths exist may game-mod produce the producer-bound
v1 fixture set named in ADR 0034 and protocol bind the refreshed bytes. A
native two-peer trace remains a separate live acceptance gate.
