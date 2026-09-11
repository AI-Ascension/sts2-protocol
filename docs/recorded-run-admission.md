# Recorded-run candidate admission and release gates

Candidate 3 remains proposed at wire version 1.0.0-candidate.3. A local commit,
passing checks, or a bounded independent-review closure does not admit it.
Candidate 2's original artifacts and runnable source are preserved as history.

## Evidence and ownership

The independent candidate-3 closure review verified six validator corrections
and accepted the identity-role clarification at schema SHA-256
a6c32127290f4d5e670d8863f97a74a7b8e3e411e735d81394b51fe1578b4eb6 and inventory
580c1cf3be4bb3e4eb37b9acd9166808b7386b0eb84286cc0798a0d88e35bb35.
Its scope was the reported regressions, not exhaustive source or runtime proof.
Review commands passed 62 Node tests, 3 Rust schema tests and 44 archived tests.
The coordinator reports Studio ad9f764 and observability c5f9d36 pinned to
candidate 3; the fresh harness candidate-3 real-export round trip remains pending.
These consumer references are coordinator evidence, not protocol-owned runtime
verification.

Protocol maintainers own technical contract admission: the exact schema, profile,
semantic rules, artifact inventory, compatibility classification and conformance.
Harness owns source detection, provenance, identity/field mapping, source-witness
validation, omission arithmetic and deterministic export. Studio and observability
each own their validation and consumption evidence. The coordinator records the
shared round trip and exact revisions; governance documents that evidence without
moving technical ownership or turning protocol checks into organization-wide proof.

## Before admission

1. Complete harness review fixes and export the real source recording again,
   preserving the source. Record exporter/source versions, adapter revision,
   artifact bytes and source-to-output reconciliation.
2. Validate those exact candidate-3 bytes with the protocol oracle, then process
   the same artifact in both pinned consumers. Reconcile identities, counts,
   evidence, numeric observations and accounting, including unknown states.
3. Resolve any new independent-review or consumer findings. Record scope and
   limitations explicitly. Complete the applicable browser/tracking integration
   gates without substituting candidate-2 evidence for the new round trip.
4. Protocol maintainers record admission against the exact approved source commit
   and schema/inventory pins, with the named consumer evidence and migration
   classification. Any changed artifact metadata gets a new digest and explicit
   redistribution. Keep candidate-3 bytes and wire name unchanged until a
   coordinated version decision is made.

The checked-in candidate manifest deliberately retains status=candidate and
admitted_consumers=[] until these gates are met. Admission does not automatically
grant push, merge, tag, publication or deployment authority.

## Release is a separate authorized action

[RELEASING.md](../RELEASING.md) requires an exact approved source revision,
completed review and local/CI checks, schema/golden agreement, consumer evidence,
dependency notices from the exact lockfile, and inspection of allowlisted package
bytes from a clean build. The candidate's base_revision is its starting point,
not a claim that the new files already existed at that upstream commit.

[WORKFLOWS.md](WORKFLOWS.md) separates pull request/review, authorized merge,
candidate packaging and authorized publication. No publication job or write
permission is introduced by these changes. An explicitly authorized maintainer
must publish immutable tags/artifacts, then verify freshly retrieved bytes.
A correction uses a new version; never overwrite an accepted artifact or move a
tag. Retain a last-known-good consumer pin for rollback.

The current authorization permits a reviewable local commit only. Push, merge,
release and deployment remain outside this handoff. A future stable wire name
must be coordinated with all consumers; removing the candidate suffix silently
would be a new compatibility change requiring repins and revalidation.
