# save-profile-v1 candidate contract

This inert bundle specifies bounded save-profile observations and retained operation metadata.
The manifest lists prospective owners and consumers; none is adopted by these synthetic vectors.
The source schema and `schema.json` are identical. `conformance.json` is also copied verbatim from
the source case. Verify every bundle byte against `SHA256SUMS` before using its exact profile and
schema digest. A schema-only pass is insufficient: the following semantic and join rules are
normative too.

## Ownership and identifiers

Game-mod owns actual save slots, host compatibility, active-run state, freshness baselines,
host-thread selection and operation settlement. Gateway owns caller/lease admission, approved
process configuration, namespace exclusivity, physical allocation, durable intent and launch
coordination. MCP maps through Gateway. Harness retains validation and coordination authority.
Protocol describes values and relationships; it authenticates no producer and performs no effect.

All identifiers are opaque ASCII tokens of 1–128 characters matching
`^[A-Za-z0-9]+(?:[._-][A-Za-z0-9]+)*$`, or exact numeric identities as specified below. No field can
carry a filesystem path, executable, shell command, account details, save contents, unlock data or
an arbitrary diagnostic. The host compatibility token names a producer-owned supported context; it
is not a Protocol compatibility attestation.

| Identifier | Namespace and lifetime |
| --- | --- |
| `scope.instance_id` | Gateway instance; every frame and original intent name the same selected instance |
| Caller/session/lease IDs and `lease_epoch` | Current request admission, separately retained in the original operation scope |
| `process_profile_id` | Gateway-approved numeric process profile |
| `user_data_namespace_id` | Gateway-private namespace with exclusive lifecycle ownership |
| `launch_binding_id` | Gateway-issued configuration incarnation; replace on any private launch mapping change |
| `allocation_id` | Retained numeric directory reservation inside that exact profile/namespace binding; never reused |
| `process_generation_id` | Gateway-owned exact process generation, not a PID or a lease epoch |
| `producer_incarnation` | One Mod producer lifetime; never reused after restart or generation exhaustion |
| `observation_generation` | Monotonic profile-state version within that exact process/producer lifetime |
| `slot_id` / `current_slot` | Host-owned logical slot inside the allocation; never a directory or process profile |
| `baseline.identity` / `baseline.digest` | Host-owned freshness reference and lowercase SHA-256 digest for that allocation |
| `operation_id` | Immutable intent key within its original instance/caller/session scope |
| `correlation_id` | One request/response exchange; retry and lookup may use a new correlation |

Numeric identities and lease epochs are 1 through 9,007,199,254,740,991. Observation and unattached
lifecycle generations also allow zero. Values from different namespaces are independent even if
their numbers happen to match. Changing a launch binding is not an implicit adoption of an old
directory, operation, owner or slot.

## Discovery and selection

Discovery carries a server-resolved `binding`, with no caller baseline or selected slot. The owner
returns the same binding, a baseline and owner fence, explicit run state, capabilities, up to 32
slot summaries and a nullable current slot. Slots are strictly sorted by ASCII slot ID and unique;
a non-null current slot must be listed. Empty slots plus null selection is a valid discovery.
`active_run` is exactly `inactive`, `active` or `unavailable`. Neither unavailable nor a missing
capability can be interpreted as inactive or supported. Discovery performs no save/profile writes.

Every selection intent binds its original scope, operation ID, an inactive `admission` containing
the exact binding, process/producer/generation and baseline, and one desired slot. Mod rechecks
all those values, current run state, slot support and availability immediately before a host-thread
effect. Baseline equality alone is insufficient. Generation advances whenever slot inventory,
selection, baseline or active-run state changes. Exhaustion requires a new incarnation; a selection
at the maximum generation cannot be admitted by this profile.

The host keeps durable idempotency or an explicitly equivalent retained operation authority.
An identical original intent replays the retained operation; changed reuse refuses without a new
effect. A successful selection receipt requires an available exact target as the current slot, an
inactive readback, `selection: true`, the same binding/process/producer and a strictly newer
observation generation. Its `slot_selected` witness binds that owner and the original intent digest.
A fresh observation of the desired slot alone cannot settle an uncertain selection.

## Fresh allocation and launch binding

`create_disposable` is Gateway-owned coordination. Its immutable intent names the approved profile
and one of two admission forms. `observed_owner` carries an exact inactive source fence under that
same profile. `unattached` carries Gateway's exact selected-instance lifecycle generation and
means there is no attached owner. It does not assert inactive game state or authorize host mutation.
An unattached instance permits fresh directory preparation before a Mod baseline exists.

Gateway first durably reserves the operation, allocation ID and private launch mapping. It verifies
the namespace/root is its own and refuses unknown contents, symlinks, traversal, implicit adoption
or overwrite. A `reserved` allocation means a retained identity with no claimed directory creation;
`created` requires durable confirmed physical ownership; `unknown` preserves uncertain physical
work. None is a Mod slot or a successful profile operation. No portable field carries the root.

Actual process launch needs its separately admitted lifecycle authority. The newly launched Mod
must establish that its real host user-data context is the exact Gateway binding before reporting
it; echoing an environment variable or request is not this proof. A matching inactive observation
is required before any host selection is forwarded. Creation itself authorizes no stop, restart,
live user-data switch or host slot mutation. A first-launch default slot may be observed without
claiming that Gateway created or selected that logical slot.

