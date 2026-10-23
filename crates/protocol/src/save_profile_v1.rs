// SPDX-License-Identifier: MIT

//! Inert candidate: supplied identity and readback are not authenticated by this codec.
mod semantics;

use serde_json::Value;
use sha2::{Digest, Sha256};

pub const PROFILE: &str = "save-profile-v1";
pub const SCHEMA: &str = include_str!("../../../schemas/save-profile-v1.schema.json");
pub const SCHEMA_DIGEST: &str = "6e57edffbedec9df4f0b5bb794e7bd89790aaa1b0e2f6d021a21cc73a163190b";
pub const MAX_MESSAGE_BYTES: usize = 65_536;

/// Fixed errors never include untrusted values or host paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rejection {
    TooLarge,
    Malformed,
    UnsupportedProfile,
    InvalidBinding,
}
impl std::fmt::Display for Rejection {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{self:?}")
    }
}
impl std::error::Error for Rejection {}

/// Pure schema/relationship validation, not a lease issuer, ledger or mutation capability.
pub struct Codec {
    schema: jsonschema::Validator,
}
impl Codec {
    pub fn new() -> Result<Self, Rejection> {
        if digest(SCHEMA.as_bytes()) != SCHEMA_DIGEST {
            return Err(Rejection::UnsupportedProfile);
        }
        let schema: Value = serde_json::from_str(SCHEMA).map_err(|_| Rejection::Malformed)?;
        let schema = jsonschema::draft202012::options()
            .build(&schema)
            .map_err(|_| Rejection::Malformed)?;
        Ok(Self { schema })
    }

    /// Checks the byte limit before duplicate-preserving, exact-integer parsing.
    pub fn decode(&self, bytes: &[u8]) -> Result<Value, Rejection> {
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(Rejection::TooLarge);
        }
        let value = crate::game_information_v2::decode_unique_exact_json(bytes)
            .map_err(|_| Rejection::Malformed)?;
        self.validate(&value)?;
        Ok(value)
    }

    /// Compact UTF-8 with recursively lexicographically sorted keys and retained array order.
    pub fn encode(&self, value: &Value) -> Result<Vec<u8>, Rejection> {
        self.validate(value)?;
        // The closed schema bounds all members and arrays below the message ceiling.
        let bytes = serde_json::to_vec(value).map_err(|_| Rejection::Malformed)?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(Rejection::TooLarge);
        }
        Ok(bytes)
    }

    pub fn validate(&self, value: &Value) -> Result<(), Rejection> {
        if value["protocol_version"] != PROFILE || value["schema_digest"] != SCHEMA_DIGEST {
            return Err(Rejection::UnsupportedProfile);
        }
        if !self.schema.is_valid(value) || !integer_values(value) {
            return Err(Rejection::Malformed);
        }
        semantics::validate(value)
    }

    /// Verify a reply or recovery request against the retained original operation request.
    /// A caller must retain the original; validating an isolated reply cannot establish origin.
    pub fn bind_operation(&self, original: &Value, next: &Value) -> Result<(), Rejection> {
        self.validate(original)?;
        self.validate(next)?;
        if original["kind"] != "operation_request"
            || !matches!(
                next["kind"].as_str(),
                Some("operation_result" | "lookup_request" | "lookup_result")
            )
            || original["intent"] != next["intent"]
            || original["intent_digest"] != next["intent_digest"]
        {
            return Err(Rejection::InvalidBinding);
        }
        Ok(())
    }

    /// Read-only lookup results bind both the retained operation and the current lookup lease.
    pub fn bind_lookup(
        &self,
        original: &Value,
        lookup: &Value,
        reply: &Value,
    ) -> Result<(), Rejection> {
        self.bind_operation(original, lookup)?;
        self.bind_operation(original, reply)?;
        if lookup["kind"] != "lookup_request"
            || !matches!(
                reply["kind"].as_str(),
                Some("lookup_result" | "operation_result")
            )
            || lookup["current_lease_id"] != reply["current_lease_id"]
        {
            return Err(Rejection::InvalidBinding);
        }
        Ok(())
    }

    pub fn bind_discovery(&self, request: &Value, reply: &Value) -> Result<(), Rejection> {
        self.validate(request)?;
        self.validate(reply)?;
        if request["kind"] != "discover_request"
            || reply["kind"] != "discover_response"
            || request["owner"] != reply["observation"]["owner"]
        {
            return Err(Rejection::InvalidBinding);
        }
        Ok(())
    }
}

/// Domain-separated digest of the complete original intent (including original lease and scope).
/// Caller-supplied values must pass the intent schema before use as an admitted operation.
pub fn intent_digest(intent: &Value) -> Result<String, Rejection> {
    let bytes = serde_json::to_vec(intent).map_err(|_| Rejection::Malformed)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(Rejection::TooLarge);
    }
    let mut input = b"sts2-save-profile-intent-v1\n".to_vec();
    input.extend(bytes);
    Ok(digest(&input))
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn integer_values(value: &Value) -> bool {
    match value {
        Value::Number(number) => number.as_u64().is_some_and(|n| n <= 9_007_199_254_740_991),
        Value::Array(values) => values.iter().all(integer_values),
        Value::Object(values) => values.values().all(integer_values),
        _ => true,
    }
}
