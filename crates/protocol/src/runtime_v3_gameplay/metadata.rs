// SPDX-License-Identifier: MIT

//! Inert artifact provenance and request identity for gameplay messages.

use super::{
    RUNTIME_V3_GAMEPLAY_ARTIFACT, RUNTIME_V3_GAMEPLAY_GENERATOR,
    RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION, RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST,
    RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE,
};

/// Inert release metadata carried by each message.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayMetadata {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeV3GameplayProvenance,
}

impl Default for RuntimeV3GameplayMetadata {
    fn default() -> Self {
        Self {
            protocol_version: RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION.to_owned(),
            schema_digest: RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST.to_owned(),
            provenance: RuntimeV3GameplayProvenance::default(),
        }
    }
}

/// Provenance for the owner-local release-like artifact.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for RuntimeV3GameplayProvenance {
    fn default() -> Self {
        Self {
            artifact: RUNTIME_V3_GAMEPLAY_ARTIFACT.to_owned(),
            source: RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE.to_owned(),
            generator: RUNTIME_V3_GAMEPLAY_GENERATOR.to_owned(),
        }
    }
}

/// Identity preserved across gateway, MCP, host, and harness boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeV3GameplayContext {
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
}

impl RuntimeV3GameplayContext {
    /// Creates context metadata without granting authorization.
    #[must_use]
    pub fn new(
        correlation_id: impl Into<String>,
        instance_id: impl Into<String>,
        session_id: impl Into<String>,
        lease_id: impl Into<String>,
        lease_epoch: u64,
    ) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            instance_id: instance_id.into(),
            session_id: session_id.into(),
            lease_id: lease_id.into(),
            lease_epoch,
        }
    }
}
