// SPDX-License-Identifier: MIT

use super::support::{self, TestResult};
use serde_json::Value as JsonValue;
use sts2_protocol::game_facts_reference_v1::{
    Availability, EvidenceStatus, GameFactsReferenceV1Codec, GameFactsReferenceV1Rejection,
    Message, MessageKind, QueryResultEntry, Unit, validate_response_against_request,
};

fn decode(name: &str) -> Result<Message, String> {
    GameFactsReferenceV1Codec::decode(&support::golden(name)?).map_err(|error| error.to_string())
}

#[test]
fn closed_envelopes_require_every_nullable_member_explicitly() -> TestResult {
    let samples = [
        "query-static-request.json",
        "response-found-static.json",
        "capabilities-static-only.json",
        "error-missing-capability.json",
    ];
    for name in samples {
        let bytes = support::golden(name)?;
        let mut value: JsonValue =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        for field in ["query", "result", "capabilities", "error"] {
            let mut missing = value.clone();
            missing
                .as_object_mut()
                .ok_or("envelope object missing")?
                .remove(field);
            let bytes = serde_json::to_vec(&missing).map_err(|error| error.to_string())?;
            assert!(
                GameFactsReferenceV1Codec::decode(&bytes).is_err(),
                "{name}: {field}"
            );
        }
        value["extra"] = serde_json::json!(true);
        let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
        assert!(GameFactsReferenceV1Codec::decode(&bytes).is_err());
    }
    Ok(())
}

#[test]
fn duplicate_members_and_bounds_are_rejected_before_typed_acceptance() -> TestResult {
    let bytes = support::golden("query-static-request.json")?;
    let text = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
    let duplicate = text.replacen(
        "\"kind\": \"query_request\"",
        "\"kind\": \"query_request\", \"kind\": \"query_request\"",
        1,
    );
    assert!(GameFactsReferenceV1Codec::decode(duplicate.as_bytes()).is_err());
    let unknown_kind = text.replace("query_request", "unregistered_kind");
    assert_eq!(
        GameFactsReferenceV1Codec::decode(unknown_kind.as_bytes()),
        Err(GameFactsReferenceV1Rejection::UnknownKind)
    );
    let wrong_version = text.replace("game-facts-reference-v1", "game-facts-reference-v0");
    assert_eq!(
        GameFactsReferenceV1Codec::decode(wrong_version.as_bytes()),
        Err(GameFactsReferenceV1Rejection::UnsupportedVersion)
    );
    let mut oversized = Vec::new();
    oversized.resize(262_145, b' ');
    assert_eq!(
        GameFactsReferenceV1Codec::decode(&oversized),
        Err(GameFactsReferenceV1Rejection::ResultLimitExceeded)
    );
    Ok(())
}

#[test]
fn integer_lexemes_are_exact_and_multiplier_parts_remain_independent() -> TestResult {
    let bytes = support::golden("response-found-live-synthetic.json")?;
    let text = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
    let integral_decimal = text.replace("\"value\": 12", "\"value\": 12.0");
    let parsed = GameFactsReferenceV1Codec::decode(integral_decimal.as_bytes())
        .map_err(|error| error.to_string())?;
    let original = GameFactsReferenceV1Codec::decode(&bytes).map_err(|error| error.to_string())?;
    assert_eq!(parsed, original);
    for invalid in ["12.5", "9007199254740992"] {
        let raw = text.replace("\"value\": 12", &format!("\"value\": {invalid}"));
        assert!(
            GameFactsReferenceV1Codec::decode(raw.as_bytes()).is_err(),
            "{invalid}"
        );
    }
    let response = decode("response-found-live-synthetic.json")?;
    let result = response.result.as_ref().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &result.results[0] else {
        return Err("found entry missing".into());
    };
    assert_eq!(inputs[1].name, "multiplier_numerator");
    assert_eq!(inputs[1].unit, Unit::Multiplier);
    assert_eq!(inputs[2].name, "multiplier_denominator");
    assert_eq!(inputs[2].unit, Unit::Multiplier);
    Ok(())
}

