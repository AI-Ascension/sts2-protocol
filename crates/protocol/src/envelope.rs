// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::identity::CorrelationMetadata;
use crate::lifecycle::OperationStatus;
use crate::{ValidationError, validate_text};

/// The abstract origin class of an error mapping.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorOrigin {
    Decode,
    Contract,
    Boundary,
    Internal,
}

/// Retry guidance that does not itself authorize a retry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Retryability {
    Never,
    Safe,
    AfterRefresh,
    Unknown,
}

/// Sanitized error metadata whose code authority remains at the origin owner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorMetadata {
    pub origin: crate::QualifiedId,
    pub code: crate::QualifiedId,
    pub retryability: Retryability,
    pub operation: OperationStatus,
    pub safe_message: String,
}

impl ErrorMetadata {
    /// Validates qualified error identity and the bounded safe message.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.origin.validate()?;
        self.code.validate()?;
        validate_text("safe_message", &self.safe_message, 256)
    }
}

/// A neutral error envelope that preserves correlation and uncertain outcome.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorEnvelope {
    pub correlation: CorrelationMetadata,
    pub error: ErrorMetadata,
}

impl ErrorEnvelope {
    /// Validates the envelope without mapping or retrying the originating operation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.correlation.validate()?;
        self.error.validate()
    }
}
