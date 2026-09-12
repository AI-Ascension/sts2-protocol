// SPDX-License-Identifier: MIT

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use sha2::{Digest, Sha256};

const SCHEMA: &str = include_str!("../../../schemas/exact-checkpoint-reference-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/exact-checkpoint-reference-v1/schema.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/exact-checkpoint-reference-v1/manifest.json");
const CASE: &str = include_str!("../../../conformance/cases/exact-checkpoint-reference-v1.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/exact-checkpoint-reference-v1/SHA256SUMS");
const REFERENCE: &str =
    include_str!("../../../artifacts/exact-checkpoint-reference-v1/golden/reference.json");
const INVALID_PRIVILEGED: &str =
    include_str!("../../../artifacts/exact-checkpoint-reference-v1/golden/invalid-privileged.json");
const UNSUPPORTED_VERSION: &str = include_str!(
    "../../../artifacts/exact-checkpoint-reference-v1/golden/unsupported-version.json"
);

fn artifact_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts/exact-checkpoint-reference-v1")
}

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is valid JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles")
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn artifact_bytes_match_the_normative_source() {
    assert_eq!(ARTIFACT_SCHEMA, SCHEMA);
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
    assert_eq!(manifest["schema_digest"], hex(SCHEMA.as_bytes()));
    assert_eq!(
        manifest["consumers"].as_array().expect("consumers").len(),
        3
    );
}

#[test]
fn artifact_checksums_match_file_bytes() {
    let root = artifact_dir();
    let mut verified = 0;
    for line in CHECKSUMS.lines() {
        let (expected, relative) = line.split_once("  ").expect("checksum line");
        let bytes =
            fs::read(root.join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"));
        assert_eq!(hex(&bytes), expected, "checksum mismatch for {relative}");
        verified += 1;
    }
    assert_eq!(verified, 8);
}

#[test]
fn the_envelope_has_no_privileged_digest_member() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema JSON");
    assert_eq!(schema["additionalProperties"], false);
    let properties = schema["properties"].as_object().expect("properties");
    for name in properties.keys() {
        assert!(
            !name.contains("digest"),
            "public reference schema must not define {name}"
        );
    }
    assert_eq!(properties.len(), 8);
}

#[test]
fn valid_reference_validates_and_privileged_or_future_ones_do_not() {
    let validator = validator();
    let reference: Value = serde_json::from_str(REFERENCE).expect("reference JSON");
    validator.validate(&reference).expect("valid reference");

    let privileged: Value = serde_json::from_str(INVALID_PRIVILEGED).expect("reference JSON");
    assert!(
        validator.validate(&privileged).is_err(),
        "a privileged member must be rejected"
    );
    let future: Value = serde_json::from_str(UNSUPPORTED_VERSION).expect("reference JSON");
    assert!(
        validator.validate(&future).is_err(),
        "an unsupported reference version must be rejected"
    );
}

#[test]
fn case_file_binds_the_fixtures() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    assert_eq!(
        case["contract"],
        "sts2.protocol/exact-checkpoint-reference-v1"
    );
    assert_eq!(case["setup"]["live_runtime"], false);
    assert_eq!(case["goldens"].as_array().expect("goldens").len(), 1);
    assert_eq!(case["invalid"].as_array().expect("invalid").len(), 2);
}
