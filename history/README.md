# Historical candidate snapshots

recorded-run-candidate2 preserves original project-owned source and synthetic
artifact bytes for the candidate-2 review pin. It is a compatibility-history
snapshot, not a supported release or a second normative implementation.
Do not regenerate or patch it. Active development uses candidate 3.

Historical validation:

```sh
node history/recorded-run-candidate2/tools/recorded-run/validate.mjs artifacts/recorded-run-bundle-v1/golden/legacy-failed.zip
(cd artifacts/recorded-run-bundle-v1 && sha256sum -c SHA256SUMS)
```

The original artifact inventory remains
41d760f8c41064c4e6b49a48dbe6e1a6c8f2a9958afbc50374986a54858fd598.
tooling.json references resolve against the historical snapshot root when
checking the original validator and test hashes.