Creation settles only with a `created` allocation whose operation/profile match the intent, an
exact allocation readback, an available current slot, inactive run state,
`disposable_readback: true`, and the Gateway's retained `disposable_profile_ready` witness. For an
observed source owner the new allocation ID differs, and both process generation and producer
incarnation differ. An existing process cannot be relabeled as using a fresh directory. Pending
launch or unsupported/missing native readback remains an unfinished or blocked operation.

## Outcomes and lookup

| Status | Required meaning and representation |
| --- | --- |
| `accepted` | Intent retained; no readback, witness or reason. Allocation may be absent, reserved or confirmed created. |
| `settled` | Exact operation-linked witness and valid readback; no reason. Selection has no allocation receipt; creation has a created allocation. |
| `unknown` | Outcome uncertain; reason `outcome_unavailable`, no settlement readback/witness. Retain any reserved or uncertain allocation identity. |
| `rejected` | Known refusal before physical allocation/host effects; a retained reserved identity is allowed, a created/unknown allocation is not. |
| `cancelled` | Confirmed cancellation before those effects; reason `cancelled_before_write`. A known uncreated reservation remains retained. |
| `blocked` | A known refusal or preserved partial allocation; no settlement claim. Allocation cannot be unknown. A fixed refusal reason is required. |

Every non-settled receipt has null readback and witness. Refusal reasons are closed tokens, not host
exceptions. `outcome_unavailable` is reserved for unknown; `cancelled_before_write` is reserved for
cancelled. Cancellation never deletes a directory or erases an uncertain operation.

Lookup uses the same original operation ID, original scope and immutable intent digest. Its current
request scope must name
the same instance, caller and session but may carry a newly admitted lease/epoch. Gateway must
authenticate that current scope; the protocol does not authorize lease rotation. The returned
receipt retains the full original intent, including its old lease. Correlation and current lookup
scope never become a new operation identity. A historical settlement is not a fresh active-run
admission observation.

`not_found` echoes the exact original operation ID, scope and intent digest. It is an absence result from the
queried retention authority, not proof of no prior effect. Lookup is read-only, including after
restart or reply loss. An error, not-found result or unknown outcome never authorizes blind
resubmission, allocation of a replacement ID or selection under a new key. Capacity exhaustion
must refuse before an effect; unconfirmed operations and their identity reservations cannot be
evicted to admit a duplicate.

## Frame closure, response joins and canonical bytes

Every frame has these members in canonical order:
`protocol_version`, `schema_digest`, `kind`, `correlation_id`, `scope`, `request`, `result`, `error`.
All are required. Unused payloads are explicit null, never omitted. Request kinds and response
payloads are constrained by the schema; result-specific identities and outcomes are also checked
by semantic conformance. A response echoes the exact request, correlation and current scope.
Before use, consumers must join those three values to their own retained request; validation of an
embedded request alone is insufficient. Error responses with a null request are unbound failures
and cannot pass that join. An error concerns this request, never whether an older mutation ran.

Objects reject unknown and duplicate members, including escaped duplicate names. Unknown profile,
digest, kind, status or enum values are rejected; there is no success fallback for an unknown status.
Exact mathematical integer spellings such as `1`, `1.0` and `1e0` have the same value, but fractions
and unsafe integers are refused before any floating-point rounding. Encoders use the plain decimal
integer spelling. Complete input/output frames are bounded at 65,536 UTF-8 bytes. Object order on
input is irrelevant; arrays retain their specified order.

Canonical encoding is compact UTF-8 JSON, no BOM, spaces or trailing newline, standard lowercase
booleans/null, and these member orders. Every tagged variant emits `kind` first, then its listed
members. Tokens are ASCII, so Unicode escaping cannot create an alternate canonical token spelling.

| Object / variant | Ordered members after any `kind` tag |
| --- | --- |
| Scope | instance_id, caller_id, session_id, lease_id, lease_epoch |
| Launch profile | process_profile_id, user_data_namespace_id, launch_binding_id |
| Binding | instance_id, profile, allocation_id |
| Owner | process_generation_id, producer_incarnation, observation_generation |
| Baseline / fence | identity, digest / owner, baseline |
| Admission | binding, fence, active_run |
| Provisioning unattached / observed_owner | lifecycle_generation / observation |
| Select / create_disposable mutation | admission, slot_id / profile, admission |
| Immutable intent | operation_id, origin, mutation |
| Discover / mutate / lookup request | binding / intent / operation_id, origin, intent_digest |
| Slot / capabilities | slot_id, status / selection, disposable_readback |
| Observation | binding, fence, host_compatibility, capabilities, active_run, slots, current_slot |
| Allocation | owner, operation_id, binding, status |
| Witness | kind, operation_id, intent_digest, owner; kind is its first member |
| Receipt | intent, status, allocation, readback, witness, reason |
| Discovery / operation / not_found result | observation / receipt / operation_id, origin, intent_digest |
| Error | code |

The intent digest is lowercase SHA-256 over the ASCII bytes `sts2-save-profile-v1-intent`, one NUL
byte, then the canonical compact immutable intent JSON. It excludes frame correlation and any
current recovery scope. Conformance provides independent fixed digest values and checks canonical
goldens. A digest binds bytes and relationships; it cannot authenticate a host or prove an effect.

The bundle adds no host implementation or native qualification. Mod production discovery and
selection, Gateway approved per-allocation launch composition, MCP/Harness mappings and native
preservation tests remain separate implementation/evidence gates. Synthetic goldens cannot satisfy
those gates.
