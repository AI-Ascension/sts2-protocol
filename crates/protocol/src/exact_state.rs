// SPDX-License-Identifier: MIT

//! Inert exact-state identity contract for the `asc-jcs-state-v1` profile.
//!
//! This module owns the restricted canonical byte profile, the domain-separated
//! SHA-256 identities, and typed wrappers that keep exact state, checkpoint, and
//! blob identities distinguishable from existing observation/action identities.
//! It carries no game rules, host state, persistence, or mutation authority; the
//! protocol target describes the encodings and the game-mod owns the state.

mod identity;
mod parse;

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

use parse::{parse_canonical, parse_raw};

pub use identity::{BlobDigest, ExactCheckpointId, ExactStateDigest, IdentityError, blob_digest};

/// Canonicalization profile bound into every exact identity.
pub const CANONICAL_PROFILE: &str = "asc-jcs-state-v1";
/// Schema identifier for the exact-state payload envelope.
pub const EXACT_STATE_SCHEMA: &str = "ascension.exact_state.v1";
/// Schema identifier for the immutable checkpoint-manifest envelope.
pub const CHECKPOINT_MANIFEST_SCHEMA: &str = "ascension.checkpoint_manifest.v1";
/// Domain separator for the exact-state content identity.
pub const EXACT_STATE_DOMAIN: &[u8] = b"AI-ASCENSION/EXACT-STATE/v1\0";
/// Domain separator for the immutable checkpoint-manifest identity.
pub const CHECKPOINT_MANIFEST_DOMAIN: &[u8] = b"AI-ASCENSION/CHECKPOINT/v1\0";
/// Serialized prefix of an exact-state digest value.
pub const EXACT_STATE_DIGEST_PREFIX: &str = "asc-state:v1:sha256:";
/// Serialized prefix of an immutable checkpoint identifier.
pub const EXACT_CHECKPOINT_ID_PREFIX: &str = "asc-checkpoint:v1:sha256:";
/// Serialized prefix of a raw artifact blob digest.
pub const BLOB_DIGEST_PREFIX: &str = "sha256:";
/// Maximum accepted raw UTF-8 input length in bytes.
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
/// Maximum canonical UTF-8 output length in bytes.
pub const MAX_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
/// Maximum number of nested containers; a root container counts as one.
pub const MAX_DEPTH: usize = 64;
/// Maximum magnitude of a profile integer.
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// Rejection reasons for the restricted canonical profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalError {
    /// Raw input exceeds [`MAX_INPUT_BYTES`].
    InputBytes,
    /// Canonical output exceeds [`MAX_CANONICAL_BYTES`].
    CanonicalBytes,
    /// A single string exceeds [`MAX_CANONICAL_BYTES`].
    StringBytes,
    /// Input is not well-formed UTF-8.
    Utf8,
    /// Input begins with a UTF-8 byte-order mark.
    Bom,
    /// Nesting exceeds [`MAX_DEPTH`].
    Depth,
    /// Input is not valid restricted JSON.
    Syntax,
    /// An object key is not an ASCII schema field name.
    Key,
    /// An object repeats a decoded key.
    DuplicateKey,
    /// An integer is outside the safe range or longer than sixteen digits.
    IntegerRange,
    /// A number token is a float, exponent, `-0`, or has leading zeros.
    NumberToken,
    /// Input is valid JSON but not the canonical encoding of its own value.
    NonCanonical,
    /// Content follows the first top-level value.
    TrailingData,
    /// A native value is outside the canonical value domain.
    UnsupportedType,
}

impl CanonicalError {
    /// Returns the stable machine-readable code for this rejection.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputBytes => "input_bytes",
            Self::CanonicalBytes => "canonical_bytes",
            Self::StringBytes => "string_bytes",
            Self::Utf8 => "utf8",
            Self::Bom => "bom",
            Self::Depth => "depth",
            Self::Syntax => "syntax",
            Self::Key => "key",
            Self::DuplicateKey => "duplicate_key",
            Self::IntegerRange => "integer_range",
            Self::NumberToken => "number_token",
            Self::NonCanonical => "noncanonical",
            Self::TrailingData => "trailing_data",
            Self::UnsupportedType => "unsupported_type",
        }
    }
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CanonicalError {}

