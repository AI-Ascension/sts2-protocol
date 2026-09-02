// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_ACTION_ID, RUNTIME_ARTIFACT, RUNTIME_PROTOCOL_VERSION, RUNTIME_SCHEMA_DIGEST,
    RUNTIME_SCHEMA_SOURCE,
};

const CASE: &str = include_str!("../../../conformance/cases/runtime-v1.json");
const MANIFEST: &str = include_str!("../../../artifacts/runtime-v1/manifest.json");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-v1/schema.json");
const STATE_REQUEST: &str = include_str!("../../../artifacts/runtime-v1/golden/state-request.json");
const STATE_RESPONSE: &str =
    include_str!("../../../artifacts/runtime-v1/golden/state-response.json");
const ACTION_REQUEST: &str =
    include_str!("../../../artifacts/runtime-v1/golden/action-request.json");
const ACCEPTED: &str = include_str!("../../../artifacts/runtime-v1/golden/action-accepted.json");
const REJECTED: &str = include_str!("../../../artifacts/runtime-v1/golden/action-rejected.json");

#[test]
fn runtime_goldens_validate_against_the_identical_release_like_schema() {
    let source: Value = serde_json::from_str(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = serde_json::from_str(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-runtime-v1");
    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("runtime schema compiles");
    for fixture in [
        STATE_REQUEST,
        STATE_RESPONSE,
        ACTION_REQUEST,
        ACCEPTED,
        REJECTED,
    ] {
        let value: Value = serde_json::from_str(fixture).expect("runtime fixture is JSON");
        assert!(validator.is_valid(&value), "runtime fixture must validate");
    }
    let mut unknown: Value = serde_json::from_str(STATE_RESPONSE).expect("state is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));
}

#[test]
fn runtime_case_and_manifest_bind_the_fixed_action_and_schema_digest() {
    let case: Value = serde_json::from_str(CASE).expect("runtime case is JSON");
    let manifest: Value = serde_json::from_str(MANIFEST).expect("runtime manifest is JSON");
    assert_eq!(case["profile"], RUNTIME_PROTOCOL_VERSION);
    assert_eq!(case["schema"], RUNTIME_SCHEMA_SOURCE);
    assert_eq!(case["consumers"].as_array().map(Vec::len), Some(4));
    assert_eq!(manifest["artifact"], RUNTIME_ARTIFACT);
    assert_eq!(manifest["protocol_version"], RUNTIME_PROTOCOL_VERSION);
    assert_eq!(manifest["schema_digest"], RUNTIME_SCHEMA_DIGEST);
    assert_eq!(manifest["provenance"]["source"], RUNTIME_SCHEMA_SOURCE);
    assert_eq!(manifest["consumers"].as_array().map(Vec::len), Some(4));
    let action: Value = serde_json::from_str(ACTION_REQUEST).expect("action is JSON");
    assert_eq!(action["action"]["action_id"], RUNTIME_ACTION_ID);
}
