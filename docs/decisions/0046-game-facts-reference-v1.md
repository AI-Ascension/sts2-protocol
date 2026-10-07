# ADR 0046: game-facts-reference-v1 typed rule-input contract

- Status: proposed additive candidate
- Owner: `sts2-protocol`
- Consumers: none adopted
- Related issue: `AI-Ascension/sts2-game-mod#207`

## Context

The existing closed game-information messages do not carry typed rule inputs.
Issue #207 identifies agreement on supported rules and a negotiated typed
payload as cross-repository work. The contract can be reviewed before the game
owner supplies an actual supported-rule inventory, build/mode binding, and
source evidence. Those missing facts remain producer-owned.

## Decision

Add an independent `game-facts-reference-v1` schema and codec. Requests carry
an ordered, duplicate-free set of at most 16 opaque core rule IDs, rules
reference version 2, and a static or live binding. Responses echo the query and
provide one tagged `found` or `unsupported` outcome per requested ID, in the
same order. A found outcome contains up to 64 named inputs. Unsupported
combinations are explicit and bounded to 256 records.

Each input carries a unit, applicability, optional observation, and bounded
opaque source reference. Values are tagged safe integers, booleans, or entity
references. Multiplier numerator and denominator remain independent integer
inputs with unit `multiplier`; this contract supplies no rational value,
formula, modifier order, rule catalog, or support decision. Non-available
observations have null values and a bounded reason. Static inputs have null
observations.

The profile reuses the query-v2 instance and snapshot shapes and bootstrap-v1
scope shape. A live query binds the matching instance, snapshot and generation;
the authenticated scope's authority epoch remains a separate owner axis. A
null snapshot policy means a static-only capability offer. A non-null policy
describes generation-based expiry and invalidators; it makes no claim about
the current producer.

The envelope is closed, duplicate-rejecting and limited to 262144 UTF-8 bytes
for this new profile. Every envelope includes `query`, `result`, `capabilities`
and `error`; unused slots are explicit nulls. JSON Schema character bounds and
typed UTF-8 byte bounds are separate checks. Source references are opaque
ASCII tokens matching `^[A-Za-z0-9_-]+(?:[.][A-Za-z0-9_-]+){0,7}$`, with a
stricter runtime rejection for a case-insensitive `file` prefix or an
`exception` token; they are never paths or host exceptions. A response decoder
validates its embedded query, while the caller-facing join API also compares
correlation ID and the full echoed query to the original request. An error
without an echoed query cannot be bound to a specific request by this API.

The Protocol codec checks duplicates, bounds, value/unit compatibility,
reference equalities, input-name uniqueness and query/result ordering. It does
not contain the Core RuleId catalog or prescribe Core input ordering. The
producer remains responsible for both.

## Ownership and compatibility

The Protocol package owns the wire shape, bounds, codec, and conformance
vectors. Game-mod owns actual build/mode support, source-to-input mappings,
source evidence, applicability and any exactness claim. Game-core owns its
rule catalog and calculation semantics. Gateway owns transport fencing and
caller authorization; Harness owns its authority epoch. This contract does
not combine these responsibilities.

The change is additive. Existing query-v1/v2, runtime-v4-expert, and other
profile schemas, digests, and behavior remain unchanged. No consumer is listed
as adopted until its implementation and compatibility evidence are reviewed.
Gateway, MCP and Harness are prospective integration points only. Goldens are
synthetic fixtures, including the live example; they are not native or live
acceptance evidence.

The intended rollout is Protocol schema and codec first, then a game-mod
producer pinning the schema digest and rules-reference version alongside
owner-reviewed build/source evidence. Gateway can expose the new profile only
after transport fencing and authorization are tested; its lease epoch remains
a fence and is not substituted for either game-mod's instance epoch or
Harness's authority epoch. MCP can pin the profile and use the response join
API. Harness may supply its separately owned scope epoch. These are planned
integration roles, not current implementations or adoption claims.

## Consequences

Consumers can agree on a typed envelope and reject malformed or ambiguous
payloads while unsupported rule inputs remain explicit. The codec does not
turn a source-shaped observation into a verified game rule. Producers must
preserve request order, identify the selected inventory, and supply an
evidence-backed source token before marking a result source-derived or
confirmed. Exactness remains a producer/core joint determination outside this
transport contract.
