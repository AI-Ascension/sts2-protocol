// SPDX-License-Identifier: MIT

use super::*;

fn completion(option: &str, evidence: Value) -> Value {
    let mut value = golden(if option == "mend" {
        "action-mend-selection-completed.json"
    } else {
        "action-completed.json"
    });
    value["action"]["action"]["rest_option_id"] = option.into();
    value["transition"]["rest_option_id"] = option.into();
    value["effect_witness"]["rest_option_id"] = option.into();
    value["effect_witness"]["kind"] = format!("{option}_applied").into();
    value["effect_witness"]["evidence"] = evidence;
    value["transition"]["effect_witness"] = value["effect_witness"].clone();
    value
}

fn native_completion(option: &str) -> Value {
    let state = if option == "mend" {
        "live:22"
    } else {
        "live:8"
    };
    completion(
        option,
        serde_json::json!({
            "kind": "native_completion",
            "completion_id": "completion:rest:1",
            "native_state_id": state,
        }),
    )
}

fn conforms(value: &Value) -> bool {
    let mut requests = request_actions();
    requests.insert(
        value["operation_id"].as_str().unwrap().to_owned(),
        value["action"].clone(),
    );
    schema_validator(SOURCE_SCHEMA).is_valid(value)
        && nested_observation_is_valid(value, &schema_validator(EXPERT_SCHEMA))
        && strict_semantics(value, &requests, &selector_admissions()) == Some(true)
}

#[test]
fn native_completion_alternatives_conform_for_mend_and_lift() {
    for option in ["mend", "lift"] {
        assert!(conforms(&native_completion(option)), "{option}");
    }
}

#[test]
fn native_completion_rejects_stale_or_missing_identity_and_wrong_option() {
    for option in ["mend", "lift"] {
        let valid = native_completion(option);
        for field in ["completion_id", "native_state_id"] {
            let mut invalid = valid.clone();
            invalid["effect_witness"]["evidence"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            invalid["transition"]["effect_witness"] = invalid["effect_witness"].clone();
            assert!(!conforms(&invalid), "{option}: missing {field}");
        }
        let mut stale = valid;
        stale["effect_witness"]["evidence"]["native_state_id"] = "live:stale".into();
        stale["transition"]["effect_witness"] = stale["effect_witness"].clone();
        assert!(!conforms(&stale), "{option}: stale state");
    }
    assert!(!conforms(&native_completion("heal")));
}

#[test]
fn numeric_evidence_still_requires_an_actual_effect() {
    assert!(conforms(&golden("action-mend-selection-completed.json")));
    for (before, after, expected) in [(0, 1, true), (1, 1, false)] {
        let value = completion(
            "lift",
            serde_json::json!({
                "kind": "stat_change", "stat_id": "strength", "before": before, "after": after,
            }),
        );
        assert_eq!(conforms(&value), expected, "lift {before} to {after}");
    }
}
