// SPDX-License-Identifier: MIT

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use super::{
    DENIED_SCOPE, GOLDENS, INVALID_IDENTITY, MALFORMED, MISSING_CAPABILITY, MIXED_BINDING, PROFILE,
    SCHEMA_DIGEST, SOURCE_SCHEMA, STALE_SNAPSHOT, UNSUPPORTED_VERSION,
};

const KINDS: &[&str] = &[
    "lookup_binding_discovery_response",
    "lookup_binding_observation_response",
    "error_response",
];

const STATES: &[&str] = &[
    "not_yet_observed",
    "observed",
    "reobserve_required",
    "reobserve_exhausted",
];

const ERROR_CODES: &[&str] = &[
    "unsupported_version",
    "invalid_identity",
    "denied_scope",
    "missing_capability",
    "stale_snapshot",
    "mixed_binding",
    "reobserve_unavailable",
    "malformed",
];

struct UniqueValue;

impl<'de> DeserializeSeed<'de> for UniqueValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for UniqueValue {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object members")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        UniqueValue.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(UniqueValue)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(<A::Error as serde::de::Error>::custom(format!(
                    "duplicate object member {key}"
                )));
            }
            values.insert(key, map.next_value_seed(UniqueValue)?);
        }
        Ok(Value::Object(values))
    }
}

pub(crate) fn parse_unique_json(text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = deserializer.deserialize_any(UniqueValue)?;
    deserializer.end()?;
    Ok(value)
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn binding_id_of(binding: &Value) -> String {
    let members = [
        ("agent_id", &binding["scope"]["agent_id"]),
        ("authority_epoch", &binding["authority_epoch"]),
        ("content_manifest_id", &binding["content_manifest_id"]),
        ("episode_id", &binding["scope"]["episode_id"]),
        ("game_profile", &binding["game_profile"]),
        ("locale", &binding["locale"]),
        ("project_id", &binding["scope"]["project_id"]),
        ("run_id", &binding["scope"]["run_id"]),
    ];
    let identity = members
        .into_iter()
        .map(|(member, value)| (member.to_owned(), value.clone()))
        .collect::<Map<String, Value>>();
    let canonical = serde_json::to_string(&Value::Object(identity)).expect("identity JSON");
    digest(canonical.as_bytes())
}

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn read_json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

pub(crate) fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

pub(crate) fn golden(name: &str) -> Value {
    GOLDENS
        .iter()
        .find_map(|(golden_name, text)| {
            (*golden_name == name).then(|| serde_json::from_str(text).expect("golden JSON"))
        })
        .unwrap_or_else(|| panic!("named lookup-binding golden exists: {name}"))
}

pub(crate) fn schema_validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles")
}

pub(crate) fn rejection_after(
    context: &Value,
    bind: impl FnOnce(&mut Value),
) -> Option<&'static str> {
    let mut document = golden("observation-response");
    bind(&mut document);
    semantic_rejection(&document, context)
}

fn behind(observed: &Value, retained: &Value) -> bool {
    match (observed.as_u64(), retained.as_u64()) {
        (Some(observed), Some(retained)) => observed < retained,
        _ => false,
    }
}

pub(crate) fn semantic_rejection(document: &Value, context: &Value) -> Option<&'static str> {
    if !KINDS.contains(&document["kind"].as_str().unwrap_or_default()) {
        return Some(MALFORMED);
    }
    if document["protocol_version"].as_str() != Some(PROFILE)
        || document["schema_digest"].as_str() != Some(SCHEMA_DIGEST)
    {
        return Some(UNSUPPORTED_VERSION);
    }
    if document["binding"].is_object() {
        if !document["binding"]["scope"].is_object() {
            return Some(MALFORMED);
        }
        if binding_id_of(&document["binding"]) != document["binding"]["binding_id"] {
            return Some(INVALID_IDENTITY);
        }
        if document["binding"]["scope"] != context["scope"] {
            return Some(DENIED_SCOPE);
        }
        if document["binding"]["instance_id"] != context["instance_id"] {
            return Some(DENIED_SCOPE);
        }
    }
    if document["discovery"].is_object() {
        let required = &document["discovery"]["required_capabilities"]["profile"];
        let negotiated = context["negotiated_capabilities"].as_array();
        if !negotiated.is_some_and(|capabilities| capabilities.contains(required)) {
            return Some(MISSING_CAPABILITY);
        }
    }
    if !document["observation"].is_null() {
        if document["observation"]["binding_id"] != document["binding"]["binding_id"] {
            return Some(MIXED_BINDING);
        }
        if behind(
            &document["observation"]["state_generation"],
            &context["retained_state_generation"],
        ) {
            return Some(STALE_SNAPSHOT);
        }
    }
    state_rejection(document, context)
}

fn state_rejection(document: &Value, context: &Value) -> Option<&'static str> {
    match document["kind"].as_str().unwrap_or_default() {
        "lookup_binding_discovery_response" => {
            if document["binding"].is_null()
                || !document["discovery"].is_object()
                || !document["observation"].is_null()
                || !document["error"].is_null()
                || document["discovery"]["observation_state"].as_str() != Some("not_yet_observed")
                || !document["discovery"]["reobserve"].is_null()
            {
                return Some(MALFORMED);
            }
            None
        }
        _ => observation_shape(document, context),
    }
}

fn observation_shape(document: &Value, context: &Value) -> Option<&'static str> {
    if document["kind"].as_str() == Some("error_response") {
        return error_shape(document);
    }
    let state = document["discovery"]["observation_state"]
        .as_str()
        .unwrap_or_default();
    if !STATES.contains(&state) {
        return Some(MALFORMED);
    }
    if document["binding"].is_null()
        || !document["discovery"].is_object()
        || !document["error"].is_null()
    {
        return Some(MALFORMED);
    }
    let observation = &document["observation"];
    let reobserve = &document["discovery"]["reobserve"];
    match state {
        "observed" => {
            if observation.is_null() || !reobserve.is_null() {
                return Some(MALFORMED);
            }
        }
        "reobserve_required" | "reobserve_exhausted" => {
            if !observation.is_null() || !reobserve.is_object() {
                return Some(MALFORMED);
            }
            let attempts = reobserve["attempts"].as_u64().unwrap_or(0);
            if (state == "reobserve_exhausted" && attempts < 1)
                || (!context["retained_observation_id"].is_null()
                    && reobserve["supersedes_observation_id"] != context["retained_observation_id"])
            {
                return Some(MALFORMED);
            }
        }
        _ => return Some(MALFORMED),
    }
    None
}

fn error_shape(document: &Value) -> Option<&'static str> {
    let code = document["error"]["code"].as_str().unwrap_or_default();
    if !ERROR_CODES.contains(&code) || !document["observation"].is_null() {
        return Some(MALFORMED);
    }
    if document["binding"].is_null() != document["discovery"].is_null() {
        return Some(MALFORMED);
    }
    if code == "reobserve_unavailable"
        && document["discovery"]["observation_state"].as_str() != Some("reobserve_exhausted")
    {
        return Some(MALFORMED);
    }
    None
}
