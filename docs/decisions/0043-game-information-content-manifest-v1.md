# 0043 — Typed game-information content manifest transport

`game-information-content-manifest-v1` transports the typed #83 producer output
without semantic input or localized text exposure. Its `inventory_revision` is
the canonical value bound by query-v1 `content_manifest_id`; no registry-only
digest may substitute for it. The producer fails with `missing_capability` until
its native source supplies every `ContentCatalogSnapshot` member coherently.

The closed response variant requires a manifest object; the error variant requires
an error object and no manifest. Error mapping is: source unavailable to
`missing_capability/source_unavailable`; source access denied to
`access_denied/source_access_denied`; source malformed and typed catalog validation
failures to `malformed` with the matching closed reason token; and an encoded
envelope over 16 MiB to
`result_limit_exceeded/serialized_payload_too_large`. Reasons are fixed tokens,
never free-form diagnostics or content. The protocol codec enforces the complete
UTF-8 byte bound before emission and on decode. Integration of that codec into
the #83 game-mod producer remains pending; the protocol contract does not claim
that owner-side integration is complete.
