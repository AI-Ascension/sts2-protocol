// SPDX-License-Identifier: MIT

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(crate) type TestResult = Result<(), String>;

pub(crate) const SOURCE_SCHEMA: &str =
    include_str!("../../../../schemas/game-facts-reference-v1.schema.json");
pub(crate) const ARTIFACT_SCHEMA: &str =
    include_str!("../../../../artifacts/game-facts-reference-v1/schema.json");
pub(crate) const MANIFEST: &str =
    include_str!("../../../../artifacts/game-facts-reference-v1/manifest.json");
pub(crate) const CHECKSUMS: &str =
    include_str!("../../../../artifacts/game-facts-reference-v1/SHA256SUMS");
pub(crate) const CASE: &str =
    include_str!("../../../../conformance/cases/game-facts-reference-v1.json");

pub(crate) const VECTORS: &[&str] = &[
    "capabilities-live-synthetic.json",
    "capabilities-static-only.json",
    "error-invalid-binding.json",
    "error-missing-capability.json",
    "error-stale-snapshot.json",
    "error-unknown-id.json",
    "query-live-request.json",
    "query-static-request.json",
    "response-found-live-synthetic.json",
    "response-found-static.json",
    "response-unsupported-field.json",
];

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn golden(name: &str) -> Result<Vec<u8>, String> {
    let path = root()
        .join("artifacts/game-facts-reference-v1/golden")
        .join(name);
    std::fs::read(path).map_err(|error| error.to_string())
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
