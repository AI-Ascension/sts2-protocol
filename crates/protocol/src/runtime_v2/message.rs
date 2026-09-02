// SPDX-License-Identifier: MIT

use super::*;

/// A complete Runtime-v2 request, response, or reconciliation receipt.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2Message {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeV2Provenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: RuntimeV2MessageKind,
    pub operation_id: Option<String>,
    pub observation: Option<RuntimeV2Observation>,
    pub action: Option<RuntimeV2Action>,
    pub status: Option<RuntimeV2Status>,
    pub error_code: Option<String>,
    pub effect_witness: Option<RuntimeV2EffectWitness>,
}

impl RuntimeV2Message {
    /// Validates metadata, bounds, identities, and kind-specific receipt shape.
    pub fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        self.validate_metadata()?;
        self.validate_common_fields()?;
        match self.kind {
            RuntimeV2MessageKind::StateRequest => self.validate_state_request(),
            RuntimeV2MessageKind::StateResponse => self.validate_state_response(),
            RuntimeV2MessageKind::ActionRequest => self.validate_action_request(),
            RuntimeV2MessageKind::ReconcileRequest => self.validate_reconcile_request(),
            RuntimeV2MessageKind::ActionResponse | RuntimeV2MessageKind::ReconcileResponse => {
                self.validate_result()
            }
        }
    }

    fn validate_metadata(&self) -> Result<(), RuntimeV2ValidationError> {
        RuntimeV2Metadata {
            protocol_version: self.protocol_version.clone(),
            schema_digest: self.schema_digest.clone(),
            provenance: self.provenance.clone(),
        }
        .validate()
    }

    fn validate_common_fields(&self) -> Result<(), RuntimeV2ValidationError> {
        for identity in [
            &self.correlation_id,
            &self.instance_id,
            &self.session_id,
            &self.lease_id,
        ] {
            validate_identity(identity)?;
        }
        if self.lease_epoch > RUNTIME_V2_MAX_GENERATION
            || self.generation > RUNTIME_V2_MAX_GENERATION
        {
            return Err(RuntimeV2ValidationError::GenerationBounds);
        }
        if let Some(operation_id) = &self.operation_id {
            validate_identity(operation_id)?;
        }
        if let Some(observation) = self.observation {
            observation.validate()?;
        }
        if let Some(action) = &self.action {
            action.validate()?;
        }
        if let Some(error_code) = &self.error_code {
            validate_identity(error_code)?;
        }
        if let Some(effect_witness) = self.effect_witness.as_ref() {
            effect_witness.validate()?;
        }
        Ok(())
    }

    fn validate_state_request(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.operation_id.is_none()
            && self.observation.is_none()
            && self.action.is_none()
            && self.status.is_none()
            && self.error_code.is_none()
            && self.effect_witness.is_none()
        {
            Ok(())
        } else {
            Err(RuntimeV2ValidationError::ResultShape)
        }
    }

    fn validate_state_response(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.operation_id.is_none()
            && self.observation_generation_matches()
            && self.action.is_none()
            && self.status.is_none()
            && self.error_code.is_none()
            && self.effect_witness.is_none()
        {
            Ok(())
        } else {
            Err(RuntimeV2ValidationError::ResultShape)
        }
    }

    fn validate_action_request(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.operation_id.is_some()
            && self.action.is_some()
            && self.observation.is_none()
            && self.status.is_none()
            && self.error_code.is_none()
            && self.effect_witness.is_none()
        {
            Ok(())
        } else {
            Err(RuntimeV2ValidationError::ResultShape)
        }
    }

    fn validate_reconcile_request(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.operation_id.is_some()
            && self.action.is_none()
            && self.observation.is_none()
            && self.status.is_none()
            && self.error_code.is_none()
            && self.effect_witness.is_none()
        {
            Ok(())
        } else {
            Err(RuntimeV2ValidationError::ResultShape)
        }
    }

    fn validate_result(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.operation_id.is_some()
            && self.action.is_some()
            && self.status.is_some()
            && self.result_fields_match_status()
        {
            Ok(())
        } else {
            Err(RuntimeV2ValidationError::ResultShape)
        }
    }

    fn result_fields_match_status(&self) -> bool {
        match self.status {
            Some(RuntimeV2Status::Accepted) => {
                self.observation_generation_matches()
                    && self.error_code.is_none()
                    && self.effect_witness.is_none()
            }
            Some(RuntimeV2Status::Settled) => {
                self.observation_generation_matches()
                    && self.error_code.is_none()
                    && self
                        .effect_witness
                        .as_ref()
                        .is_some_and(|witness| witness.generation == self.generation)
            }
            Some(RuntimeV2Status::Rejected | RuntimeV2Status::Cancelled) => {
                self.observation_generation_matches()
                    && self.error_code.is_some()
                    && self.effect_witness.is_none()
            }
            Some(RuntimeV2Status::Unknown) => {
                self.observation.is_none()
                    && self.error_code.is_some()
                    && self.effect_witness.is_none()
            }
            None => false,
        }
    }

    fn observation_generation_matches(&self) -> bool {
        self.observation
            .is_some_and(|observation| observation.generation == self.generation)
    }
}
