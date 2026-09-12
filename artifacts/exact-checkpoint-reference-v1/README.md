# `exact-checkpoint-reference-v1` public reference contract

This artifact is the digest-free public reference for an exact checkpoint. Ordinary agents,
transcripts, logs, and dashboards may carry the keyed `ckpt-h1:` handle plus the boundary and
assurance summary; they must never carry an exact-state, checkpoint, blob, or compatibility digest.
The schema closes the object, so a privileged member is a validation error rather than silently
ignored data.

The envelope is inert. A valid reference does not prove coverage, restorability, or that the handle
was honestly issued; those are established by the trusted producer, the coverage contract, and a
verified restore. An unsupported `reference_version` must be rejected without lossy coercion.

`golden/reference.json` is a valid synthetic reference. `golden/invalid-privileged.json` adds an
exact-state digest and must fail schema validation. `golden/unsupported-version.json` names a future
version and must be rejected. See
[ADR 0037](../../docs/decisions/0037-public-checkpoint-reference-contract.md).
