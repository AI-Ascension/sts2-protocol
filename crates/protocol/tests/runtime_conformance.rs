// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_ACTION_ID, RUNTIME_ARTIFACT, RUNTIME_MAX_ACTION_COUNT, RUNTIME_MAX_GENERATION,
    RUNTIME_PROTOCOL_VERSION, RUNTIME_SCHEMA_DIGEST, RUNTIME_SCHEMA_SOURCE, RuntimeMessage,
    RuntimeValidationError, canonical_json, decode_json,
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

const GOLDENS: &[(&str, &str)] = &[
    ("state-request", STATE_REQUEST),
    ("state-response", STATE_RESPONSE),
    ("action-request", ACTION_REQUEST),
    ("action-accepted", ACCEPTED),
    ("action-rejected", REJECTED),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn validator() -> jsonschema::Validator {
    let source: Value = serde_json::from_str(SOURCE_SCHEMA).expect("source schema is JSON");
    jsonschema::draft202012::options()
        .build(&source)
        .expect("runtime schema compiles")
}

fn typed(value: &Value) -> Result<(), RuntimeValidationError> {
    serde_json::from_value::<RuntimeMessage>(value.clone())
        .expect("mutated golden still decodes")
        .validate()
}

#[test]
fn runtime_goldens_validate_against_the_identical_release_like_schema() {
    let source: Value = serde_json::from_str(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = serde_json::from_str(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-runtime-v1");
    let validator = validator();
    for (_, fixture) in GOLDENS {
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

#[test]
fn runtime_goldens_round_trip_with_stable_bytes() {
    for (name, text) in GOLDENS {
        let message: RuntimeMessage = decode_json(text).expect("golden JSON is valid");
        message
            .validate()
            .unwrap_or_else(|error| panic!("{name} must validate: {error}"));
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            payload(text),
            "{name} canonical bytes"
        );
    }
}

#[test]
fn runtime_message_rejects_values_outside_the_documented_bounds() {
    let validator = validator();
    let above_generation = json!(RUNTIME_MAX_GENERATION + 1);
    let above_action_count = json!(RUNTIME_MAX_ACTION_COUNT + 1);
    for (pointer, replacement, expected) in [
        (
            "/generation",
            &above_generation,
            RuntimeValidationError::GenerationBounds,
        ),
        (
            "/lease_epoch",
            &above_generation,
            RuntimeValidationError::GenerationBounds,
        ),
        (
            "/observation/action_count",
            &above_action_count,
            RuntimeValidationError::ObservationBounds,
        ),
        (
            "/effect_witness/generation",
            &above_generation,
            RuntimeValidationError::EffectBounds,
        ),
    ] {
        let mut value: Value = serde_json::from_str(ACCEPTED).expect("accepted is JSON");
        *value.pointer_mut(pointer).expect("pointer exists") = replacement.clone();
        assert!(!validator.is_valid(&value), "schema rejects {pointer}");
        assert_eq!(typed(&value), Err(expected), "typed rejects {pointer}");
    }
    let mut at_bound: Value = serde_json::from_str(ACCEPTED).expect("accepted is JSON");
    at_bound["generation"] = json!(RUNTIME_MAX_GENERATION);
    at_bound["lease_epoch"] = json!(RUNTIME_MAX_GENERATION);
    at_bound["effect_witness"]["generation"] = json!(RUNTIME_MAX_GENERATION);
    at_bound["observation"]["action_count"] = json!(RUNTIME_MAX_ACTION_COUNT);
    assert!(validator.is_valid(&at_bound));
    assert_eq!(typed(&at_bound), Ok(()));
}

#[test]
fn runtime_message_rejects_metadata_provenance_and_identity_drift() {
    let validator = validator();
    let zero_digest = "0".repeat(64);
    for (pointer, replacement, expected) in [
        (
            "/protocol_version",
            "runtime-v2",
            RuntimeValidationError::Metadata,
        ),
        (
            "/schema_digest",
            zero_digest.as_str(),
            RuntimeValidationError::Metadata,
        ),
        (
            "/provenance/artifact",
            "sts2-protocol/runtime-v2",
            RuntimeValidationError::Provenance,
        ),
        (
            "/correlation_id",
            "corr 0003",
            RuntimeValidationError::Identity,
        ),
        ("/lease_id", "", RuntimeValidationError::Identity),
        (
            "/error_code",
            "stale generation",
            RuntimeValidationError::Identity,
        ),
        (
            "/action/action_id",
            "end_turn",
            RuntimeValidationError::ActionBounds,
        ),
    ] {
        let mut value: Value = serde_json::from_str(REJECTED).expect("rejected is JSON");
        *value.pointer_mut(pointer).expect("pointer exists") = json!(replacement);
        assert_eq!(typed(&value), Err(expected), "typed rejects {pointer}");
        let schema_agrees = pointer != "/schema_digest";
        assert_eq!(
            !validator.is_valid(&value),
            schema_agrees,
            "schema at {pointer}"
        );
    }
}

#[test]
fn runtime_message_rejects_members_that_contradict_the_kind_or_status() {
    let validator = validator();
    let observation = json!({
        "host_ready": true,
        "overlay_visible": false,
        "screen": "host",
        "action_count": 0
    });
    for (name, fixture, pointer, replacement) in [
        (
            "state-request",
            STATE_REQUEST,
            "/observation",
            observation.clone(),
        ),
        (
            "state-response",
            STATE_RESPONSE,
            "/observation",
            Value::Null,
        ),
        ("action-request", ACTION_REQUEST, "/action", Value::Null),
        (
            "action-request",
            ACTION_REQUEST,
            "/status",
            json!("accepted"),
        ),
        ("action-accepted", ACCEPTED, "/effect_witness", Value::Null),
        ("action-accepted", ACCEPTED, "/error_code", json!("late")),
        ("action-rejected", REJECTED, "/error_code", Value::Null),
        ("action-rejected", REJECTED, "/status", Value::Null),
    ] {
        let mut value: Value = serde_json::from_str(fixture).expect("golden is JSON");
        *value.pointer_mut(pointer).expect("pointer exists") = replacement;
        assert!(
            !validator.is_valid(&value),
            "schema rejects {name}{pointer}"
        );
        assert_eq!(
            typed(&value),
            Err(RuntimeValidationError::ResultShape),
            "typed rejects {name}{pointer}"
        );
    }
}
