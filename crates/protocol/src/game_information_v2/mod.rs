// SPDX-License-Identifier: MIT

//! Candidate inert schema and semantic validation, with caller-owned capture evidence.
//! This validates supplied evidence, not its authenticity or host-side read-only behavior.

mod accounting;
mod capture;
mod page;
mod parse;
mod query;
mod unsupported_request;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const PROFILE: &str = "game-information-query-v2";
pub const SCHEMA_DIGEST: &str = "63cc23e2c75f29aab21b058279c6aa17c5fd4e65b8163a7c33f77fa746d2fb6f";
pub const SCHEMA: &str = include_str!("../../../../schemas/game-information-query-v2.schema.json");
pub const MAX_RESPONSE_BYTES: usize = 262_144;
pub const MAX_REQUEST_BYTES: usize = 16_384;
const STATIC_FIELDS: &[&str] = &["description", "display_name", "rest_mode", "rest_option_id"];
const LIVE_FIELDS: &[&str] = &[
    "description",
    "display_name",
    "rest_action",
    "rest_eligible",
    "rest_mode",
    "rest_option_id",
    "rest_selector",
];
type Result<T = ()> = std::result::Result<T, Rejection>;

/// Fixed, bounded protocol rejection without input contents or paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    Malformed,
    UnsupportedVersion,
    MissingCapability,
    UnsupportedField,
    UnsupportedFilter,
    UnsupportedProjection,
    InvalidIdentity,
    DeniedScope,
    StaleSnapshot,
    StaleCursor,
    MixedGeneration,
    ResultLimitExceeded,
    AmbiguousId,
    ReadOnlyViolation,
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for Rejection {}

/// Authority identity is supplied by the boundary that authenticated it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    pub producer_lease: String,
    pub instance_id: String,
    pub run_id: String,
    pub epoch: u64,
}

/// An already captured option, never a request to extract or mutate the host.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RestEntry {
    pub definition_ref: Value,
    pub instance_ref: Value,
    pub option_id: String,
    pub fields: Vec<Value>,
    /// A separately captured eligibility fact; None does not imply false.
    pub eligibility: Option<bool>,
    /// Already selected owner IDs. Never exposed as selector candidates.
    pub selected_choices: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RestCapture {
    pub content_manifest_id: String,
    pub locale: String,
    /// Null for a static-only registry; never invent a live room to expose definitions.
    pub context: Value,
    pub revision: String,
    pub fully_classified: bool,
    pub text_index_available: bool,
    pub legal_actions: Vec<Value>,
    pub sources: Vec<Value>,
    pub entries: Vec<RestEntry>,
}

/// Owner registry input. Normalized query removes only cursor.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CursorBinding {
    pub cursor: String,
    pub profile: String,
    pub schema_digest: String,
    pub producer_lease: String,
    pub occurrence_revision: String,
    pub normalized_query: Value,
    pub offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationContext {
    pub capabilities: Value,
    pub authority: Authority,
    pub rest: Option<RestCapture>,
    pub cursors: Vec<CursorBinding>,
}

/// Compiles only the embedded schema. No caller schema, transport or resolver is accepted.
pub struct Validator {
    envelope: jsonschema::Validator,
    request_shape: jsonschema::Validator,
    fragments: BTreeMap<&'static str, jsonschema::Validator>,
}

impl Validator {
    pub fn new() -> Result<Self> {
        let schema: Value = serde_json::from_str(SCHEMA).map_err(|_| Rejection::Malformed)?;
        let envelope = jsonschema::validator_for(&schema).map_err(|_| Rejection::Malformed)?;
        let request_shape = unsupported_request::compile(&schema)?;
        let mut fragments = BTreeMap::new();
        for name in [
            "capabilities",
            "rest_read_context",
            "item",
            "identity",
            "cursor",
            "rest_action_ref",
            "source",
            "locale",
        ] {
            let mut fragment = schema.clone();
            fragment
                .as_object_mut()
                .ok_or(Rejection::Malformed)?
                .remove("oneOf");
            fragment["$ref"] = Value::String(format!("#/$defs/{name}"));
            fragments.insert(
                name,
                jsonschema::validator_for(&fragment).map_err(|_| Rejection::Malformed)?,
            );
        }
        Ok(Self {
            envelope,
            request_shape,
            fragments,
        })
    }

    /// Rejects duplicate members before Value materialization and enforces raw byte limits.
    pub fn decode(&self, bytes: &[u8], context: &ValidationContext) -> Result<Value> {
        require(
            bytes.len() <= MAX_RESPONSE_BYTES,
            Rejection::ResultLimitExceeded,
        )?;
        let value = parse::unique(bytes)?;
        if value["kind"] == "query_request" {
            require(
                bytes.len() <= MAX_REQUEST_BYTES,
                Rejection::ResultLimitExceeded,
            )?;
        }
        self.validate(&value, context)?;
        Ok(value)
    }

    /// Values must come from a duplicate-rejecting parser; prefer decode for untrusted bytes.
    pub fn validate(&self, value: &Value, context: &ValidationContext) -> Result {
        require(
            value["protocol_version"] == PROFILE && value["schema_digest"] == SCHEMA_DIGEST,
            Rejection::UnsupportedVersion,
        )?;
        require(
            value["result"]["read_only"] != false,
            Rejection::ReadOnlyViolation,
        )?;
        if !self.envelope.is_valid(value) {
            return Err(self.unsupported_request(value));
        }
        self.validate_context(context)?;
        let maximum = if value["kind"] == "query_request" {
            MAX_REQUEST_BYTES
        } else {
            MAX_RESPONSE_BYTES
        };
        require(
            size(value)? <= maximum.min(number(&context.capabilities["max_message_bytes"])?),
            Rejection::ResultLimitExceeded,
        )?;
        match value["kind"].as_str() {
            Some("capabilities_response") => require(
                value["capabilities"] == context.capabilities,
                Rejection::MissingCapability,
            ),
            Some("query_request") => self.query(&value["query"], context),
            Some("query_response") => {
                self.query(&value["query"], context)?;
                self.response(value, context)
            }
            Some("error_response") => {
                // Refused/stale queries need not satisfy current capability admission.
                let deterministic = !matches!(
                    value["error"]["code"].as_str(),
                    Some("stale_snapshot" | "stale_cursor" | "missing_capability")
                );
                require(
                    !deterministic || value["error"]["retryable"] == false,
                    Rejection::Malformed,
                )
            }
            _ => Err(Rejection::Malformed),
        }
    }

    fn fragment(&self, name: &str, value: &Value) -> Result {
        require(
            self.fragments
                .get(name)
                .is_some_and(|schema| schema.is_valid(value)),
            Rejection::Malformed,
        )
    }
}

fn require(condition: bool, error: Rejection) -> Result {
    if condition { Ok(()) } else { Err(error) }
}
fn number(value: &Value) -> Result<usize> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(Rejection::Malformed)
}
fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().ok_or(Rejection::Malformed)
}
fn size(value: &Value) -> Result<usize> {
    // serde_json's Map uses lexicographically ordered keys (preserve_order is disabled).
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .map_err(|_| Rejection::Malformed)
}
pub fn normalized_query(query: &Value) -> Result<Value> {
    let mut query = query.as_object().ok_or(Rejection::Malformed)?.clone();
    query.remove("cursor");
    Ok(Value::Object(query))
}
fn fence(reference: &Value, authority: &Authority) -> bool {
    reference["instance_id"] == authority.instance_id
        && reference["run_id"] == authority.run_id
        && reference["epoch"] == authority.epoch
}
