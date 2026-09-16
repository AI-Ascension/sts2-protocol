# Game-information content manifest v1

This closed, bounded whole-manifest transport carries the exact typed output of
`ContentManifestProducer`. Producers refuse payloads larger than 16 MiB; they do
not page or truncate this contract. Query-v1 `content_manifest_id` names the
canonical `inventory_revision` only after the owner has produced this manifest.
