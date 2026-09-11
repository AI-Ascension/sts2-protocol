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

The coordinator's final actual Train review2 evidence records two identical
11,506-byte candidate-3 exports, with all inventoried source files preserved:

- ZIP SHA-256: fe10fe2d9674493469f17a51f59d2b01f07b4cbd1c5179e061148e97bf476317.
- Semantic digest: e5bd1aaac5209573192861c81f3bbac6b6356c24f48c129b52e93e508454b686.
- Reconciliation: 240 source records = 9 emitted (8 events, 1 accounting) +
  231 filtered. Seed start is settled; gameplay is episode_failed and process
  evidence is failed. Both gameplay action outcomes remain unknown.
- Protocol, Studio and observability CLIs passed. Cross-consumer checks confirmed
  exact stored events, accounting, omissions, identities, provenance and evidence.
- Real MLflow recorded 19 spans and one revision; retry and restart preserved
  the trace. Browser import, numeric observations, privacy regressions and node
  contrast passed through the LAN URL from VM-origin Chromium.

The final source revisions are harness
a065fe5187afefa9deccba2355ed5c1f00ba20ac, Studio
ad9f764c5caf6a7208b55d380e4f79f7ad8e6455 and observability
f6861dffb0f45cdc01f122c027f219437f4773fd. Harness reports 656 tests passed,
zero failed and five ignored, plus passing format, Clippy, strict policy and
build checks. Its independent reviewer closed the remaining specific findings,
independently reran all five review2 cases and validated the actual Train ZIP.
That review confirmed seed settlement, three raw-seed omission rows and the
invalid-versus-unsupported accounting distinction, with no new defect found.

The workspace integration ledger preserves the execution evidence in
`recorded-run-integration/train-candidate3-review2-export-result.json`,
`train-candidate3-review2-seed-evidence.json`,
`train-candidate3-review2-cross-consumer.json`,
`train-candidate3-review2-backend.json` and `train-lan-candidate3-review2.json`;
review closure is in `recorded-run-integration/reviews/harness-independent.md`.
These are coordinator execution and bounded independent-review evidence, not
protocol-owned host verification. Source completeness remains partial with an
unverified source snapshot. Laminar remains unverified. Final review2 content
verification used VM-origin Chromium. Separately, the user's Edge screenshot
from another LAN computer displaying http://192.168.1.146:4173 supplies
user-provided independent-LAN reachability evidence for the unchanged Studio
release. It predates the final review2 input and does not verify that input's
content. Seed launch settlement does not establish successful gameplay or
settled gameplay actions.

Protocol maintainers own technical contract admission: the exact schema, profile,
semantic rules, artifact inventory, compatibility classification and conformance.
Harness owns source detection, provenance, identity/field mapping, source-witness
validation, omission arithmetic and deterministic export. Studio and observability
each own their validation and consumption evidence. The coordinator records the
shared round trip and exact revisions; governance documents that evidence without
moving technical ownership or turning protocol checks into organization-wide proof.

## Before admission

The fresh export, shared consumer round trip and scoped harness review closure
are complete as recorded above. They supply evidence for an admission decision;
they do not themselves authorize admission.

1. Authorized protocol maintainers assess the final source-owner, consumer and
   independent-review evidence and its stated limitations. Resolve any additional
   findings required by that assessment before approving the contract.
2. Protocol maintainers record admission against the exact approved source commit
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
