// SPDX-License-Identifier: MIT

mod constructors;
mod message;

pub use constructors::RuntimeV2ActionResult;
pub use message::RuntimeV2Message;

/// Version of the bounded gameplay-operation contract.
pub const RUNTIME_V2_PROTOCOL_VERSION: &str = "runtime-v2";
/// Release-like artifact name carried by every Runtime-v2 message.
pub const RUNTIME_V2_ARTIFACT: &str = "sts2-protocol/runtime-v2";
/// Repository-relative source of the Runtime-v2 schema.
pub const RUNTIME_V2_SCHEMA_SOURCE: &str = "schemas/runtime-v2.schema.json";
/// Generator recorded for the hand-authored Runtime-v2 schema.
pub const RUNTIME_V2_GENERATOR: &str = "hand-authored";
/// SHA-256 of the canonical Runtime-v2 schema source bytes.
pub const RUNTIME_V2_SCHEMA_DIGEST: &str =
    "f7963b19c8ed5bbdc02c08e83c7a2e16c4771ed5eb798b29a8208d7a917a86c2";
/// The single bounded gameplay action in this profile.
pub const RUNTIME_V2_ACTION_ID: &str = "end_turn";
/// The witness required for an authoritative end-turn settlement.
pub const RUNTIME_V2_EFFECT_KIND: &str = "turn_end_settled";
/// The only observation phase in which `end_turn` is legal.
pub const RUNTIME_V2_PLAYER_TURN_PHASE: &str = "combat/player_turn";
/// Maximum generation that remains exact in common JSON number implementations.
pub const RUNTIME_V2_MAX_GENERATION: u64 = 9_007_199_254_740_991;
/// Maximum turn index represented by the bounded contract.
pub const RUNTIME_V2_MAX_TURN_INDEX: u16 = 1024;

/// A bounded combat phase used by the neutral domain observation.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub enum RuntimeV2CombatPhase {
    #[serde(rename = "outside_combat")]
    OutsideCombat,
    #[serde(rename = "combat/player_turn")]
    PlayerTurn,
    #[serde(rename = "combat/enemy_turn")]
    EnemyTurn,
}

/// The five possible operation outcomes in Runtime-v2.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV2Status {
    Accepted,
    Settled,
    Rejected,
    Unknown,
    Cancelled,
}

/// The message shapes in the Runtime-v2 profile.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV2MessageKind {
    StateRequest,
    StateResponse,
    ActionRequest,
    ActionResponse,
    ReconcileRequest,
    ReconcileResponse,
}

/// Release metadata required by every Runtime-v2 message.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2Metadata {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeV2Provenance,
}

impl RuntimeV2Metadata {
    /// Creates metadata for the checked-in Runtime-v2 schema digest.
    #[must_use]
    pub fn new() -> Self {
        Self {
            protocol_version: RUNTIME_V2_PROTOCOL_VERSION.to_owned(),
            schema_digest: RUNTIME_V2_SCHEMA_DIGEST.to_owned(),
            provenance: RuntimeV2Provenance::default(),
        }
    }

    /// Validates the fixed profile and inert provenance.
    pub fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.protocol_version != RUNTIME_V2_PROTOCOL_VERSION
            || self.schema_digest != RUNTIME_V2_SCHEMA_DIGEST
            || !is_digest(&self.schema_digest)
        {
            return Err(RuntimeV2ValidationError::Metadata);
        }
        self.provenance.validate()
    }
}

impl Default for RuntimeV2Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Provenance identifying the inert release-like Runtime-v2 artifact.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2Provenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

/// Request context preserved across an operation and its receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeV2Context {
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
}

impl RuntimeV2Context {
    /// Creates a context without granting lease or mutation authority.
    #[must_use]
    pub fn new(
        correlation_id: impl Into<String>,
        instance_id: impl Into<String>,
        session_id: impl Into<String>,
        lease_id: impl Into<String>,
        lease_epoch: u64,
        generation: u64,
    ) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            instance_id: instance_id.into(),
            session_id: session_id.into(),
            lease_id: lease_id.into(),
            lease_epoch,
            generation,
        }
    }
}

impl Default for RuntimeV2Provenance {
    fn default() -> Self {
        Self {
            artifact: RUNTIME_V2_ARTIFACT.to_owned(),
            source: RUNTIME_V2_SCHEMA_SOURCE.to_owned(),
            generator: RUNTIME_V2_GENERATOR.to_owned(),
        }
    }
}

impl RuntimeV2Provenance {
    fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.artifact != RUNTIME_V2_ARTIFACT
            || self.source != RUNTIME_V2_SCHEMA_SOURCE
            || self.generator != RUNTIME_V2_GENERATOR
        {
            return Err(RuntimeV2ValidationError::Provenance);
        }
        Ok(())
    }
}

/// Bounded domain state carried in a state or settled receipt.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2Observation {
    pub combat_phase: RuntimeV2CombatPhase,
    pub turn_index: u16,
    pub host_ready: bool,
    pub generation: u64,
}

impl RuntimeV2Observation {
    /// Validates observation bounds without interpreting host or game authority.
    pub fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.turn_index > RUNTIME_V2_MAX_TURN_INDEX {
            return Err(RuntimeV2ValidationError::ObservationBounds);
        }
        if self.generation > RUNTIME_V2_MAX_GENERATION {
            return Err(RuntimeV2ValidationError::GenerationBounds);
        }
        Ok(())
    }
}

/// The fixed action identity admitted by Runtime-v2.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2Action {
    pub action_id: String,
}

impl RuntimeV2Action {
    /// Validates the fixed action identity.
    pub fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.action_id != RUNTIME_V2_ACTION_ID {
            return Err(RuntimeV2ValidationError::ActionBounds);
        }
        Ok(())
    }
}

/// A witness that the host authoritatively settled the end-turn effect.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV2EffectWitness {
    pub kind: String,
    pub generation: u64,
}

impl RuntimeV2EffectWitness {
    /// Validates the fixed settlement witness and generation bound.
    pub fn validate(&self) -> Result<(), RuntimeV2ValidationError> {
        if self.kind != RUNTIME_V2_EFFECT_KIND {
            return Err(RuntimeV2ValidationError::EffectBounds);
        }
        if self.generation > RUNTIME_V2_MAX_GENERATION {
            return Err(RuntimeV2ValidationError::GenerationBounds);
        }
        Ok(())
    }
}

/// Deterministic validation failures for Runtime-v2 values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeV2ValidationError {
    Metadata,
    Provenance,
    InvalidIdentity,
    GenerationBounds,
    ObservationBounds,
    ActionBounds,
    EffectBounds,
    ResultShape,
}

impl std::fmt::Display for RuntimeV2ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Metadata => "runtime-v2 metadata is unsupported",
            Self::Provenance => "runtime-v2 provenance is unsupported",
            Self::InvalidIdentity => "runtime-v2 identity is empty, unsafe, or too long",
            Self::GenerationBounds => "runtime-v2 generation is outside the bound",
            Self::ObservationBounds => "runtime-v2 observation is outside the bound",
            Self::ActionBounds => "runtime-v2 action is outside the fixed action identity",
            Self::EffectBounds => "runtime-v2 effect witness is outside the fixed identity",
            Self::ResultShape => "runtime-v2 message fields do not match the message kind/status",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RuntimeV2ValidationError {}

fn validate_identity(value: &str) -> Result<(), RuntimeV2ValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(RuntimeV2ValidationError::InvalidIdentity);
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
