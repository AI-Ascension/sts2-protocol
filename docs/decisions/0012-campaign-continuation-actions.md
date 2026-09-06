# ADR 0012: Campaign continuation and selection controls

- Status: Proposed; protocol candidate, consumer and host verification pending
- Date: 2026-09-06

The current gameplay catalog cannot represent the visible Proceed control after rewards,
shops or rest sites, or explicit confirmation/cancellation of a card selection. Reusing
`skip_reward`, `event_choice`, or a fabricated card identifier would obscure the intended
mutation and break receipt/replay semantics.

Add argument-free `proceed`, `confirm_selection`, and `cancel_selection` actions. The
game-mod remains the authoritative producer; gateway, MCP and harness are named consumers.
Each action exists only in a current host-generated catalog and retains the enclosing
instance/session/lease/generation/state/operation lifetime. JSON objects are closed and
contain only `kind`. Unknown kinds, extra fields and stale digests fail closed.

`proceed` activates a currently available continuation control. `confirm_selection` commits
the host's current selection. `cancel_selection` cancels it only when the host permits
cancellation. The protocol does not decide where these are legal, which choices to make,
or whether a transition settled. Existing action identity and settlement-witness rules apply.

This changes the schema digest. It is an explicit incompatible artifact revision under the
existing profile: every producer/consumer must migrate together. No mixed-digest session or
silent downgrade is permitted. Runtime-v1/v2 and provider contracts are unchanged.

Deterministic vectors cover all three request payloads, exact round trips, closed-object
rejection, schema/Rust agreement, and rejection of the prior digest. Provenance remains
project-owned, hand-authored MIT synthetic data. These tests do not prove live controls.
