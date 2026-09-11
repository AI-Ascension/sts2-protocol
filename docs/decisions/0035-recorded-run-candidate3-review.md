# ADR 0035: recorded-run candidate 3 review corrections

Status: proposed, not admitted. 2026-09-11. Extends ADR 0034 with a new candidate
revision after Hypatia's independent review of candidate 2.

Six findings are accepted: wrong STS2 process-result source stream, missing
unsupported-value diagnostic digests, inconsistent optional-profile grammar,
invalid disposition/reason pairs, parsing past the combined record budget, and
copying oversized library inputs before rejection. Candidate 3 enforces the
strict source projection without restricting unrelated common/optional profiles.
It also verifies each case digest and profile/version/schema metadata.

The suggested blanket inequality between recording and runtime/session identity
tuples is rejected. Identity fields and authority roles must stay distinct; no
source-owner lifetime rule proves that tuple equality itself is invalid. Clarify
the overly broad candidate-2 prose and retain aliases without inferring a join.
Positive regression cases cover exact tuple aliases and coincident value strings
under different namespaces. The actual Train model-execution identities remain
independent and unequal; this does not change their source mapping.

Candidate 2's existing artifact and schema bytes remain unchanged. Its original
tooling and dependencies within this repository are preserved under
history/recorded-run-candidate2; the archived validator can still validate its pin.
New schema, artifact, and conformance case use explicit candidate3 paths. Active
tooling targets candidate 3 only; no silent fallback admits older wire versions.

Source facts, privacy boundaries, inspection-only capabilities and ownership remain
as ADR 0034 defines. Producer source validation cannot be replaced by bundle
validation. Admission requires the corrected consumer implementations, a fresh
real-export round trip and independent review; current synthetic passes alone
do not satisfy these gates.
