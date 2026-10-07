// SPDX-License-Identifier: MIT

mod query;

use self::query::{query, query_validate, query_validate_response};
use super::types::*;
use super::{
    GAME_FACTS_REFERENCE_V1_MAX_MESSAGE_BYTES, GAME_FACTS_REFERENCE_V1_PROFILE,
    GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST,
};
use crate::game_facts_reference_v1::GameFactsReferenceV1Rejection as Reject;
use std::collections::BTreeSet;

const MAX_SAFE: u64 = 9_007_199_254_740_991;
const MAX_RULES: usize = 16;
const MAX_INPUTS: usize = 64;
const MAX_COMBINATIONS: usize = 256;
const MAX_IDENTITY: usize = 128;
const MAX_FACTS_IDENTITY: usize = 256;

pub(super) fn validate_message(message: &Message) -> Result<(), Reject> {
    require(
        message.protocol_version == GAME_FACTS_REFERENCE_V1_PROFILE
            && message.schema_digest == GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST,
        Reject::UnsupportedVersion,
    )?;
    identity(&message.correlation_id)?;
    require(
        message.provenance.artifact == "sts2-protocol/game-facts-reference-v1"
            && message.provenance.source == "schemas/game-facts-reference-v1.schema.json"
            && message.provenance.generator == "hand-authored",
        Reject::InvalidRequest,
    )?;
    match message.kind {
        MessageKind::QueryRequest => {
            require(
                message.query.is_some()
                    && message.result.is_none()
                    && message.capabilities.is_none()
                    && message.error.is_none(),
                Reject::Malformed,
            )?;
            query(message.query.as_ref().ok_or(Reject::Malformed)?)?;
        }
        MessageKind::QueryResponse => {
            require(
                message.query.is_some()
                    && message.result.is_some()
                    && message.capabilities.is_none()
                    && message.error.is_none(),
                Reject::Malformed,
            )?;
            let query = message.query.as_ref().ok_or(Reject::Malformed)?;
            query_validate_response(query, message.result.as_ref().ok_or(Reject::Malformed)?)?;
        }
        MessageKind::CapabilitiesResponse => {
            require(
                message.query.is_none()
                    && message.result.is_none()
                    && message.capabilities.is_some()
                    && message.error.is_none(),
                Reject::Malformed,
            )?;
            capabilities(message.capabilities.as_ref().ok_or(Reject::Malformed)?)?;
        }
        MessageKind::ErrorResponse => {
            require(
                message.result.is_none()
                    && message.capabilities.is_none()
                    && message.error.is_some(),
                Reject::Malformed,
            )?;
            if let Some(query) = message.query.as_ref() {
                query_validate(query)?;
            }
            error(message.error.as_ref().ok_or(Reject::Malformed)?)?;
        }
    }
    Ok(())
}

pub(super) fn validate_response_against_request(
    request: &Message,
    response: &Message,
) -> Result<(), Reject> {
    validate_message(request)?;
    validate_message(response)?;
    require(
        request.kind == MessageKind::QueryRequest
            && matches!(
                response.kind,
                MessageKind::QueryResponse | MessageKind::ErrorResponse
            ),
        Reject::InvalidRequest,
    )?;
    require(
        request.correlation_id == response.correlation_id,
        Reject::InvalidBinding,
    )?;
    require(
        request.query == response.query && response.query.is_some(),
        Reject::InvalidBinding,
    )
}

fn observation(input: &RuleInput, mode: BindingMode) -> Result<(), Reject> {
    match mode {
        BindingMode::Static => require(input.observation.is_none(), Reject::InvalidBinding)?,
        BindingMode::Live => {
            let observed = input.observation.as_ref().ok_or(Reject::InvalidBinding)?;
            match observed.availability {
                Availability::Available => {
                    let value = observed.value.as_ref().ok_or(Reject::Malformed)?;
                    require(observed.reason.is_none(), Reject::Malformed)?;
                    typed_value(value, input.unit)?;
                }
                _ => {
                    require(observed.value.is_none(), Reject::Malformed)?;
                    bounded_text(observed.reason.as_ref().ok_or(Reject::Malformed)?, 256)?;
                }
            }
        }
    }
    source_ref(&input.source_ref)
}

fn typed_value(value: &Value, unit: Unit) -> Result<(), Reject> {
    match value {
        Value::Integer { value } => require(
            value.unsigned_abs() <= MAX_SAFE && !matches!(unit, Unit::Entity | Unit::Boolean),
            Reject::InvalidBounds,
        ),
        Value::Boolean { .. } => require(unit == Unit::Boolean, Reject::Malformed),
        Value::EntityRef { id, .. } => {
            require(unit == Unit::Entity, Reject::Malformed)?;
            identity(id)
        }
    }
}

