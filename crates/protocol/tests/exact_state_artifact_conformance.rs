// SPDX-License-Identifier: MIT

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sts2_protocol::{CanonicalValue, ExactCheckpointId, ExactStateDigest, blob_digest};

const MANIFEST: &str = include_str!("../../../artifacts/exact-state-v1/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/exact-state-v1/SHA256SUMS");
const PAYLOAD: &str = include_str!("../../../artifacts/exact-state-v1/golden/state-payload.json");
const CANONICAL: &[u8] = include_bytes!("../../../artifacts/exact-state-v1/golden/state.canonical");
const GOLDEN_MANIFEST: &str =
    include_str!("../../../artifacts/exact-state-v1/golden/manifest.json");
const RECEIPT: &str = include_str!("../../../artifacts/exact-state-v1/golden/receipt.json");
const COVERAGE: &str = include_str!("../../../artifacts/exact-state-v1/coverage-contract.json");
const SOURCE_EXACT: &str = include_str!("../../../schemas/exact-state-v1.schema.json");
const SOURCE_MANIFEST: &str = include_str!("../../../schemas/checkpoint-manifest-v1.schema.json");
const SOURCE_COVERAGE: &str = include_str!("../../../schemas/coverage-contract-v1.schema.json");
const ARTIFACT_EXACT: &str = include_str!("../../../artifacts/exact-state-v1/schema.json");
const ARTIFACT_MANIFEST: &str =
    include_str!("../../../artifacts/exact-state-v1/checkpoint-manifest.schema.json");
const ARTIFACT_COVERAGE: &str =
    include_str!("../../../artifacts/exact-state-v1/coverage-contract.schema.json");

fn artifact_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts/exact-state-v1")
}

fn payload() -> CanonicalValue {
    CanonicalValue::parse_str(PAYLOAD.trim_end_matches('\n'))
        .expect("golden payload is inside the profile")
}

fn manifest() -> CanonicalValue {
    CanonicalValue::parse_str(GOLDEN_MANIFEST.trim_end_matches('\n'))
        .expect("golden manifest is inside the profile")
}

fn receipt() -> Value {
    serde_json::from_str(RECEIPT).expect("receipt JSON is valid")
}

#[test]
fn artifact_schema_bytes_equal_normative_sources() {
    assert_eq!(ARTIFACT_EXACT, SOURCE_EXACT);
    assert_eq!(ARTIFACT_MANIFEST, SOURCE_MANIFEST);
    assert_eq!(ARTIFACT_COVERAGE, SOURCE_COVERAGE);
}

#[test]
fn artifact_checksums_match_file_bytes() {
    assert!(!CHECKSUMS.is_empty());
    let root = artifact_dir();
    let mut verified = 0;
    for line in CHECKSUMS.lines() {
        let (expected, relative) = line.split_once("  ").expect("checksum line has two fields");
        let path = root.join(relative);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{relative}: {error}"));
        let actual: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(actual, expected, "checksum mismatch for {relative}");
        verified += 1;
    }
    assert_eq!(verified, 17);
}

#[test]
fn envelope_schemas_validate_the_recorded_fixtures() {
    for (schema_text, fixture_name, fixture) in [
        (SOURCE_EXACT, "golden payload", PAYLOAD),
        (SOURCE_MANIFEST, "golden manifest", GOLDEN_MANIFEST),
        (SOURCE_COVERAGE, "coverage fixture", COVERAGE),
    ] {
        let schema: Value = serde_json::from_str(schema_text).expect("schema JSON is valid");
        let validator = jsonschema::draft202012::options()
            .build(&schema)
            .unwrap_or_else(|error| panic!("{fixture_name} schema compiles: {error}"));
        let instance: Value = serde_json::from_str(fixture).expect("fixture JSON is valid");
        validator
            .validate(&instance)
            .unwrap_or_else(|error| panic!("{fixture_name} validates: {error}"));
    }
}

#[test]
fn golden_payload_canonicalizes_to_the_recorded_bytes_and_identity() {
    let value = payload();
    assert_eq!(value.to_canonical_bytes().expect("bytes"), CANONICAL);
    let digest = value.exact_state_digest().expect("digest");
    assert_eq!(
        digest.as_str(),
        receipt()["exact_state_digest"].as_str().expect("digest")
    );
    assert!(ExactStateDigest::parse(digest.as_str()).is_ok());
    assert_eq!(
        blob_digest(CANONICAL).as_str(),
        receipt()["canonical_payload_digest"]
            .as_str()
            .expect("blob")
    );
    assert_eq!(
        CANONICAL.len() as u64,
        receipt()["canonical_payload_size_bytes"]
            .as_u64()
            .expect("size")
    );
}

#[test]
fn golden_manifest_binds_the_payload_and_its_identifier() {
    let manifest = manifest();
    let digest = payload().exact_state_digest().expect("digest");
    assert_eq!(
        manifest.to_canonical_bytes().expect("bytes"),
        GOLDEN_MANIFEST.trim_end_matches('\n').as_bytes()
    );
    let recorded = serde_json::from_str::<Value>(GOLDEN_MANIFEST).expect("manifest JSON");
    assert_eq!(recorded["exact_state_digest"], digest.as_str());
    assert_eq!(
        recorded["canonical_payload"]["digest"],
        receipt()["canonical_payload_digest"]
    );
    let checkpoint_id = manifest.exact_checkpoint_id().expect("checkpoint id");
    assert_eq!(
        checkpoint_id.as_str(),
        receipt()["exact_checkpoint_id"].as_str().expect("id")
    );
    assert!(ExactCheckpointId::parse(checkpoint_id.as_str()).is_ok());
    assert!(ExactCheckpointId::parse(digest.as_str()).is_err());
}

#[test]
fn coverage_fixture_declares_every_required_status_field() {
    let coverage: Value = serde_json::from_str(COVERAGE).expect("coverage JSON is valid");
    let rows = coverage["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty());
    let row = rows[0].as_object().expect("row object");
    for field in [
        "field_group",
        "source_symbol",
        "phase",
        "visibility",
        "status",
        "capture_method",
        "restore_method",
        "ordering",
        "rationale",
        "acceptance_ids",
    ] {
        assert!(row.contains_key(field), "coverage row lacks {field}");
    }
    assert_eq!(coverage["adapter_id"], "fixture.exact_state.contract");
    let manifest: Value = serde_json::from_str(MANIFEST).expect("artifact manifest JSON");
    assert_eq!(manifest["canonical_profile"], "asc-jcs-state-v1");
    assert_eq!(
        manifest["consumers"].as_array().expect("consumers").len(),
        3
    );
}
