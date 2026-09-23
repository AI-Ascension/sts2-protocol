// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{RuntimeV3GameplayMessage, RuntimeV3GameplayValidationError, canonical_json};

const DISPATCH: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/dispatch-action-request.json");
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::new(&schema).expect("schema compiles as Draft 2020-12")
}

/// Builds a dispatch request whose typed action payload is exactly `payload`.
fn dispatch(payload: Value) -> Value {
    let mut value: Value = serde_json::from_str(DISPATCH).expect("golden is JSON");
    value["action"]["action"] = payload;
    value
}

fn rust_accepts(value: &Value) -> bool {
    serde_json::from_value::<RuntimeV3GameplayMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

#[test]
fn continue_run_parser_agrees_with_the_schema() {
    let validator = validator();
    for (payload, expected) in [
        (json!({"kind": "continue_run"}), true),
        (json!({"kind": "continue_run", "run_id": "profile1"}), true),
        (json!({"kind": "continue_run", "unexpected": true}), false),
        (
            json!({"kind": "continue_run", "save_path": "profile1/saves/current_run.save"}),
            false,
        ),
        (json!({"kind": "continue_run", "run_id": null}), false),
        (json!({"kind": "continue_run", "run_id": ""}), false),
        (json!({"kind": "continue_run", "run_id": "   "}), false),
        (
            json!({"kind": "continue_run", "run_id": "profile 1"}),
            false,
        ),
        (json!({"kind": "continue_run", "run_id": 42}), false),
        (
            json!({"kind": "continue_run", "run_id": "profile1", "seed": "1"}),
            false,
        ),
    ] {
        let value = dispatch(payload.clone());
        assert_eq!(validator.is_valid(&value), expected, "schema {payload}");
        assert_eq!(rust_accepts(&value), expected, "rust {payload}");
    }
}

#[test]
fn continue_run_rejects_a_non_identity_run_id_at_validation() {
    // The payload parses as a string and fails the identity rule during validation, not merely
    // because serde is strict.
    let message: RuntimeV3GameplayMessage = serde_json::from_value(dispatch(
        json!({"kind": "continue_run", "run_id": "profile 1"}),
    ))
    .expect("run_id is a string at the wire layer");
    assert_eq!(
        message.validate(),
        Err(RuntimeV3GameplayValidationError::ActionShape)
    );
}

#[test]
fn continue_run_round_trips_without_inventing_a_run_identity() {
    for payload in [
        json!({"kind": "continue_run"}),
        json!({"kind": "continue_run", "run_id": "profile1"}),
    ] {
        let message: RuntimeV3GameplayMessage =
            serde_json::from_value(dispatch(payload.clone())).expect("continue_run parses");
        message.validate().expect("continue_run validates");
        let encoded = canonical_json(&message).expect("encoding succeeds");
        let reparsed: Value = serde_json::from_str(&encoded).expect("canonical JSON is JSON");
        assert_eq!(reparsed["action"]["action"], payload);
    }
}
