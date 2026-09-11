# ADR 0034: recorded-run bundle candidate

Status: proposed, 2026-09-11. This additive candidate changes no existing runtime
artifact. It is not a release or organization-wide compatibility admission.

## Evidence and ownership

The coordinator's sanitized Train shape, format and receipt probes plus harness
and Studio source reviews establish a shared inspection need. The source marker
is seed-readiness-controller-release-v2. The source has six trajectory rows,
one decision, one accounting row and 230 MCP rows. Both action receipts are Unknown;
an episode_failed row exists, and the two model-execution identifiers are unequal.
These findings justify bounded projections, not raw producer forwarding.

Protocol owns the narrow envelope, schema, serialization, conformance and packaging.
Harness retains detector, field mapping, redaction and export. Studio and
ai-agent-observability are the two named candidate consumers. Game-mod retains
authoritative host evidence, gateway lifecycle, and MCP transport. No implementation
policy, game authority or persistent recording store moves into protocol.

## Decision

Author candidate bytes now so producer and consumers can implement and review them.
Use the closed schema and semantic rules in
[recorded-run-bundle-v1](../../artifacts/recorded-run-bundle-v1/README.md).
The common envelope separates source identity, semantic revision, evidence dimensions,
row reconciliation and inert capabilities. The legacy payload permits only reviewed
summaries. Common completion can be represented with an authority/witness reference;
the current legacy adapter has no supporting gameplay-completion evidence.

The protocol product crate remains inert. A Node standard-library conformance tool
reads an untrusted archive only on explicit CLI invocation. It enforces ZIP/JCS,
bounded expansion, strict schema and cross-field rules. Rust schema tests provide an
independent Draft 2020-12 check; synthetic fixtures and invalid vectors accompany it.
Original code and synthetic data are MIT. No third-party source or host data is copied.

## Admission and migration

Candidates can be implemented before admission but are pinned by exact schema and
artifact inventory digests. Consumer review, source mapping, independent validation
and real Train round-trip evidence remain pending. Subsequent wire/semantic changes
require a new candidate revision and revalidation; major incompatible changes require
a new profile/version. Old runtime/seed contracts are untouched.
