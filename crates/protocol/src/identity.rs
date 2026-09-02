// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::{ValidationError, validate_token};

/// An opaque value qualified by the authority that issued it.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct QualifiedId {
    pub namespace: String,
    pub value: String,
}

impl QualifiedId {
    /// Creates a qualified identifier after checking stable lexical bounds.
    pub fn new(
        namespace: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let identifier = Self {
            namespace: namespace.into(),
            value: value.into(),
        };
        identifier.validate()?;
        Ok(identifier)
    }

    /// Validates the namespace and opaque value without granting authority.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_token("namespace", &self.namespace, 64)?;
        validate_token("value", &self.value, 128)
    }
}

/// Non-authoritative identity references carried between owners.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IdentityMetadata {
    pub subject: QualifiedId,
    pub session: Option<QualifiedId>,
}

impl IdentityMetadata {
    /// Validates all identity references.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.subject.validate()?;
        if let Some(session) = &self.session {
            session.validate()?;
        }
        Ok(())
    }
}

/// Correlation references that do not define request or operation authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CorrelationMetadata {
    pub request: QualifiedId,
    pub trace: Option<QualifiedId>,
    pub operation: Option<QualifiedId>,
}

impl CorrelationMetadata {
    /// Validates correlation references and their namespaces.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.request.validate()?;
        if let Some(trace) = &self.trace {
            trace.validate()?;
        }
        if let Some(operation) = &self.operation {
            operation.validate()?;
        }
        Ok(())
    }
}

/// Opaque parentage and artifact references for lineage without experiment semantics.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LineageMetadata {
    pub root: QualifiedId,
    pub parent: Option<QualifiedId>,
    pub artifact: Option<QualifiedId>,
}

impl LineageMetadata {
    /// Validates lineage references without interpreting their owners.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.root.validate()?;
        if let Some(parent) = &self.parent {
            parent.validate()?;
        }
        if let Some(artifact) = &self.artifact {
            artifact.validate()?;
        }
        Ok(())
    }
}

/// A monotonically increasing sequence within one named stream.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SequenceMetadata {
    pub stream: QualifiedId,
    pub number: u64,
}

impl SequenceMetadata {
    /// Validates the stream identity; sequence ordering belongs to its owner.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.stream.validate()
    }
}
