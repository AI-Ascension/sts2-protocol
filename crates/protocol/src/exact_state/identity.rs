// SPDX-License-Identifier: MIT

//! Typed identity wrappers that keep exact-state, checkpoint, and blob namespaces apart.

use sha2::{Digest, Sha256};

use super::{BLOB_DIGEST_PREFIX, EXACT_CHECKPOINT_ID_PREFIX, EXACT_STATE_DIGEST_PREFIX, to_hex};

/// Returns the domain-free SHA-256 digest of raw artifact bytes.
#[must_use]
pub fn blob_digest(bytes: &[u8]) -> BlobDigest {
    BlobDigest(format!(
        "{BLOB_DIGEST_PREFIX}{}",
        to_hex(&Sha256::digest(bytes))
    ))
}

/// A domain-separated exact-state content identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExactStateDigest(pub(super) String);

impl ExactStateDigest {
    /// Parses a serialized identifier, rejecting other identity namespaces.
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        parse_hex_identity(value, EXACT_STATE_DIGEST_PREFIX).map(Self)
    }

    /// Returns the serialized identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A domain-separated immutable checkpoint identifier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExactCheckpointId(pub(super) String);

impl ExactCheckpointId {
    /// Parses a serialized identifier, rejecting other identity namespaces.
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        parse_hex_identity(value, EXACT_CHECKPOINT_ID_PREFIX).map(Self)
    }

    /// Returns the serialized identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A digest of exact artifact bytes under the blob namespace.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BlobDigest(pub(super) String);

impl BlobDigest {
    /// Parses a serialized blob digest.
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        parse_hex_identity(value, BLOB_DIGEST_PREFIX).map(Self)
    }

    /// Returns the serialized digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Rejection reasons for a serialized identity value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    /// The namespace/version prefix is not the expected one.
    Prefix,
    /// The digest body is not exactly 64 lowercase hexadecimal characters.
    Hex,
}

fn parse_hex_identity(value: &str, prefix: &str) -> Result<String, IdentityError> {
    let hex = value.strip_prefix(prefix).ok_or(IdentityError::Prefix)?;
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(IdentityError::Hex);
    }
    Ok(value.to_owned())
}
