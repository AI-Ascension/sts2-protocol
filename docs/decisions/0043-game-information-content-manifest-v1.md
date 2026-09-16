# 0043 — Typed game-information content manifest transport

`game-information-content-manifest-v1` transports the typed #83 producer output
without semantic input or localized text exposure. Its `inventory_revision` is
the canonical value bound by query-v1 `content_manifest_id`; no registry-only
digest may substitute for it. The producer fails with `missing_capability` until
its native source supplies every `ContentCatalogSnapshot` member coherently.
