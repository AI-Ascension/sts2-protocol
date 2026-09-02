// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::identity::{CorrelationMetadata, IdentityMetadata, LineageMetadata, SequenceMetadata};
use crate::{ValidationError, validate_text};

/// A neutral lifecycle projection; the resource owner defines legal transitions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Created,
    Starting,
    Ready,
    Busy,
    Degraded,
    Stopping,
    Stopped,
    Failed,
    Expired,
}

/// The observable status of work associated with metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    NotAccepted,
    Accepted,
    Settled,
    Rejected,
    Cancelled,
    Unknown,
}

/// Lifecycle metadata without transition authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LifecycleMetadata {
    pub state: LifecycleState,
    pub operation: OperationStatus,
}

/// The owner clock used for a relative deadline budget.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockKind {
    WallTimeObservation,
    MonotonicOwner,
}

/// A bounded deadline budget; portable monotonic instants are intentionally excluded.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeadlineMetadata {
    pub clock: ClockKind,
    pub owner: crate::QualifiedId,
    pub budget_ms: u64,
}

impl DeadlineMetadata {
    /// Validates the deadline owner while leaving expiry enforcement to that owner.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.owner.validate()
    }
}

/// Cancellation state propagated as metadata, not as an execution mechanism.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationState {
    NotRequested,
    Requested,
    Acknowledged,
    Rejected,
}

/// Cancellation metadata with a sanitized optional reason.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CancellationMetadata {
    pub state: CancellationState,
    pub reason: Option<String>,
    pub sequence: u64,
}

impl CancellationMetadata {
    /// Validates the optional human-readable reason without interpreting it.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(reason) = &self.reason {
            validate_text("cancellation.reason", reason, 256)?;
        }
        Ok(())
    }
}

/// The complete neutral metadata seam shared by accepted consumers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NeutralMetadata {
    pub identity: IdentityMetadata,
    pub correlation: CorrelationMetadata,
    pub lineage: LineageMetadata,
    pub sequence: SequenceMetadata,
    pub lifecycle: LifecycleMetadata,
    pub deadline: Option<DeadlineMetadata>,
    pub cancellation: CancellationMetadata,
}

impl NeutralMetadata {
    /// Validates every nested metadata value without contacting a boundary.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.identity.validate()?;
        self.correlation.validate()?;
        self.lineage.validate()?;
        self.sequence.validate()?;
        if let Some(deadline) = &self.deadline {
            deadline.validate()?;
        }
        self.cancellation.validate()
    }
}
