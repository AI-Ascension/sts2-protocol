// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::RuntimeV3GameplayMessage;

const RESPONSE: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json");
const REQUEST: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-request.json");
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn rust_accepts(value: &Value) -> bool {
    serde_json::from_value::<RuntimeV3GameplayMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

#[test]
fn nested_enum_objects_are_closed() {
    for pointer in ["/observation/state", "/legal_actions/0/action"] {
        let mut value: Value = serde_json::from_str(RESPONSE).unwrap();
        value["legal_actions"] = json!([{"action_id":"end-turn","action":{"kind":"end_turn"}}]);
        value.pointer_mut(pointer).unwrap()["privileged_rng"] = json!(123);
        assert!(!rust_accepts(&value), "{pointer}");
    }
}

#[test]
fn required_nullable_members_cannot_be_omitted() {
    let request: Value = serde_json::from_str(REQUEST).unwrap();
    for key in request.as_object().unwrap().keys() {
        let mut value = request.clone();
        value.as_object_mut().unwrap().remove(key);
        assert!(!rust_accepts(&value), "missing {key}");
    }
}

#[test]
fn schema_rejects_result_payload_on_state_request() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    let mut value: Value = serde_json::from_str(REQUEST).unwrap();
    value["status"] = json!("settled");
    assert!(!validator.is_valid(&value));
}

#[test]
fn visible_text_accepts_player_names_and_rejects_controls() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    for (name, expected) in [
        ("Fury", true),
        ("\n", false),
        ("\u{85}", false),
        ("é", true),
    ] {
        let mut value: Value = serde_json::from_str(RESPONSE).unwrap();
        value["observation"]["visible_seed"] = json!(name);
        assert_eq!(validator.is_valid(&value), expected, "schema {name:?}");
        assert_eq!(rust_accepts(&value), expected, "rust {name:?}");
    }
}

#[test]
fn every_nested_enum_variant_rejects_unknown_fields() {
    for value in [
        json!({"kind":"end_turn","unexpected":true}),
        json!({"kind":"play_card","card_id":"card-1","target_id":null,"unexpected":true}),
    ] {
        assert!(serde_json::from_value::<sts2_protocol::RuntimeV3GameplayAction>(value).is_err());
    }
    for value in [
        json!({"kind":"unknown","extra":1}),
        json!({"kind":"attack","damage":3,"hits":1,"extra":1}),
    ] {
        assert!(
            serde_json::from_value::<sts2_protocol::RuntimeV3GameplayEnemyIntent>(value).is_err()
        );
    }
    assert!(
        serde_json::from_value::<sts2_protocol::RuntimeV3GameplayState>(
            json!({"state":"victory","extra":1})
        )
        .is_err()
    );
}

#[test]
fn schema_and_rust_agree_on_all_kind_payload_shapes() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    let request: Value = serde_json::from_str(REQUEST).unwrap();
    let response: Value = serde_json::from_str(RESPONSE).unwrap();
    let settled: Value = serde_json::from_str(include_str!(
        "../../../artifacts/runtime-v3-gameplay/golden/dispatch-action-settled.json"
    ))
    .unwrap();
    let mut valid = Vec::new();
    for kind in ["state_request", "reobserve_request"] {
        let mut value = request.clone();
        value["kind"] = json!(kind);
        valid.push(value);
    }
    for kind in ["state_response", "reobserve_response"] {
        let mut value = response.clone();
        value["kind"] = json!(kind);
        valid.push(value);
    }
    let mut legal = request.clone();
    legal["kind"] = json!("legal_actions_request");
    legal["state_id"] = json!("combat-1");
    valid.push(legal.clone());
    legal["kind"] = json!("legal_actions_response");
    legal["legal_actions"] = json!([]);
    valid.push(legal);
    valid.push(
        serde_json::from_str(include_str!(
            "../../../artifacts/runtime-v3-gameplay/golden/dispatch-action-request.json"
        ))
        .unwrap(),
    );
    let mut wait = request.clone();
    wait["kind"] = json!("wait_request");
    wait["operation_id"] = json!("op-1");
    wait["wait_for_millis"] = json!(1);
    valid.push(wait);
    for kind in ["reobserve", "reconcile", "release_lease", "stop_episode"] {
        let mut recover = request.clone();
        recover["kind"] = json!("recover_request");
        recover["recovery"] = json!({"kind":kind,"operation_id":if kind == "reconcile" {json!("op-1")} else {Value::Null}});
        valid.push(recover);
    }
    for kind in [
        "dispatch_action_response",
        "recover_response",
        "wait_response",
    ] {
        for status in ["accepted", "settled", "rejected", "cancelled", "unknown"] {
            if kind == "wait_response" && !["settled", "unknown"].contains(&status) {
                continue;
            }
            let mut value = settled.clone();
            value["kind"] = json!(kind);
            value["status"] = json!(status);
            if status != "settled" {
                value["transition"] = Value::Null;
            }
            if ["rejected", "cancelled", "unknown"].contains(&status) {
                value["error_code"] = json!("error");
            }
            if status == "unknown" {
                value["observation"] = Value::Null;
                value["legal_actions"] = Value::Null;
            }
            if kind == "wait_response" {
                value["wait_outcome"] = json!(if status == "settled" {
                    "successor"
                } else {
                    "recovery_required"
                });
            }
            valid.push(value);
        }
    }
    for value in valid {
        assert!(
            rust_accepts(&value),
            "valid {} {}",
            value["kind"],
            value["status"]
        );
        assert!(
            validator.is_valid(&value),
            "schema {} {}",
            value["kind"],
            value["status"]
        );
        for field in [
            "state_id",
            "operation_id",
            "observation",
            "legal_actions",
            "action",
            "status",
            "transition",
            "error_code",
            "wait_for_millis",
            "wait_outcome",
            "recovery",
        ] {
            let mut changed = value.clone();
            changed[field] = if value[field].is_null() {
                json!("invalid")
            } else {
                Value::Null
            };
            assert_eq!(
                validator.is_valid(&changed),
                rust_accepts(&changed),
                "{} {} field {field}",
                value["kind"],
                value["status"]
            );
        }
    }
}