fn source_ref(source: &SourceRef) -> Result<(), Reject> {
    if let Some(reference) = &source.r#ref {
        require(valid_opaque(reference, 128), Reject::InvalidIdentity)?;
    }
    Ok(())
}

fn inventory(binding: &InventoryBinding) -> Result<(), Reject> {
    for value in [
        &binding.build_id,
        &binding.mode_id,
        &binding.manifest.adapter_compatibility,
        &binding.manifest.content_set_revision,
        &binding.manifest.localized_text_revision,
        &binding.manifest.inventory_revision,
    ] {
        require(
            valid_opaque(value, MAX_FACTS_IDENTITY),
            Reject::InvalidIdentity,
        )?;
    }
    safe(binding.manifest.catalog_generation)
}

fn capabilities(capabilities: &Capabilities) -> Result<(), Reject> {
    require(
        capabilities.profile == GAME_FACTS_REFERENCE_V1_PROFILE
            && capabilities.rules_reference_version == 2
            && capabilities.max_rule_ids as usize == MAX_RULES
            && capabilities.max_inputs_per_rule as usize == MAX_INPUTS
            && capabilities.max_unsupported_combinations as usize == MAX_COMBINATIONS
            && capabilities.max_message_bytes as usize == GAME_FACTS_REFERENCE_V1_MAX_MESSAGE_BYTES,
        Reject::InvalidBounds,
    )?;
    if let Some(policy) = &capabilities.snapshot_policy {
        require(
            policy.supports_live
                && (1..=65_536).contains(&policy.lifetime_generations)
                && (1..=64).contains(&policy.max_retained_snapshots)
                && policy.expiry_behavior == ExpiryBehavior::RejectStaleSnapshot,
            Reject::InvalidBounds,
        )?;
        let expected = [
            SnapshotInvalidator::Restore,
            SnapshotInvalidator::Restart,
            SnapshotInvalidator::ProfileChange,
            SnapshotInvalidator::ContentChange,
            SnapshotInvalidator::RunChange,
            SnapshotInvalidator::EpochChange,
        ];
        let actual: BTreeSet<_> = policy.invalidated_by.iter().copied().collect();
        require(
            policy.invalidated_by.len() == expected.len()
                && actual == expected.into_iter().collect(),
            Reject::InvalidRequest,
        )?;
    }
    Ok(())
}

fn error(error: &ErrorBody) -> Result<(), Reject> {
    if let Some(field) = &error.field {
        identity(field)?;
    }
    if let Some(reason) = &error.reason {
        bounded_text(reason, 256)?;
    }
    Ok(())
}

fn instance_ref(reference: &InstanceRef) -> Result<(), Reject> {
    identity(&reference.instance_id)?;
    identity(&reference.run_id)?;
    identity(&reference.entity_id)?;
    safe(reference.epoch)
}

fn snapshot_ref(reference: &SnapshotRef) -> Result<(), Reject> {
    identity(&reference.snapshot_id)?;
    instance_ref(&reference.instance_ref)?;
    safe(reference.state_generation)
}

fn identity(value: &str) -> Result<(), Reject> {
    require(
        !value.is_empty()
            && value.len() <= MAX_IDENTITY
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte)),
        Reject::InvalidIdentity,
    )
}

fn valid_opaque(value: &str, maximum: usize) -> bool {
    let bytes = value.as_bytes();
    !value.is_empty()
        && bytes.len() <= maximum
        && !bytes
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"file"))
        && !bytes
            .windows(b"exception".len())
            .any(|part| part.eq_ignore_ascii_case(b"exception"))
        && value.split('.').count() <= 8
        && value.split('.').all(|part| !part.is_empty())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        && !value.contains("..")
}

fn locale(value: &str) -> Result<(), Reject> {
    let mut parts = value.split('-');
    let first = parts.next().unwrap_or_default();
    require(
        (2..=3).contains(&first.len())
            && first.bytes().all(|byte| byte.is_ascii_alphabetic())
            && value.len() <= 35
            && parts.all(|part| {
                (2..=8).contains(&part.len())
                    && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
            }),
        Reject::InvalidIdentity,
    )
}

fn bounded_text(value: &str, maximum: usize) -> Result<(), Reject> {
    require(
        !value.is_empty()
            && value.chars().count() <= maximum
            && value.len() <= maximum
            && !value.chars().any(char::is_control),
        Reject::InvalidBounds,
    )
}

fn safe(value: u64) -> Result<(), Reject> {
    require(value <= MAX_SAFE, Reject::InvalidBounds)
}
fn require(condition: bool, reject: Reject) -> Result<(), Reject> {
    if condition { Ok(()) } else { Err(reject) }
}
