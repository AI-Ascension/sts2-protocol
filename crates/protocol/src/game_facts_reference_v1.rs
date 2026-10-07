// SPDX-License-Identifier: MIT

//! Additive inert contract for typed rule-input references; it does not execute rules.

mod codec;
mod types;
mod validation;

pub use types::*;

/// Stable artifact identifier for the proposed, transport-neutral profile.
pub const GAME_FACTS_REFERENCE_V1_ARTIFACT: &str = "sts2-protocol/game-facts-reference-v1";
/// Profile identifier carried by every message.
pub const GAME_FACTS_REFERENCE_V1_PROFILE: &str = "game-facts-reference-v1";
/// Maximum UTF-8 bytes for any message in this new profile.
pub const GAME_FACTS_REFERENCE_V1_MAX_MESSAGE_BYTES: usize = 262_144;
/// Embedded standalone schema. Character bounds here complement typed byte bounds.
pub const GAME_FACTS_REFERENCE_V1_SCHEMA: &str =
    include_str!("../../../schemas/game-facts-reference-v1.schema.json");
/// SHA-256 of the standalone schema bytes, pinned by the distributable artifact.
pub const GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST: &str =
    "3065a2ff96e6e5af628b3d5cb43b8f2db4232ae4c4dc71324b973419908a2985";

/// Fixed local codec refusal without input text, filesystem paths, or host exceptions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameFactsReferenceV1Rejection {
    Malformed,
    UnsupportedVersion,
    UnknownKind,
    ResultLimitExceeded,
    InvalidIdentity,
    InvalidBounds,
    InvalidRequest,
    InvalidBinding,
}

impl std::fmt::Display for GameFactsReferenceV1Rejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl std::error::Error for GameFactsReferenceV1Rejection {}

impl GameFactsReferenceV1Rejection {
    /// Returns the stable lowercase wire code for this refusal.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::UnsupportedVersion => "unsupported_version",
            Self::UnknownKind => "unknown_kind",
            Self::ResultLimitExceeded => "result_limit_exceeded",
            Self::InvalidIdentity => "invalid_identity",
            Self::InvalidBounds => "invalid_bounds",
            Self::InvalidRequest => "invalid_request",
            Self::InvalidBinding => "invalid_binding",
        }
    }
}

/// Bounded duplicate-rejecting codec for the candidate profile.
pub struct GameFactsReferenceV1Codec;

impl GameFactsReferenceV1Codec {
    /// Decodes untrusted raw bytes with duplicate, exact-number, schema, and bound checks.
    pub fn decode(bytes: &[u8]) -> Result<Message, GameFactsReferenceV1Rejection> {
        codec::decode(bytes)
    }

    /// Validates a typed value and emits deterministic compact JSON within the byte ceiling.
    pub fn encode(message: &Message) -> Result<Vec<u8>, GameFactsReferenceV1Rejection> {
        codec::encode(message)
    }

    /// Applies schema and cross-field checks to an already typed value.
    pub fn validate(message: &Message) -> Result<(), GameFactsReferenceV1Rejection> {
        codec::validate(message)
    }
}

/// Binds a response with an echoed query to the caller's original request.
pub fn validate_response_against_request(
    request: &Message,
    response: &Message,
) -> Result<(), GameFactsReferenceV1Rejection> {
    codec::validate(request)?;
    codec::validate(response)?;
    validation::validate_response_against_request(request, response)
}
