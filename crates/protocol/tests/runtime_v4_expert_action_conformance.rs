// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_SCHEMA_DIGEST, decode_json,
};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert-action.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v4-expert-action.json");
const REQUEST: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/golden/action-request.json");
const SETTLED: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/golden/action-settled.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v4-expert-action/SHA256SUMS");

#[test]
fn action_artifact_is_closed_and_goldens_match_schema() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let schema: Value = decode_json(SOURCE_SCHEMA)?;
    let validator = jsonschema::draft202012::options().build(&schema)?;
    let request: Value = decode_json(REQUEST)?;
    let settled: Value = decode_json(SETTLED)?;
    assert!(validator.is_valid(&request));
    assert!(validator.is_valid(&settled));
    assert_eq!(request["kind"], "action_request");
    assert_eq!(settled["kind"], "action_response");
    assert_eq!(settled["status"], "settled");
    assert_eq!(settled["transition"]["removed"], true);
    assert_eq!(
        settled["transition"]["before_generation"],
        request["generation"]
    );
    assert_eq!(
        settled["transition"]["after_generation"],
        settled["generation"]
    );
    assert_eq!(
        settled["observation"]["schema_digest"],
        RUNTIME_V4_EXPERT_SCHEMA_DIGEST
    );
    assert_eq!(
        request["schema_digest"],
        RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST
    );
    assert_eq!(
        settled["schema_digest"],
        RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST
    );
    Ok(())
}

#[test]
fn action_conformance_requires_identity_and_unknown_is_reconciled_by_operation()
-> Result<(), Box<dyn std::error::Error>> {
    let case: Value = decode_json(CASE)?;
    assert_eq!(case["contract"], "sts2.protocol/runtime-v4-expert-action");
    assert_eq!(
        case["required"],
        json!([
            "authenticated_identity",
            "generation_fence",
            "operation_identity",
            "fresh_settlement_witness"
        ])
    );
    let schema: Value = decode_json(SOURCE_SCHEMA)?;
    let validator = jsonschema::draft202012::options().build(&schema)?;
    let mut malformed: Value = decode_json(REQUEST)?;
    malformed["operation_id"] = Value::Null;
    assert!(!validator.is_valid(&malformed));
    let mut unknown: Value = decode_json(SETTLED)?;
    unknown["status"] = "unknown".into();
    unknown["observation"] = Value::Null;
    unknown["transition"] = Value::Null;
    unknown["error_code"] = "transport_timeout".into();
    assert!(validator.is_valid(&unknown));
    Ok(())
}

#[test]
fn action_checksums_have_all_six_inputs() {
    for path in [
        "../../conformance/cases/runtime-v4-expert-action.json",
        "../../schemas/runtime-v4-expert-action.schema.json",
        "manifest.json",
        "schema.json",
        "golden/action-request.json",
        "golden/action-settled.json",
    ] {
        let line = CHECKSUMS
            .lines()
            .find(|line| line.ends_with(&format!("  {path}")))
            .expect("checksum inventory contains every action input");
        let digest = line.split_once("  ").expect("checksum has two columns").0;
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
