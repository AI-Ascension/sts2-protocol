// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{RuntimeV3GameplayMessage, decode_json};

const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");
const REQUEST: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/action-request.json");
const SETTLED: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/action-settled.json");
const STATE: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-request.json");

fn typed_valid(value: &Value) -> bool {
    decode_json::<RuntimeV3GameplayMessage>(&value.to_string())
        .is_ok_and(|message| message.validate().is_ok())
}

#[test]
fn required_nullable_envelope_fields_cannot_be_omitted() {
    let schema: Value = decode_json(SCHEMA).unwrap();
    let validator = jsonschema::draft202012::options().build(&schema).unwrap();
    let original: Value = decode_json(STATE).unwrap();
    assert!(validator.is_valid(&original));
    assert!(typed_valid(&original));
    for field in [
        "operation_id",
        "observation",
        "action",
        "status",
        "error_code",
        "effect_witness",
    ] {
        let mut missing = original.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(!validator.is_valid(&missing), "schema requires {field}");
        assert!(!typed_valid(&missing), "Rust must require {field}");
    }
}

#[test]
fn nullable_targets_must_be_present_even_when_null() {
    let schema: Value = decode_json(SCHEMA).unwrap();
    let validator = jsonschema::draft202012::options().build(&schema).unwrap();
    for (text, field) in [(REQUEST, "action"), (SETTLED, "effect_witness")] {
        let mut value: Value = decode_json(text).unwrap();
        value["action"]["target_id"] = Value::Null;
        if !value["effect_witness"].is_null() {
            value["effect_witness"]["target_id"] = Value::Null;
        }
        assert!(validator.is_valid(&value));
        assert!(typed_valid(&value));
        value[field].as_object_mut().unwrap().remove("target_id");
        assert!(!validator.is_valid(&value));
        assert!(!typed_valid(&value));
    }
}

#[test]
fn settled_receipt_witness_must_match_the_recorded_action() {
    for kind in ["action_response", "reconcile_response"] {
        let mut original: Value = decode_json(SETTLED).unwrap();
        original["kind"] = json!(kind);
        assert!(typed_valid(&original));
        for (field, replacement) in [
            ("card_index", json!(3)),
            ("target_id", json!("enemy-other")),
            ("target_id", Value::Null),
            ("generation", json!(4)),
        ] {
            let mut mismatch = original.clone();
            mismatch["effect_witness"][field] = replacement;
            assert!(
                !typed_valid(&mismatch),
                "must reject {kind} {field} mismatch"
            );
        }
    }
}

#[test]
fn safe_integer_and_unsigned_generation_bounds_remain_strict() {
    let original: Value = decode_json(REQUEST).unwrap();
    for field in ["generation", "lease_epoch"] {
        for bad in [json!(-1), json!(1.5), json!(9_007_199_254_740_992_u64)] {
            let mut value = original.clone();
            value[field] = bad;
            assert!(!typed_valid(&value), "invalid {field} must be rejected");
        }
        let mut boundary = original.clone();
        boundary[field] = json!(9_007_199_254_740_991_u64);
        assert!(typed_valid(&boundary));
    }
}
