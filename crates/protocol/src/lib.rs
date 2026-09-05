// SPDX-License-Identifier: MIT

mod descriptor;
mod envelope;
mod identity;
mod lifecycle;
mod poc;
mod runtime;
mod runtime_v2;
mod runtime_v3_gameplay;
mod serialization;

pub use descriptor::{
    ContractManifest, DigestAlgorithm, DigestDescriptor, Provenance, VersionProfile,
};
pub use envelope::{ErrorEnvelope, ErrorMetadata, ErrorOrigin, Retryability};
pub use identity::{
    CorrelationMetadata, IdentityMetadata, LineageMetadata, QualifiedId, SequenceMetadata,
};
pub use lifecycle::{
    CancellationMetadata, CancellationState, ClockKind, DeadlineMetadata, LifecycleMetadata,
    LifecycleState, NeutralMetadata, OperationStatus,
};
pub use poc::{
    POC_ARTIFACT, POC_GENERATOR, POC_MAX_GENERATION, POC_MAX_SETTLED_EFFECTS, POC_MAX_UNITS,
    POC_PROTOCOL_VERSION, POC_SCHEMA_DIGEST, POC_SCHEMA_SOURCE, PocAction, PocActionResult,
    PocMessage, PocMessageKind, PocMetadata, PocObservation, PocProvenance, PocStatus,
    PocValidationError,
};
pub use runtime::{
    RUNTIME_ACTION_ID, RUNTIME_ARTIFACT, RUNTIME_GENERATOR, RUNTIME_MAX_ACTION_COUNT,
    RUNTIME_MAX_GENERATION, RUNTIME_PROTOCOL_VERSION, RUNTIME_SCHEMA_DIGEST, RUNTIME_SCHEMA_SOURCE,
    RuntimeAction, RuntimeEffectWitness, RuntimeMessage, RuntimeMessageKind, RuntimeObservation,
    RuntimeProvenance, RuntimeStatus, RuntimeValidationError,
};
pub use runtime_v2::{
    RUNTIME_V2_ACTION_ID, RUNTIME_V2_ARTIFACT, RUNTIME_V2_EFFECT_KIND, RUNTIME_V2_GENERATOR,
    RUNTIME_V2_MAX_GENERATION, RUNTIME_V2_MAX_TURN_INDEX, RUNTIME_V2_PLAYER_TURN_PHASE,
    RUNTIME_V2_PROTOCOL_VERSION, RUNTIME_V2_SCHEMA_DIGEST, RUNTIME_V2_SCHEMA_SOURCE,
    RuntimeV2Action, RuntimeV2ActionResult, RuntimeV2CombatPhase, RuntimeV2Context,
    RuntimeV2EffectWitness, RuntimeV2Message, RuntimeV2MessageKind, RuntimeV2Metadata,
    RuntimeV2Observation, RuntimeV2Provenance, RuntimeV2Status, RuntimeV2ValidationError,
};
pub use runtime_v3_gameplay::{
    GameObservation, LegalAction, RUNTIME_V3_GAMEPLAY_ARTIFACT, RUNTIME_V3_GAMEPLAY_GENERATOR,
    RUNTIME_V3_GAMEPLAY_MAX_ENTITIES, RUNTIME_V3_GAMEPLAY_MAX_GENERATION,
    RUNTIME_V3_GAMEPLAY_MAX_LEGAL_ACTIONS, RUNTIME_V3_GAMEPLAY_MAX_TEXT_BYTES,
    RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION, RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST,
    RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE, RuntimeV3GameplayAction, RuntimeV3GameplayActionResult,
    RuntimeV3GameplayCard, RuntimeV3GameplayContext, RuntimeV3GameplayEnemy,
    RuntimeV3GameplayEnemyIntent, RuntimeV3GameplayLegalAction, RuntimeV3GameplayMessage,
    RuntimeV3GameplayMessageKind, RuntimeV3GameplayMetadata, RuntimeV3GameplayObservation,
    RuntimeV3GameplayPlayer, RuntimeV3GameplayProvenance, RuntimeV3GameplayRecovery,
    RuntimeV3GameplayRecoveryKind, RuntimeV3GameplayShopItem, RuntimeV3GameplayState,
    RuntimeV3GameplayStateKind, RuntimeV3GameplayStatus, RuntimeV3GameplayTransitionWitness,
    RuntimeV3GameplayValidationError, RuntimeV3GameplayWaitOutcome, RuntimeV3Message,
    RuntimeV3MessageKind,
};
pub use serialization::{canonical_json, decode_json};

/// The initial neutral metadata profile owned by this package.
pub const CONTRACT_PROFILE: &str = "sts2-neutral-contract-v1";

/// A validation failure for a neutral contract value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    Empty { field: &'static str },
    TooLong { field: &'static str, maximum: usize },
    InvalidCharacters { field: &'static str },
    AbsolutePath { field: &'static str },
    ParentPath { field: &'static str },
    InvalidDigest { field: &'static str },
    TooFewConsumers { minimum: usize },
    UnsortedConsumers,
    DuplicateConsumer,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty { field } => write!(formatter, "{field} must not be empty"),
            Self::TooLong { field, maximum } => {
                write!(formatter, "{field} exceeds {maximum} bytes")
            }
            Self::InvalidCharacters { field } => {
                write!(formatter, "{field} contains invalid characters")
            }
            Self::AbsolutePath { field } => write!(formatter, "{field} must be relative"),
            Self::ParentPath { field } => {
                write!(formatter, "{field} must not contain parent paths")
            }
            Self::InvalidDigest { field } => write!(formatter, "{field} is not a SHA-256 digest"),
            Self::TooFewConsumers { minimum } => {
                write!(formatter, "at least {minimum} consumers are required")
            }
            Self::UnsortedConsumers => formatter.write_str("consumers must be sorted"),
            Self::DuplicateConsumer => formatter.write_str("consumers must be unique"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub(crate) fn validate_token(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::Empty { field });
    }
    if value.len() > maximum {
        return Err(ValidationError::TooLong { field, maximum });
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"._:-/".contains(&byte))
    {
        return Err(ValidationError::InvalidCharacters { field });
    }
    Ok(())
}

pub(crate) fn validate_text(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::Empty { field });
    }
    if value.len() > maximum {
        return Err(ValidationError::TooLong { field, maximum });
    }
    if value.chars().any(char::is_control) {
        return Err(ValidationError::InvalidCharacters { field });
    }
    Ok(())
}
