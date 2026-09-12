// SPDX-License-Identifier: MIT

mod coop_synchronization;
mod descriptor;
mod envelope;
mod exact_state;
mod identity;
mod lifecycle;
mod poc;
mod runtime;
mod runtime_v2;
mod runtime_v3_gameplay;
mod runtime_v4_expert;
mod runtime_v4_expert_rest_action;
mod seeded_run;
mod serialization;
mod visible_map;

pub use coop_synchronization::{
    COOP_SYNC_ARTIFACT, COOP_SYNC_MAX_GENERATION, COOP_SYNC_MAX_PEERS, COOP_SYNC_PROTOCOL_VERSION,
    COOP_SYNC_SCHEMA_DIGEST, COOP_SYNC_SCHEMA_SOURCE, CoopPeer, CoopPeerRole, CoopProvenance,
    CoopSyncStatus, CoopSynchronization, CoopSynchronizationError, CoopSynchronizationMessage,
};
pub use descriptor::{
    ContractManifest, DigestAlgorithm, DigestDescriptor, Provenance, VersionProfile,
};
pub use envelope::{ErrorEnvelope, ErrorMetadata, ErrorOrigin, Retryability};
pub use exact_state::{
    BLOB_DIGEST_PREFIX, BlobDigest, CANONICAL_PROFILE, CHECKPOINT_MANIFEST_DOMAIN,
    CHECKPOINT_MANIFEST_SCHEMA, CanonicalError, CanonicalValue, EXACT_CHECKPOINT_ID_PREFIX,
    EXACT_STATE_DIGEST_PREFIX, EXACT_STATE_DOMAIN, EXACT_STATE_SCHEMA, ExactCheckpointId,
    ExactStateDigest, IdentityError, MAX_CANONICAL_BYTES, MAX_DEPTH, MAX_INPUT_BYTES,
    MAX_SAFE_INTEGER, blob_digest,
};
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
pub use runtime_v4_expert::{
    RUNTIME_V4_EXPERT_ACTION_ARTIFACT, RUNTIME_V4_EXPERT_ACTION_PROTOCOL_VERSION,
    RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_ACTION_SCHEMA_SOURCE,
    RUNTIME_V4_EXPERT_ARTIFACT, RUNTIME_V4_EXPERT_PROTOCOL_VERSION,
    RUNTIME_V4_EXPERT_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_SCHEMA_SOURCE,
};
pub use runtime_v4_expert_rest_action::{
    RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT, RUNTIME_V4_EXPERT_REST_ACTION_EFFECT_WITNESS_VERSION,
    RUNTIME_V4_EXPERT_REST_ACTION_GENERATOR, RUNTIME_V4_EXPERT_REST_ACTION_MAX_GENERATION,
    RUNTIME_V4_EXPERT_REST_ACTION_MAX_IDENTITY_BYTES,
    RUNTIME_V4_EXPERT_REST_ACTION_MAX_SELECTOR_CHOICES, RUNTIME_V4_EXPERT_REST_ACTION_PROFILE,
    RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION, RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST,
    RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_SOURCE,
};
pub use seeded_run::{
    SEEDED_RUN_ARTIFACT, SEEDED_RUN_EFFECT_KIND, SEEDED_RUN_GENERATOR, SEEDED_RUN_MAX_ACTS,
    SEEDED_RUN_MAX_CONTEXT_ID_BYTES, SEEDED_RUN_MAX_CONTEXT_TEXT_BYTES, SEEDED_RUN_MAX_GENERATION,
    SEEDED_RUN_MAX_IDENTITY_BYTES, SEEDED_RUN_MAX_MODIFIERS, SEEDED_RUN_MAX_SEED_BYTES,
    SEEDED_RUN_PROTOCOL_VERSION, SEEDED_RUN_SCHEMA_DIGEST, SEEDED_RUN_SCHEMA_SOURCE,
    SeededRunCharacter, SeededRunCompatibility, SeededRunContext, SeededRunEffectWitness,
    SeededRunGameMode, SeededRunIdentityDigest, SeededRunMessage, SeededRunMessageKind,
    SeededRunMode, SeededRunObservation, SeededRunProfileBaseline, SeededRunProfileKind,
    SeededRunProvenance, SeededRunSavePolicy, SeededRunSelectionContext, SeededRunStatus,
    SeededRunValidationError,
};
pub use serialization::{canonical_json, decode_json};
pub use visible_map::{
    MapActionBinding, MapAvailability, MapCompleteness, MapContext, MapEdge, MapFreshness,
    MapMessage, MapMessageKind, MapNavigationAction, MapNode, MapPosition, MapProvenance,
    MapRoomCategory, MapSnapshot, MapTimeout, RUNTIME_MAP_V1_ARTIFACT, RUNTIME_MAP_V1_GENERATOR,
    RUNTIME_MAP_V1_MAX_ACTION_OPTION_ID_BYTES, RUNTIME_MAP_V1_MAX_BINDINGS,
    RUNTIME_MAP_V1_MAX_COORDINATE, RUNTIME_MAP_V1_MAX_EDGES, RUNTIME_MAP_V1_MAX_GENERATION,
    RUNTIME_MAP_V1_MAX_HISTORY, RUNTIME_MAP_V1_MAX_HOST_ACTION_ID_BYTES,
    RUNTIME_MAP_V1_MAX_ID_BYTES, RUNTIME_MAP_V1_MAX_JSON_DEPTH, RUNTIME_MAP_V1_MAX_MESSAGE_BYTES,
    RUNTIME_MAP_V1_MAX_NODES, RUNTIME_MAP_V1_MAX_REASON_BYTES, RUNTIME_MAP_V1_MAX_TEXT_BYTES,
    RUNTIME_MAP_V1_MAX_TIMEOUT_MILLIS, RUNTIME_MAP_V1_MIN_COORDINATE,
    RUNTIME_MAP_V1_PROTOCOL_VERSION, RUNTIME_MAP_V1_SCHEMA_DIGEST, RUNTIME_MAP_V1_SCHEMA_SOURCE,
    RUNTIME_MAP_V1_SNAPSHOT_SCHEMA_VERSION, RuntimeMapV1ActionBinding, RuntimeMapV1Availability,
    RuntimeMapV1CanonicalError, RuntimeMapV1Completeness, RuntimeMapV1Context,
    RuntimeMapV1DecodeError, RuntimeMapV1Edge, RuntimeMapV1Freshness, RuntimeMapV1Message,
    RuntimeMapV1MessageKind, RuntimeMapV1NavigationAction, RuntimeMapV1Node, RuntimeMapV1Position,
    RuntimeMapV1Provenance, RuntimeMapV1RoomCategory, RuntimeMapV1Snapshot, RuntimeMapV1Timeout,
    RuntimeMapV1ValidationError, canonical_map_message_json, canonical_map_snapshot_json,
    decode_map_message, decode_map_snapshot, decode_runtime_map_message, map_content_digest,
    map_navigation_digest, map_snapshot_digest, map_topology_digest, sha256_hex,
};

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