#[test]
fn caller_join_checks_echo_and_live_references_keep_owner_epochs_separate() -> TestResult {
    let request = decode("query-static-request.json")?;
    let response = decode("response-found-static.json")?;
    validate_response_against_request(&request, &response).map_err(|error| error.to_string())?;

    let mut changed = response.clone();
    changed.correlation_id.push_str("-other");
    assert_eq!(
        validate_response_against_request(&request, &changed),
        Err(GameFactsReferenceV1Rejection::InvalidBinding)
    );
    let mut changed = response.clone();
    changed
        .query
        .as_mut()
        .ok_or("query missing")?
        .binding
        .content_manifest_id = "other-content".into();
    assert_eq!(
        validate_response_against_request(&request, &changed),
        Err(GameFactsReferenceV1Rejection::InvalidBinding)
    );

    let mut unknown_request = decode("query-static-request.json")?;
    unknown_request.correlation_id = "unknown-id-1".into();
    unknown_request
        .query
        .as_mut()
        .ok_or("query missing")?
        .rule_ids[0] = "fixture.unknown_rule".into();
    let unknown_response = decode("error-unknown-id.json")?;
    validate_response_against_request(&unknown_request, &unknown_response)
        .map_err(|error| error.to_string())?;

    let mut live = decode("query-live-request.json")?;
    GameFactsReferenceV1Codec::validate(&live).map_err(|error| error.to_string())?;
    let query = live.query.as_ref().ok_or("live query missing")?;
    let instance = query
        .binding
        .instance_ref
        .as_ref()
        .ok_or("instance ref missing")?;
    let scope = query.scope.as_ref().ok_or("scope missing")?;
    assert_ne!(instance.epoch, scope.authority_epoch);
    live.query
        .as_mut()
        .ok_or("live query missing")?
        .parent_observation
        .as_mut()
        .ok_or("parent missing")?
        .state_generation += 1;
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());
    Ok(())
}

#[test]
fn semantic_bounds_units_availability_and_source_tokens_are_enforced() -> TestResult {
    let mut live = decode("response-found-live-synthetic.json")?;
    let result = live.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[1].unit = Unit::Boolean;
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());

    let mut live = decode("response-found-live-synthetic.json")?;
    let result = live.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[0].source_ref.r#ref = Some("C:/private".into());
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());

    let mut live = decode("response-found-live-synthetic.json")?;
    let result = live.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[0].source_ref.r#ref = Some("file-backup".into());
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());

    let mut live = decode("response-found-live-synthetic.json")?;
    let result = live.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[0].source_ref.r#ref = Some("NullReferenceException".into());
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());

    let mut live = decode("response-found-live-synthetic.json")?;
    let result = live.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[0]
        .observation
        .as_mut()
        .ok_or("observation missing")?
        .availability = Availability::Missing;
    assert!(GameFactsReferenceV1Codec::validate(&live).is_err());

    let mut static_response = decode("response-found-static.json")?;
    let result = static_response.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found {
        evidence_status, ..
    } = &mut result.results[0]
    else {
        return Err("found entry missing".into());
    };
    *evidence_status = EvidenceStatus::Confirmed;
    assert!(GameFactsReferenceV1Codec::validate(&static_response).is_err());
    Ok(())
}

#[test]
fn duplicate_rule_ids_input_names_and_result_order_are_rejected() -> TestResult {
    let mut request = decode("query-static-request.json")?;
    request
        .query
        .as_mut()
        .ok_or("query missing")?
        .rule_ids
        .push("resource.fixed_card_cost".into());
    assert!(GameFactsReferenceV1Codec::validate(&request).is_err());

    let mut response = decode("response-found-live-synthetic.json")?;
    let result = response.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { inputs, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    inputs[1].name = inputs[0].name.clone();
    assert!(GameFactsReferenceV1Codec::validate(&response).is_err());

    let mut response = decode("response-found-static.json")?;
    let result = response.result.as_mut().ok_or("result missing")?;
    let QueryResultEntry::Found { rule_id, .. } = &mut result.results[0] else {
        return Err("found entry missing".into());
    };
    *rule_id = "other.rule".into();
    assert!(GameFactsReferenceV1Codec::validate(&response).is_err());
    Ok(())
}

#[test]
fn typed_reason_enforces_utf8_bytes_beyond_schema_character_bound() -> TestResult {
    let mut error = decode("error-missing-capability.json")?;
    error.error.as_mut().ok_or("error missing")?.reason = Some("é".repeat(129));
    assert_eq!(
        GameFactsReferenceV1Codec::validate(&error),
        Err(GameFactsReferenceV1Rejection::InvalidBounds)
    );
    let mut bounded = decode("error-missing-capability.json")?;
    bounded.error.as_mut().ok_or("error missing")?.reason = Some("é".repeat(128));
    GameFactsReferenceV1Codec::validate(&bounded).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn error_and_capability_profiles_have_closed_fixed_semantics() -> TestResult {
    let live = decode("capabilities-live-synthetic.json")?;
    assert_eq!(live.kind, MessageKind::CapabilitiesResponse);
    assert!(
        live.capabilities
            .as_ref()
            .ok_or("capabilities missing")?
            .snapshot_policy
            .is_some()
    );
    let static_only = decode("capabilities-static-only.json")?;
    assert!(
        static_only
            .capabilities
            .as_ref()
            .ok_or("capabilities missing")?
            .snapshot_policy
            .is_none()
    );
    Ok(())
}
