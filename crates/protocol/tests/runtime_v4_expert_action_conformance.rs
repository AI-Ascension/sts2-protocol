// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_SCHEMA_DIGEST, decode_json,
};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert-action.schema.json");
const EXPERT_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v4-expert-action.json");
const REQUEST: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/golden/action-request.json");
const SETTLED: &str =
    include_str!("../../../artifacts/runtime-v4-expert-action/golden/action-settled.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v4-expert-action/SHA256SUMS");

fn settlement_observation_matches_outer_identity(value: &Value) -> bool {
    value["observation"]["state_id"] == value["state_id"]
        && value["observation"]["generation"] == value["generation"]
}

#[test]
fn action_artifact_is_closed_and_goldens_match_schema() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let schema: Value = decode_json(SOURCE_SCHEMA)?;
    let validator = jsonschema::draft202012::options().build(&schema)?;
    let request: Value = decode_json(REQUEST)?;
    let settled: Value = decode_json(SETTLED)?;
    assert!(validator.is_valid(&request));
    assert!(validator.is_valid(&settled));
    let expert_schema: Value = decode_json(EXPERT_SCHEMA)?;
    let expert_validator = jsonschema::draft202012::options().build(&expert_schema)?;
    assert!(expert_validator.is_valid(&settled["observation"]));
    assert_eq!(request["kind"], "action_request");
    assert_eq!(settled["kind"], "action_response");
    assert_eq!(settled["status"], "settled");
    assert!(settlement_observation_matches_outer_identity(&settled));
    assert_eq!(settled["observation"]["state_id"], "live:8");
    assert_eq!(settled["observation"]["generation"], 8);
    assert_eq!(settled["observation"]["player"]["potions"], json!([]));
    assert_eq!(
        settled["observation"]["legal_actions"],
        json!([
            {
                "action_id": "play:8:card:1:enemy:1",
                "action": {
                    "kind": "play_card",
                    "card_id": "card:1",
                    "target_id": "enemy:1"
                }
            },
            { "action_id": "end:8", "action": { "kind": "end_turn" } }
        ])
    );
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
fn settled_observation_rejects_cross_identity_even_when_both_schemas_accept()
-> Result<(), Box<dyn std::error::Error>> {
    let action_schema: Value = decode_json(SOURCE_SCHEMA)?;
    let action_validator = jsonschema::draft202012::options().build(&action_schema)?;
    let expert_schema: Value = decode_json(EXPERT_SCHEMA)?;
    let expert_validator = jsonschema::draft202012::options().build(&expert_schema)?;
    let settled: Value = decode_json(SETTLED)?;
    let mut foreign = settled.clone();
    foreign["observation"]["state_id"] = "foreign:8".into();
    assert!(action_validator.is_valid(&foreign));
    assert!(expert_validator.is_valid(&foreign["observation"]));
    assert!(!settlement_observation_matches_outer_identity(&foreign));
    foreign["observation"]["state_id"] = "live:8".into();
    foreign["observation"]["generation"] = 9.into();
    assert!(action_validator.is_valid(&foreign));
    assert!(expert_validator.is_valid(&foreign["observation"]));
    assert!(!settlement_observation_matches_outer_identity(&foreign));
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