/// A validated value inside the restricted canonical profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalValue {
    /// JSON `null`.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// A safe-range JSON integer.
    Int(i64),
    /// A Unicode scalar string.
    String(String),
    /// An order-preserving JSON array.
    Array(Vec<CanonicalValue>),
    /// A JSON object whose keys are ASCII schema field names.
    Object(BTreeMap<String, CanonicalValue>),
}

impl CanonicalValue {
    /// Parses raw UTF-8 bytes under the restricted profile without requiring canonical input.
    pub fn parse(input: &[u8]) -> Result<Self, CanonicalError> {
        parse_raw(input)
    }

    /// Parses raw UTF-8 text under the restricted profile without requiring canonical input.
    pub fn parse_str(input: &str) -> Result<Self, CanonicalError> {
        parse_raw(input.as_bytes())
    }

    /// Parses input that must already be the canonical encoding of its own value.
    pub fn parse_canonical(input: &[u8]) -> Result<Self, CanonicalError> {
        parse_canonical(input)
    }

    /// Returns the canonical UTF-8 bytes, with no trailing newline.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut out = Vec::new();
        write_value(self, 0, &mut out)?;
        Ok(out)
    }

    /// Returns the lowercase hexadecimal canonical bytes.
    pub fn to_canonical_hex(&self) -> Result<String, CanonicalError> {
        Ok(to_hex(&self.to_canonical_bytes()?))
    }

    /// Returns the domain-separated exact-state content identity.
    pub fn exact_state_digest(&self) -> Result<ExactStateDigest, CanonicalError> {
        let bytes = self.to_canonical_bytes()?;
        Ok(ExactStateDigest(domain_digest(
            EXACT_STATE_DIGEST_PREFIX,
            EXACT_STATE_DOMAIN,
            &bytes,
        )))
    }

    /// Returns the domain-separated immutable checkpoint identifier.
    pub fn exact_checkpoint_id(&self) -> Result<ExactCheckpointId, CanonicalError> {
        let bytes = self.to_canonical_bytes()?;
        Ok(ExactCheckpointId(domain_digest(
            EXACT_CHECKPOINT_ID_PREFIX,
            CHECKPOINT_MANIFEST_DOMAIN,
            &bytes,
        )))
    }
}

fn domain_digest(prefix: &str, domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    format!("{prefix}{}", to_hex(&hasher.finalize()))
}

pub(super) fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn write_value(
    value: &CanonicalValue,
    depth: usize,
    out: &mut Vec<u8>,
) -> Result<(), CanonicalError> {
    match value {
        CanonicalValue::Null => out.extend_from_slice(b"null"),
        CanonicalValue::Bool(flag) => out.extend_from_slice(if *flag { b"true" } else { b"false" }),
        CanonicalValue::Int(number) => {
            if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(number) {
                return Err(CanonicalError::IntegerRange);
            }
            out.extend_from_slice(number.to_string().as_bytes());
        }
        CanonicalValue::String(text) => write_string(text, out)?,
        CanonicalValue::Array(items) => {
            if depth >= MAX_DEPTH {
                return Err(CanonicalError::Depth);
            }
            out.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_value(item, depth + 1, out)?;
            }
            out.push(b']');
        }
        CanonicalValue::Object(entries) => {
            if depth >= MAX_DEPTH {
                return Err(CanonicalError::Depth);
            }
            out.push(b'{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_string(key, out)?;
                out.push(b':');
                write_value(item, depth + 1, out)?;
            }
            out.push(b'}');
        }
    }
    if out.len() > MAX_CANONICAL_BYTES {
        return Err(CanonicalError::CanonicalBytes);
    }
    Ok(())
}

fn write_string(value: &str, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    if value.len() > MAX_CANONICAL_BYTES {
        return Err(CanonicalError::StringBytes);
    }
    out.push(b'"');
    for character in value.chars() {
        match character {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            '\r' => out.extend_from_slice(b"\\r"),
            control if (control as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", control as u32).as_bytes());
            }
            other => {
                let mut buffer = [0_u8; 4];
                out.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    out.push(b'"');
    Ok(())
}
