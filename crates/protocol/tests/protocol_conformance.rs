// SPDX-License-Identifier: MIT

use serde_json::Value;
use sts2_protocol::{
    ContractManifest, ErrorEnvelope, NeutralMetadata, ValidationError, canonical_json, decode_json,
};

const CASE: &str = include_str!("../../../conformance/cases/neutral-contract-seam.v1.json");
const ERROR_GOLDEN: &str = include_str!("../../../conformance/golden/error-envelope.v1.json");
const MANIFEST_GOLDEN: &str = include_str!("../../../conformance/golden/contract-manifest.v1.json");
const METADATA_GOLDEN: &str = include_str!("../../../conformance/golden/neutral-metadata.v1.json");
const SCHEMA: &str = include_str!("../../../schemas/common/neutral-contract-seam.v1.schema.json");

fn golden_payload(golden: &str) -> &str {
    golden
        .strip_suffix('\n')
        .expect("golden fixtures have exactly one terminal LF")
}

#[test]
fn neutral_metadata_round_trips_to_the_exact_golden_bytes() {
    let metadata: NeutralMetadata = decode_json(METADATA_GOLDEN).expect("metadata JSON is valid");
    metadata.validate().expect("metadata fixture is valid");
    let encoded = canonical_json(&metadata).expect("metadata serialization succeeds");
    assert_eq!(encoded, golden_payload(METADATA_GOLDEN));
    assert_eq!(
        canonical_json(&metadata).expect("second serialization succeeds"),
        encoded
    );
}

#[test]
fn error_envelope_preserves_unknown_operation_status() {
    let envelope: ErrorEnvelope = decode_json(ERROR_GOLDEN).expect("error JSON is valid");
    envelope.validate().expect("error fixture is valid");
    let encoded = canonical_json(&envelope).expect("error serialization succeeds");
    assert_eq!(encoded, golden_payload(ERROR_GOLDEN));
    assert!(encoded.contains("\"operation\":\"unknown\""));
}

#[test]
fn manifest_round_trips_and_requires_named_sorted_consumers() {
    let mut manifest: ContractManifest =
        decode_json(MANIFEST_GOLDEN).expect("manifest JSON is valid");
    manifest.validate().expect("manifest fixture is valid");
    assert_eq!(manifest.consumers.len(), 4);
    assert_eq!(
        canonical_json(&manifest).expect("manifest serialization succeeds"),
        golden_payload(MANIFEST_GOLDEN)
    );

    manifest.consumers.reverse();
    assert_eq!(manifest.validate(), Err(ValidationError::UnsortedConsumers));
}

#[test]
fn schema_and_case_are_implementation_neutral() {
    let schema: Value = decode_json(SCHEMA).expect("schema JSON is valid");
    let case: Value = decode_json(CASE).expect("case JSON is valid");
    assert_eq!(schema["$id"], "sts2-neutral-contract-seam-v1");
    assert_eq!(schema["oneOf"].as_array().map(Vec::len), Some(3));
    assert!(schema["$defs"]["neutral_metadata"]["properties"]["deadline"]["anyOf"].is_array());
    assert_eq!(case["case_id"], "CT-PROTO-NEUTRAL-001");
    assert_eq!(case["setup"]["live_runtime"], false);
    assert_eq!(case["setup"]["network"], false);
    assert_eq!(case["setup"]["proprietary_data"], false);
}

#[test]
fn malformed_digest_and_safe_message_fail_closed() {
    let mut manifest: ContractManifest =
        decode_json(MANIFEST_GOLDEN).expect("manifest JSON is valid");
    manifest.digest.value.replace_range(..1, "A");
    assert_eq!(
        manifest.validate(),
        Err(ValidationError::InvalidDigest { field: "digest" })
    );

    let mut envelope: ErrorEnvelope = decode_json(ERROR_GOLDEN).expect("error JSON is valid");
    envelope.error.safe_message.push('\n');
    assert_eq!(
        envelope.validate(),
        Err(ValidationError::InvalidCharacters {
            field: "safe_message"
        })
    );
}
