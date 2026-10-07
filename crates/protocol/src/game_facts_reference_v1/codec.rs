// SPDX-License-Identifier: MIT

use super::types::Message;
use super::validation;
use super::{
    GAME_FACTS_REFERENCE_V1_MAX_MESSAGE_BYTES, GAME_FACTS_REFERENCE_V1_PROFILE,
    GAME_FACTS_REFERENCE_V1_SCHEMA, GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST,
};
use crate::game_facts_reference_v1::GameFactsReferenceV1Rejection as Reject;
use serde_json::Value;

pub(super) fn decode(bytes: &[u8]) -> Result<Message, Reject> {
    bound(bytes.len())?;
    let value = crate::game_information_v2::decode_unique_exact_json(bytes)
        .map_err(|_| Reject::Malformed)?;
    let version = value
        .get("protocol_version")
        .and_then(Value::as_str)
        .ok_or(Reject::Malformed)?;
    let digest = value
        .get("schema_digest")
        .and_then(Value::as_str)
        .ok_or(Reject::Malformed)?;
    if version != GAME_FACTS_REFERENCE_V1_PROFILE || digest != GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST
    {
        return Err(Reject::UnsupportedVersion);
    }
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or(Reject::Malformed)?;
    if !matches!(
        kind,
        "query_request" | "query_response" | "capabilities_response" | "error_response"
    ) {
        return Err(Reject::UnknownKind);
    }
    schema(&value)?;
    let message: Message = serde_json::from_value(value).map_err(|_| Reject::Malformed)?;
    validation::validate_message(&message)?;
    Ok(message)
}

pub(super) fn encode(message: &Message) -> Result<Vec<u8>, Reject> {
    validate(message)?;
    let bytes = serde_json::to_vec(message).map_err(|_| Reject::Malformed)?;
    bound(bytes.len())?;
    Ok(bytes)
}

pub(super) fn validate(message: &Message) -> Result<(), Reject> {
    validation::validate_message(message)?;
    let value = serde_json::to_value(message).map_err(|_| Reject::Malformed)?;
    schema(&value)?;
    bound(
        serde_json::to_vec(message)
            .map_err(|_| Reject::Malformed)?
            .len(),
    )
}

fn schema(value: &Value) -> Result<(), Reject> {
    let schema: Value =
        serde_json::from_str(GAME_FACTS_REFERENCE_V1_SCHEMA).map_err(|_| Reject::Malformed)?;
    let validator = jsonschema::validator_for(&schema).map_err(|_| Reject::Malformed)?;
    if validator.is_valid(value) {
        Ok(())
    } else {
        Err(Reject::Malformed)
    }
}

fn bound(bytes: usize) -> Result<(), Reject> {
    if bytes <= GAME_FACTS_REFERENCE_V1_MAX_MESSAGE_BYTES {
        Ok(())
    } else {
        Err(Reject::ResultLimitExceeded)
    }
}
