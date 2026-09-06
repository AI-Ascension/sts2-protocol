// SPDX-License-Identifier: MIT

use super::*;

/// A complete Runtime-v1 request or response with every nullable member explicit on the wire.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMessage {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeProvenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: RuntimeMessageKind,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub observation: Option<RuntimeObservation>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub action: Option<RuntimeAction>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub status: Option<RuntimeStatus>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub error_code: Option<String>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub effect_witness: Option<RuntimeEffectWitness>,
}

impl RuntimeMessage {
    /// Validates metadata, identities, bounds, and the kind-specific member shape.
    pub fn validate(&self) -> Result<(), RuntimeValidationError> {
        self.validate_metadata()?;
        self.validate_common_fields()?;
        match self.kind {
            RuntimeMessageKind::StateRequest => self.validate_state_request(),
            RuntimeMessageKind::StateResponse => self.validate_state_response(),
            RuntimeMessageKind::ActionRequest => self.validate_action_request(),
            RuntimeMessageKind::ActionResponse => self.validate_action_response(),
        }
    }

    fn validate_metadata(&self) -> Result<(), RuntimeValidationError> {
        if self.protocol_version != RUNTIME_PROTOCOL_VERSION
            || self.schema_digest != RUNTIME_SCHEMA_DIGEST
        {
            return Err(RuntimeValidationError::Metadata);
        }
        self.provenance.validate()
    }

    fn validate_common_fields(&self) -> Result<(), RuntimeValidationError> {
        for identity in [
            &self.correlation_id,
            &self.instance_id,
            &self.session_id,
            &self.lease_id,
        ] {
            validate_identity(identity)?;
        }
        if self.lease_epoch > RUNTIME_MAX_GENERATION || self.generation > RUNTIME_MAX_GENERATION {
            return Err(RuntimeValidationError::GenerationBounds);
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
        if let Some(effect_witness) = &self.effect_witness {
            effect_witness.validate()?;
        }
        Ok(())
    }

    fn validate_state_request(&self) -> Result<(), RuntimeValidationError> {
        if self.observation.is_none() && self.action.is_none() && self.result_members_absent() {
            Ok(())
        } else {
            Err(RuntimeValidationError::ResultShape)
        }
    }

    fn validate_state_response(&self) -> Result<(), RuntimeValidationError> {
        if self.observation.is_some() && self.action.is_none() && self.result_members_absent() {
            Ok(())
        } else {
            Err(RuntimeValidationError::ResultShape)
        }
    }

    fn validate_action_request(&self) -> Result<(), RuntimeValidationError> {
        if self.observation.is_none() && self.action.is_some() && self.result_members_absent() {
            Ok(())
        } else {
            Err(RuntimeValidationError::ResultShape)
        }
    }

    fn validate_action_response(&self) -> Result<(), RuntimeValidationError> {
        let outcome_matches = match self.status {
            Some(RuntimeStatus::Accepted) => {
                self.error_code.is_none() && self.effect_witness.is_some()
            }
            Some(RuntimeStatus::Rejected) => {
                self.error_code.is_some() && self.effect_witness.is_none()
            }
            None => false,
        };
        if self.observation.is_some() && self.action.is_some() && outcome_matches {
            Ok(())
        } else {
            Err(RuntimeValidationError::ResultShape)
        }
    }

    fn result_members_absent(&self) -> bool {
        self.status.is_none() && self.error_code.is_none() && self.effect_witness.is_none()
    }
}
