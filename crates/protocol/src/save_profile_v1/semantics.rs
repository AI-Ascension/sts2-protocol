// SPDX-License-Identifier: MIT

use super::{Rejection, intent_digest};
use serde_json::Value;

pub(super) fn validate(value: &Value) -> Result<(), Rejection> {
    if value.get("intent").is_some() {
        validate_intent(&value["intent"])?;
        require(value["intent_digest"] == intent_digest(&value["intent"])?)?;
    }
    match value["kind"].as_str() {
        Some("discover_response") => observation(&value["observation"]),
        Some("operation_result") => result(value),
        _ => Ok(()),
    }
}

fn require(valid: bool) -> Result<(), Rejection> {
    if valid {
        Ok(())
    } else {
        Err(Rejection::InvalidBinding)
    }
}

fn validate_intent(intent: &Value) -> Result<(), Rejection> {
    match intent["action"].as_str() {
        Some("select") => require(
            intent["scope"] == "select_profile"
                && intent["baseline"].is_string()
                && intent["prior_allocation_id"].is_null(),
        ),
        Some("create_disposable") => require(
            intent["scope"] == "create_disposable_profile"
                && intent["baseline"].is_null()
                && intent["prior_allocation_id"].is_string()
                && intent["prior_allocation_id"] != intent["owner"]["allocation_id"],
        ),
        _ => Err(Rejection::InvalidBinding),
    }
}

fn observation(value: &Value) -> Result<(), Rejection> {
    let slots = value["slots"].as_array().ok_or(Rejection::Malformed)?;
    let selected = &value["selected_slot_id"];
    if value["native_ready"] == true {
        require(value["baseline"].is_string() && slots.contains(selected))
    } else {
        require(selected.is_null() && value["baseline"].is_null() && value["active_run"] == false)
    }
}

fn result(value: &Value) -> Result<(), Rejection> {
    if value["status"] == "settled" {
        return settlement(value);
    }
    require(value["observation"].is_null() && value["witness"].is_null())?;
    // Preparation is never native readiness, nor evidence of completion.
    require(value["allocation_stage"] != "native_ready")?;
    match value["status"].as_str() {
        Some("accepted") => require(value["reason"].is_null()),
        Some("cancelled") => require(value["reason"] == "not_applied"),
        Some("unknown") => require(value["reason"] == "uncertain"),
        Some("rejected" | "blocked") => require(
            value["reason"].is_string()
                && value["reason"] != "not_applied"
                && value["reason"] != "uncertain",
        ),
        _ => Err(Rejection::InvalidBinding),
    }
}

fn settlement(value: &Value) -> Result<(), Rejection> {
    let intent = &value["intent"];
    let readback = &value["observation"];
    let witness = &value["witness"];
    require(readback.is_object() && witness.is_object())?;
    observation(readback)?;
    require(
        value["allocation_stage"] == "native_ready"
            && value["reason"].is_null()
            && readback["native_ready"] == true
            && readback["active_run"] == false
            && readback["owner"] == intent["owner"]
            && readback["selected_slot_id"] == intent["slot_id"]
            && readback["generation"].as_u64() > intent["expected_generation"].as_u64()
            && witness["operation_id"] == intent["operation_id"]
            && witness["intent_digest"] == value["intent_digest"]
            && witness["slot_id"] == intent["slot_id"]
            && witness["allocation_id"] == intent["owner"]["allocation_id"]
            && witness["generation"] == readback["generation"],
    )
}
