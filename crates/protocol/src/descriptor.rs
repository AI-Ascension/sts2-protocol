// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::identity::QualifiedId;
use crate::{ValidationError, validate_text, validate_token};

/// The digest algorithm named by a published descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

/// A content digest descriptor; it does not calculate or authorize a release.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DigestDescriptor {
    pub algorithm: DigestAlgorithm,
    pub value: String,
}

impl DigestDescriptor {
    /// Creates a lowercase hexadecimal SHA-256 descriptor.
    pub fn sha256(value: impl Into<String>) -> Result<Self, ValidationError> {
        let descriptor = Self {
            algorithm: DigestAlgorithm::Sha256,
            value: value.into(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    /// Validates the supported digest spelling and length.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.value.len() != 64
            || !self.value.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.value.bytes().any(|byte| byte.is_ascii_uppercase())
        {
            return Err(ValidationError::InvalidDigest { field: "digest" });
        }
        Ok(())
    }
}

/// An independently versioned component profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionProfile {
    pub component: QualifiedId,
    pub version: String,
    pub profile: String,
}

impl VersionProfile {
    /// Creates a validated component/version/profile descriptor.
    pub fn new(
        component: QualifiedId,
        version: impl Into<String>,
        profile: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let descriptor = Self {
            component,
            version: version.into(),
            profile: profile.into(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    /// Validates the component and bounded profile strings.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.component.validate()?;
        validate_token("version", &self.version, 32)?;
        validate_token("profile", &self.profile, 64)
    }
}

/// Source and licensing facts for a contract or schema artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub source: String,
    pub license: String,
    pub generator: String,
    pub source_digest: DigestDescriptor,
}

impl Provenance {
    /// Validates repository-relative provenance and its digest.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_text("source", &self.source, 256)?;
        if self.source.starts_with('/')
            || self.source.starts_with('\\')
            || self.source.as_bytes().get(1) == Some(&b':')
            || self.source.contains('\\')
        {
            return Err(ValidationError::AbsolutePath { field: "source" });
        }
        if self.source.split('/').any(|part| part == "..") {
            return Err(ValidationError::ParentPath { field: "source" });
        }
        validate_token("license", &self.license, 32)?;
        validate_token("generator", &self.generator, 64)?;
        self.source_digest.validate()
    }
}

/// A digest-backed manifest for one neutral contract/schema profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractManifest {
    pub manifest_version: VersionProfile,
    pub contract: QualifiedId,
    pub schema: VersionProfile,
    pub digest: DigestDescriptor,
    pub provenance: Provenance,
    pub consumers: Vec<QualifiedId>,
}

impl ContractManifest {
    /// Validates ownership metadata and deterministic consumer ordering.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.manifest_version.validate()?;
        self.contract.validate()?;
        self.schema.validate()?;
        self.digest.validate()?;
        self.provenance.validate()?;
        if self.consumers.len() < 2 {
            return Err(ValidationError::TooFewConsumers { minimum: 2 });
        }
        for consumer in &self.consumers {
            consumer.validate()?;
        }
        for pair in self.consumers.windows(2) {
            if pair[0] == pair[1] {
                return Err(ValidationError::DuplicateConsumer);
            }
            if pair[0] > pair[1] {
                return Err(ValidationError::UnsortedConsumers);
            }
        }
        Ok(())
    }
}
