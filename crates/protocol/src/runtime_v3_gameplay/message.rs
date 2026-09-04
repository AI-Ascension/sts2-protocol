// SPDX-License-Identifier: MIT

use super::*;

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayMessage {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeV3GameplayProvenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: RuntimeV3GameplayMessageKind,
    #[serde(deserialize_with = "required_nullable")]
    pub operation_id: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub observation: Option<RuntimeV3GameplayObservation>,
    #[serde(deserialize_with = "required_nullable")]
    pub action: Option<RuntimeV3GameplayAction>,
    #[serde(deserialize_with = "required_nullable")]
    pub status: Option<RuntimeV3GameplayStatus>,
    #[serde(deserialize_with = "required_nullable")]
    pub error_code: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub effect_witness: Option<RuntimeV3GameplayEffectWitness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeV3GameplayActionResult {
    pub status: RuntimeV3GameplayStatus,
    pub observation: Option<RuntimeV3GameplayObservation>,
    pub error_code: Option<String>,
    pub effect_witness: Option<RuntimeV3GameplayEffectWitness>,
}

impl RuntimeV3GameplayMessage {
    #[must_use]
    pub fn state_request(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
    ) -> Self {
        Self::base(
            metadata,
            context,
            RuntimeV3GameplayMessageKind::StateRequest,
        )
    }

    #[must_use]
    pub fn state_response(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
        observation: RuntimeV3GameplayObservation,
    ) -> Self {
        Self {
            observation: Some(observation),
            ..Self::base(
                metadata,
                context,
                RuntimeV3GameplayMessageKind::StateResponse,
            )
        }
    }

    #[must_use]
    pub fn action_request(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
        operation_id: impl Into<String>,
        action: RuntimeV3GameplayAction,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            action: Some(action),
            ..Self::base(
                metadata,
                context,
                RuntimeV3GameplayMessageKind::ActionRequest,
            )
        }
    }

    #[must_use]
    pub fn reconcile_request(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
        operation_id: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            ..Self::base(
                metadata,
                context,
                RuntimeV3GameplayMessageKind::ReconcileRequest,
            )
        }
    }

    #[must_use]
    pub fn result(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
        operation_id: impl Into<String>,
        action: RuntimeV3GameplayAction,
        result: RuntimeV3GameplayActionResult,
        kind: RuntimeV3GameplayMessageKind,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            observation: result.observation,
            action: Some(action),
            status: Some(result.status),
            error_code: result.error_code,
            effect_witness: result.effect_witness,
            ..Self::base(metadata, context, kind)
        }
    }

    pub fn validate(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        self.validate_metadata()?;
        for identity in [
            &self.correlation_id,
            &self.instance_id,
            &self.session_id,
            &self.lease_id,
        ] {
            validate_identity(identity)?;
        }
        if self.lease_epoch > RUNTIME_V3_GAMEPLAY_MAX_GENERATION
            || self.generation > RUNTIME_V3_GAMEPLAY_MAX_GENERATION
        {
            return Err(RuntimeV3GameplayValidationError::GenerationBounds);
        }
        if let Some(operation_id) = &self.operation_id {
            validate_identity(operation_id)?;
        }
        if let Some(observation) = &self.observation {
            observation.validate()?;
        }
        if let Some(action) = &self.action {
            action.validate()?;
        }
        if let Some(error_code) = &self.error_code {
            validate_identity(error_code)?;
        }
        if let Some(witness) = &self.effect_witness {
            witness.validate()?;
        }
        match self.kind {
            RuntimeV3GameplayMessageKind::StateRequest => self.shape(
                self.operation_id.is_none()
                    && self.observation.is_none()
                    && self.action.is_none()
                    && self.status.is_none()
                    && self.error_code.is_none()
                    && self.effect_witness.is_none(),
            ),
            RuntimeV3GameplayMessageKind::StateResponse => self.shape(
                self.operation_id.is_none()
                    && self.observation_generation_matches()
                    && self.action.is_none()
                    && self.status.is_none()
                    && self.error_code.is_none()
                    && self.effect_witness.is_none(),
            ),
            RuntimeV3GameplayMessageKind::ActionRequest => self.shape(
                self.operation_id.is_some()
                    && self.observation.is_none()
                    && self.action.is_some()
                    && self.status.is_none()
                    && self.error_code.is_none()
                    && self.effect_witness.is_none(),
            ),
            RuntimeV3GameplayMessageKind::ReconcileRequest => self.shape(
                self.operation_id.is_some()
                    && self.observation.is_none()
                    && self.action.is_none()
                    && self.status.is_none()
                    && self.error_code.is_none()
                    && self.effect_witness.is_none(),
            ),
            RuntimeV3GameplayMessageKind::ActionResponse
            | RuntimeV3GameplayMessageKind::ReconcileResponse => self.validate_result(),
        }
    }

    fn validate_metadata(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        let valid = self.protocol_version == RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION
            && self.schema_digest == RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST
            && self.provenance.artifact == RUNTIME_V3_GAMEPLAY_ARTIFACT
            && self.provenance.source == RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE
            && self.provenance.generator == RUNTIME_V3_GAMEPLAY_GENERATOR
            && is_digest(&self.schema_digest);
        self.shape(valid)
            .map_err(|_| RuntimeV3GameplayValidationError::Metadata)
    }

    fn validate_result(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        let valid = self.operation_id.is_some()
            && self.action.is_some()
            && self.status.is_some()
            && match self.status {
                Some(RuntimeV3GameplayStatus::Accepted) => {
                    self.observation_generation_matches()
                        && self.error_code.is_none()
                        && self.effect_witness.is_none()
                }
                Some(RuntimeV3GameplayStatus::Settled) => {
                    self.observation_generation_matches()
                        && self.error_code.is_none()
                        && self.effect_witness.as_ref().is_some_and(|value| {
                            value.generation == self.generation
                                && self.action.as_ref().is_some_and(|action| {
                                    value.card_index == action.card_index
                                        && value.target_id == action.target_id
                                })
                        })
                }
                Some(RuntimeV3GameplayStatus::Rejected | RuntimeV3GameplayStatus::Cancelled) => {
                    self.observation_generation_matches()
                        && self.error_code.is_some()
                        && self.effect_witness.is_none()
                }
                Some(RuntimeV3GameplayStatus::Unknown) => {
                    self.observation.is_none()
                        && self.error_code.is_some()
                        && self.effect_witness.is_none()
                }
                None => false,
            };
        self.shape(valid)
    }

    fn observation_generation_matches(&self) -> bool {
        self.observation
            .as_ref()
            .is_some_and(|value| value.generation == self.generation)
    }

    fn shape(&self, valid: bool) -> Result<(), RuntimeV3GameplayValidationError> {
        valid
            .then_some(())
            .ok_or(RuntimeV3GameplayValidationError::ResultShape)
    }

    fn base(
        metadata: RuntimeV3GameplayMetadata,
        context: RuntimeV3GameplayContext,
        kind: RuntimeV3GameplayMessageKind,
    ) -> Self {
        Self {
            protocol_version: metadata.protocol_version,
            schema_digest: metadata.schema_digest,
            provenance: metadata.provenance,
            correlation_id: context.correlation_id,
            instance_id: context.instance_id,
            session_id: context.session_id,
            lease_id: context.lease_id,
            lease_epoch: context.lease_epoch,
            generation: context.generation,
            kind,
            operation_id: None,
            observation: None,
            action: None,
            status: None,
            error_code: None,
            effect_witness: None,
        }
    }
}

fn validate_identity(value: &str) -> Result<(), RuntimeV3GameplayValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(RuntimeV3GameplayValidationError::InvalidIdentity);
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
