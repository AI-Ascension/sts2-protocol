// SPDX-License-Identifier: MIT

/// Version of the deterministic proof-of-concept contract.
pub const POC_PROTOCOL_VERSION: &str = "poc-v1";
/// Release-like artifact name recorded in every POC message.
pub const POC_ARTIFACT: &str = "sts2-protocol/poc-v1";
/// Repository-relative source of the POC schema.
pub const POC_SCHEMA_SOURCE: &str = "schemas/poc-v1.schema.json";
/// Generator recorded for the hand-authored POC schema.
pub const POC_GENERATOR: &str = "hand-authored";
/// SHA-256 of the canonical schema source bytes.
pub const POC_SCHEMA_DIGEST: &str =
    "242b8f9233e915a55ea8d2e72ca476c1258169a67e62de72ee5aed848a6a0a19";
/// Maximum number of units represented by the bounded fake observation/action.
pub const POC_MAX_UNITS: u16 = 8;
/// Maximum settled-effect count represented by the bounded fake observation.
pub const POC_MAX_SETTLED_EFFECTS: u16 = 4;
/// Maximum generation that remains exact in common JSON number implementations.
pub const POC_MAX_GENERATION: u64 = 9_007_199_254_740_991;

/// The schema and provenance metadata carried by every POC message.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PocMetadata {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: PocProvenance,
}

impl PocMetadata {
    /// Creates metadata for the supplied schema digest.
    #[must_use]
    pub fn new(schema_digest: impl Into<String>) -> Self {
        Self {
            protocol_version: POC_PROTOCOL_VERSION.to_owned(),
            schema_digest: schema_digest.into(),
            provenance: PocProvenance::default(),
        }
    }

    /// Validates the fixed contract and release-like provenance fields.
    pub fn validate(&self) -> Result<(), super::PocValidationError> {
        if self.protocol_version != POC_PROTOCOL_VERSION
            || self.schema_digest != POC_SCHEMA_DIGEST
            || !is_digest(&self.schema_digest)
        {
            return Err(super::PocValidationError::Metadata);
        }
        self.provenance.validate()
    }
}

/// Provenance that identifies the inert release-like artifact without granting authority.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PocProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for PocProvenance {
    fn default() -> Self {
        Self {
            artifact: POC_ARTIFACT.to_owned(),
            source: POC_SCHEMA_SOURCE.to_owned(),
            generator: POC_GENERATOR.to_owned(),
        }
    }
}

impl PocProvenance {
    fn validate(&self) -> Result<(), super::PocValidationError> {
        if self.artifact != POC_ARTIFACT
            || self.source != POC_SCHEMA_SOURCE
            || self.generator != POC_GENERATOR
        {
            return Err(super::PocValidationError::Provenance);
        }
        Ok(())
    }
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
