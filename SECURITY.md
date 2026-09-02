# Security Policy

## Supported state

There is no released protocol artifact yet. Security fixes apply to the current development line.
Future releases will state their support window.

## Reporting a vulnerability

When this repository is hosted, use its private vulnerability-reporting form under the Security tab.
If private reporting is unavailable, open a minimal public issue requesting a private channel without
including exploit details, credentials, saves, personal paths, or proprietary host data.

Include the affected revision or artifact, impact, reproducible conditions, and the smallest safe
proof. Maintainers should coordinate a private fix and disclosure window and credit the reporter when
requested and appropriate.

## Protocol-specific risk boundary

Ambiguous identifiers, namespace collisions, stale versions, malformed envelopes, unsafe unknown-value
handling, unbounded fields, duplicate normative definitions, digest drift, secret leakage, and
provenance failures are security-relevant. This repository must not issue credentials, authorize
mutations, expose host objects, widen network reachability, or make provider decisions. Authentication
and authorization stay with the boundary that makes the decision.

Use synthetic identities and disposable fixtures for tests. Never test a future consumer against a
valued game profile, public listener, private save, live provider, or another person's data without
separate authorization.
