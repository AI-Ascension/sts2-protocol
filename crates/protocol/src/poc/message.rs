// SPDX-License-Identifier: MIT

use super::metadata::POC_MAX_GENERATION;
use super::metadata::{POC_MAX_SETTLED_EFFECTS, POC_MAX_UNITS, PocMetadata};

mod wire;

/// The four message shapes in the POC contract.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PocMessageKind {
    StateRequest,
    StateResponse,
    ActionRequest,
    ActionResponse,
}

/// The only result statuses in the POC contract.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PocStatus {
    Accepted,
    Rejected,
}

/// The bounded state projection used by the fake vertical slice.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PocObservation {
    pub available_units: u16,
    pub settled_effects: u16,
}

impl PocObservation {
    /// Validates the bounded observation values.
    pub fn validate(&self) -> Result<(), PocValidationError> {
        if self.available_units > POC_MAX_UNITS || self.settled_effects > POC_MAX_SETTLED_EFFECTS {
            return Err(PocValidationError::ObservationBounds);
        }
        Ok(())
    }
}

/// The one typed action shape exposed by the POC contract.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PocAction {
    pub action_id: String,
    pub units: u16,
}

/// The owner-produced result fields carried by an action response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocActionResult {
    pub status: PocStatus,
    pub observation: PocObservation,
    pub error_code: Option<String>,
}

impl PocAction {
    /// Validates the action identity spelling and bounded typed argument.
    pub fn validate(&self) -> Result<(), PocValidationError> {
        if self.action_id != "use_budget" || self.units > POC_MAX_UNITS {
            return Err(PocValidationError::ActionBounds);
        }
        Ok(())
    }
}

/// A complete request or response with explicit optionality by message kind.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PocMessage {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: super::metadata::PocProvenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub generation: u64,
    pub kind: PocMessageKind,
    pub observation: Option<PocObservation>,
    pub action: Option<PocAction>,
    pub status: Option<PocStatus>,
    pub error_code: Option<String>,
}

impl PocMessage {
    /// Validates metadata, identities, bounds, and the kind-specific field shape.
    pub fn validate(&self) -> Result<(), PocValidationError> {
        PocMetadata {
            protocol_version: self.protocol_version.clone(),
            schema_digest: self.schema_digest.clone(),
            provenance: self.provenance.clone(),
        }
        .validate()?;
        validate_identity(&self.correlation_id)?;
        validate_identity(&self.instance_id)?;
        if self.generation > POC_MAX_GENERATION {
            return Err(PocValidationError::GenerationBounds);
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
        let shape = (
            self.observation.is_some(),
            self.action.is_some(),
            self.status,
        );
        match self.kind {
            PocMessageKind::StateRequest
                if shape == (false, false, None) && self.error_code.is_none() =>
            {
                Ok(())
            }
            PocMessageKind::StateResponse
                if shape == (true, false, None) && self.error_code.is_none() =>
            {
                Ok(())
            }
            PocMessageKind::ActionRequest
                if shape == (false, true, None) && self.error_code.is_none() =>
            {
                Ok(())
            }
            PocMessageKind::ActionResponse if shape.0 && shape.1 && shape.2.is_some() => {
                match self.status {
                    Some(PocStatus::Accepted) if self.error_code.is_none() => Ok(()),
                    Some(PocStatus::Rejected) if self.error_code.is_some() => Ok(()),
                    _ => Err(PocValidationError::ResultShape),
                }
            }
            _ => Err(PocValidationError::ResultShape),
        }
    }

    /// Returns a state request with no action or result fields.
    #[must_use]
    pub fn state_request(metadata: PocMetadata, correlation_id: &str, instance_id: &str) -> Self {
        Self::base(
            metadata,
            correlation_id,
            instance_id,
            0,
            PocMessageKind::StateRequest,
        )
    }

    /// Returns a state response containing one bounded observation.
    #[must_use]
    pub fn state_response(
        metadata: PocMetadata,
        correlation_id: &str,
        instance_id: &str,
        generation: u64,
        observation: PocObservation,
    ) -> Self {
        Self {
            observation: Some(observation),
            ..Self::base(
                metadata,
                correlation_id,
                instance_id,
                generation,
                PocMessageKind::StateResponse,
            )
        }
    }

    /// Returns an action request carrying the typed action and expected generation.
    #[must_use]
    pub fn action_request(
        metadata: PocMetadata,
        correlation_id: &str,
        instance_id: &str,
        generation: u64,
        action: PocAction,
    ) -> Self {
        Self {
            action: Some(action),
            ..Self::base(
                metadata,
                correlation_id,
                instance_id,
                generation,
                PocMessageKind::ActionRequest,
            )
        }
    }

    /// Returns an action result with the post-operation observation and status.
    #[must_use]
    pub fn action_response(
        metadata: PocMetadata,
        correlation_id: &str,
        instance_id: &str,
        generation: u64,
        action: PocAction,
        result: PocActionResult,
    ) -> Self {
        Self {
            observation: Some(result.observation),
            action: Some(action),
            status: Some(result.status),
            error_code: result.error_code,
            ..Self::base(
                metadata,
                correlation_id,
                instance_id,
                generation,
                PocMessageKind::ActionResponse,
            )
        }
    }

    fn base(
        metadata: PocMetadata,
        correlation_id: &str,
        instance_id: &str,
        generation: u64,
        kind: PocMessageKind,
    ) -> Self {
        Self {
            protocol_version: metadata.protocol_version,
            schema_digest: metadata.schema_digest,
            provenance: metadata.provenance,
            correlation_id: correlation_id.to_owned(),
            instance_id: instance_id.to_owned(),
            generation,
            kind,
            observation: None,
            action: None,
            status: None,
            error_code: None,
        }
    }
}

/// A deterministic validation failure for a POC message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PocValidationError {
    Metadata,
    Provenance,
    InvalidIdentity,
    ObservationBounds,
    ActionBounds,
    GenerationBounds,
    ResultShape,
}

impl std::fmt::Display for PocValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Metadata => "protocol version or schema digest is unsupported",
            Self::Provenance => "provenance does not identify the POC artifact",
            Self::InvalidIdentity => "identity is empty, unsafe, or too long",
            Self::ObservationBounds => "observation exceeds the POC bound",
            Self::ActionBounds => "action identity or typed argument is invalid",
            Self::GenerationBounds => "generation exceeds the cross-language bound",
            Self::ResultShape => "message fields do not match the message kind",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PocValidationError {}

fn validate_identity(value: &str) -> Result<(), PocValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-/".contains(&byte))
    {
        return Err(PocValidationError::InvalidIdentity);
    }
    Ok(())
}
