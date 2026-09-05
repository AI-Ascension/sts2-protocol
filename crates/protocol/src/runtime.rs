// SPDX-License-Identifier: MIT

mod message;

pub use message::RuntimeMessage;

/// Version of the first live vertical-slice contract.
pub const RUNTIME_PROTOCOL_VERSION: &str = "runtime-v1";
/// Release-like artifact identity carried by every runtime message.
pub const RUNTIME_ARTIFACT: &str = "sts2-protocol/runtime-v1";
/// Repository-relative schema source recorded in runtime provenance.
pub const RUNTIME_SCHEMA_SOURCE: &str = "schemas/runtime-v1.schema.json";
/// Generator recorded for the hand-authored runtime schema.
pub const RUNTIME_GENERATOR: &str = "hand-authored";
/// SHA-256 digest of the canonical runtime schema source.
pub const RUNTIME_SCHEMA_DIGEST: &str =
    "a76086d7a68668fd4cff53999369d2b450b0d6623827393882f458f2aa1f93eb";
/// The only action admitted by this first host-visible slice.
pub const RUNTIME_ACTION_ID: &str = "show_runtime_probe";
/// Maximum JSON-safe generation accepted at this boundary.
pub const RUNTIME_MAX_GENERATION: u64 = 9_007_199_254_740_991;
/// Maximum host-visible action count exposed in an observation.
pub const RUNTIME_MAX_ACTION_COUNT: u64 = 1024;

/// Four wire message shapes in the runtime contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMessageKind {
    StateRequest,
    StateResponse,
    ActionRequest,
    ActionResponse,
}

/// The result statuses for a runtime action.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Accepted,
    Rejected,
}

/// Provenance identifying the inert release-like Runtime-v1 artifact.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for RuntimeProvenance {
    fn default() -> Self {
        Self {
            artifact: RUNTIME_ARTIFACT.to_owned(),
            source: RUNTIME_SCHEMA_SOURCE.to_owned(),
            generator: RUNTIME_GENERATOR.to_owned(),
        }
    }
}

impl RuntimeProvenance {
    fn validate(&self) -> Result<(), RuntimeValidationError> {
        if self.artifact != RUNTIME_ARTIFACT
            || self.source != RUNTIME_SCHEMA_SOURCE
            || self.generator != RUNTIME_GENERATOR
        {
            return Err(RuntimeValidationError::Provenance);
        }
        Ok(())
    }
}

/// Bounded host observation used by the first live slice.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeObservation {
    pub host_ready: bool,
    pub overlay_visible: bool,
    pub screen: String,
    pub action_count: u64,
}

impl RuntimeObservation {
    /// Validates the host projection without interpreting game rules.
    pub fn validate(&self) -> Result<(), RuntimeValidationError> {
        if self.screen.is_empty()
            || self.screen.len() > 64
            || !self
                .screen
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
        {
            return Err(RuntimeValidationError::ObservationBounds);
        }
        if self.action_count > RUNTIME_MAX_ACTION_COUNT {
            return Err(RuntimeValidationError::ObservationBounds);
        }
        Ok(())
    }
}

/// The one safe host-visible action admitted by this contract.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeAction {
    pub action_id: String,
}

impl RuntimeAction {
    /// Validates the fixed action identity.
    pub fn validate(&self) -> Result<(), RuntimeValidationError> {
        if self.action_id != RUNTIME_ACTION_ID {
            return Err(RuntimeValidationError::ActionBounds);
        }
        Ok(())
    }
}

/// Witness that the host-visible effect was observed after acceptance.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeEffectWitness {
    pub kind: String,
    pub generation: u64,
}

impl RuntimeEffectWitness {
    /// Validates the effect witness identity and generation bound.
    pub fn validate(&self) -> Result<(), RuntimeValidationError> {
        if self.kind != "status_overlay_visible" || self.generation > RUNTIME_MAX_GENERATION {
            return Err(RuntimeValidationError::EffectBounds);
        }
        Ok(())
    }
}

/// Deterministic validation failures for runtime contract values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeValidationError {
    Metadata,
    Provenance,
    Identity,
    GenerationBounds,
    ObservationBounds,
    ActionBounds,
    EffectBounds,
    ResultShape,
}

impl std::fmt::Display for RuntimeValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Metadata => "runtime metadata is unsupported",
            Self::Provenance => "runtime provenance is unsupported",
            Self::Identity => "runtime identity is empty, unsafe, or too long",
            Self::GenerationBounds => "runtime generation is outside the bound",
            Self::ObservationBounds => "runtime observation is outside the bound",
            Self::ActionBounds => "runtime action is outside the bound",
            Self::EffectBounds => "runtime effect witness is outside the bound",
            Self::ResultShape => "runtime message members do not match the message kind/status",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RuntimeValidationError {}

fn validate_identity(value: &str) -> Result<(), RuntimeValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(RuntimeValidationError::Identity);
    }
    Ok(())
}
