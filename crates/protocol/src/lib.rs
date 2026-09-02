// SPDX-License-Identifier: MIT

mod descriptor;
mod envelope;
mod identity;
mod lifecycle;
mod poc;
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
    POC_ARTIFACT, POC_GENERATOR, POC_MAX_SETTLED_EFFECTS, POC_MAX_UNITS, POC_PROTOCOL_VERSION,
    POC_SCHEMA_DIGEST, POC_SCHEMA_SOURCE, PocAction, PocActionResult, PocMessage, PocMessageKind,
    PocMetadata, PocObservation, PocProvenance, PocStatus, PocValidationError,
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
