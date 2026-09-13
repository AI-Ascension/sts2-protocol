// SPDX-License-Identifier: MIT

use super::super::*;
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Map;
use std::fmt;

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

pub(crate) fn canonical_bytes(value: &Value) -> usize {
    serde_json::to_vec(value)
        .expect("JSON values are serializable")
        .len()
}

pub(super) fn string_in(value: &Value, choices: &[&str]) -> bool {
    value
        .as_str()
        .is_some_and(|string| choices.contains(&string))
}

pub(super) fn array_contains(array: &Value, needle: &str) -> bool {
    array
        .as_array()
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(needle)))
}

pub(super) fn safe_integer(value: &Value) -> bool {
    value
        .as_i64()
        .is_some_and(|number| (0..=9_007_199_254_740_991).contains(&number))
        || value
            .as_u64()
            .is_some_and(|number| number <= 9_007_199_254_740_991)
}

pub(super) fn bounds_error(value: &Value) -> bool {
    value.as_i64().is_some_and(|number| number < 0)
        || value
            .as_u64()
            .is_some_and(|number| number > 9_007_199_254_740_991)
}

pub(super) fn is_definition_ref(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object
        .get("content_manifest_id")
        .and_then(Value::as_str)
        .is_some_and(|string| !string.is_empty())
        && object
            .get("entity_kind")
            .is_some_and(|kind| string_in(kind, ENTITY_KINDS))
        && object
            .get("namespaced_id")
            .and_then(Value::as_str)
            .is_some_and(|string| !string.is_empty())
        && object.get("variant").is_some_and(|variant| {
            variant.is_null() || variant.as_str().is_some_and(|string| !string.is_empty())
        })
}

pub(super) fn is_instance_ref(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object
        .get("instance_id")
        .and_then(Value::as_str)
        .is_some_and(|string| !string.is_empty())
        && object
            .get("run_id")
            .and_then(Value::as_str)
            .is_some_and(|string| !string.is_empty())
        && object.get("epoch").is_some_and(safe_integer)
        && object
            .get("entity_kind")
            .is_some_and(|kind| string_in(kind, ENTITY_KINDS))
        && object
            .get("entity_id")
            .and_then(Value::as_str)
            .is_some_and(|string| !string.is_empty())
}

pub(super) fn field_error(field: &Value) -> Option<&'static str> {
    let Some(object) = field.as_object() else {
        return Some(MALFORMED);
    };
    if !object.get("name").is_some_and(|name| {
        name.as_str()
            .is_some_and(|name| FIELD_NAMES.contains(&name))
    }) || !object.get("kind").is_some_and(|kind| {
        kind.as_str()
            .is_some_and(|kind| FIELD_KINDS.contains(&kind))
    }) || !object.get("availability").is_some_and(|availability| {
        availability
            .as_str()
            .is_some_and(|availability| AVAILABILITIES.contains(&availability))
    }) || !object
        .get("source")
        .and_then(Value::as_object)
        .is_some_and(|source| {
            source.get("kind").and_then(Value::as_str).is_some() && source.contains_key("ref")
        })
    {
        return Some(MALFORMED);
    }

    if object["availability"] == "available" {
        if !object.get("reason").is_some_and(Value::is_null)
            || object.get("value").is_none_or(Value::is_null)
        {
            return Some(MALFORMED);
        }
        let kind = object["kind"].as_str().expect("field kind checked");
        let value = object.get("value").expect("available value checked");
        let valid = match kind {
            "boolean" => value.is_boolean(),
            "definition_ref" => is_definition_ref(value),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "instance_ref" => is_instance_ref(value),
            "text" => value.as_str().is_some_and(|string| !string.is_empty()),
            "text_list" => value.as_array().is_some_and(|values| {
                values.len() <= 64
                    && values
                        .iter()
                        .all(|value| value.as_str().is_some_and(|string| !string.is_empty()))
            }),
            _ => false,
        };
        if kind == "integer"
            && (value
                .as_i64()
                .is_some_and(|number| !(-2_147_483_648..=2_147_483_647).contains(&number))
                || value.as_u64().is_some_and(|number| number > 2_147_483_647))
        {
            return Some(INVALID_BOUNDS);
        }
        return valid.then_some(()).map_or(Some(MALFORMED), |_| None);
    }

    (object.get("value").is_some_and(Value::is_null)
        && object
            .get("reason")
            .and_then(Value::as_str)
            .is_some_and(|reason| !reason.is_empty()))
    .then_some(())
    .map_or(Some(MALFORMED), |_| None)
}

pub(crate) fn text_bytes(items: &Value) -> usize {
    items.as_array().map_or(0, |items| {
        items
            .iter()
            .flat_map(|item| item["fields"].as_array().into_iter().flatten())
            .filter(|field| field["availability"] == "available")
            .map(|field| match field["kind"].as_str() {
                Some("text") => field["value"].as_str().map_or(0, str::len),
                Some("text_list") => field["value"].as_array().map_or(0, |values| {
                    values.iter().filter_map(Value::as_str).map(str::len).sum()
                }),
                _ => 0,
            })
            .sum()
    })
}

pub(super) fn valid_limit_object(limits: &Value) -> bool {
    ["page_items", "item_bytes", "page_bytes", "text_bytes"]
        .iter()
        .all(|key| limits[key].as_u64().is_some_and(|number| number > 0))
}

pub(super) fn limit_error(limits: &Value, capabilities: &Value) -> Option<&'static str> {
    if !valid_limit_object(limits) {
        return Some(MALFORMED);
    }
    for key in ["page_items", "item_bytes", "page_bytes", "text_bytes"] {
        if limits[key].as_u64().unwrap_or(0) > capabilities["limits"][key].as_u64().unwrap_or(0) {
            return Some(RESULT_LIMIT_EXCEEDED);
        }
    }
    None
}

pub(crate) fn binding_value(query: &Value) -> Value {
    json!({
        "query_kind": query["query_kind"],
        "entity_kind": query["entity_kind"],
        "target": query["target"],
        "filters": query["filters"],
        "projection": query["projection"],
        "detail_level": query["detail_level"],
        "binding": query["binding"],
        "fields": query["fields"],
        "parent_observation": query["parent_observation"],
        "limits": query["limits"],
    })
}

pub(super) fn query_error(query: &Value, context: &SemanticContext) -> Option<&'static str> {
    let Some(object) = query.as_object() else {
        return Some(MALFORMED);
    };
    if !string_in(&object["query_kind"], QUERY_KINDS)
        || !string_in(&object["entity_kind"], ENTITY_KINDS)
    {
        return Some(UNKNOWN_KIND);
    }
    if !string_in(&object["projection"], PROJECTIONS)
        || !array_contains(
            &context.capabilities["projections"],
            object["projection"].as_str().unwrap_or(""),
        )
        || !string_in(&object["detail_level"], DETAIL_LEVELS)
        || !array_contains(
            &context.capabilities["detail_levels"],
            object["detail_level"].as_str().unwrap_or(""),
        )
    {
        return Some(UNSUPPORTED_PROJECTION);
    }
    let Some(fields) = object.get("fields").and_then(Value::as_array) else {
        return Some(MALFORMED);
    };
    let mut seen_fields = Vec::new();
    for field in fields {
        let Some(name) = field.as_str() else {
            return Some(UNSUPPORTED_FIELD);
        };
        if !FIELD_NAMES.contains(&name)
            || !array_contains(&context.capabilities["fields"], name)
            || seen_fields.contains(&name)
        {
            return Some(UNSUPPORTED_FIELD);
        }
        seen_fields.push(name);
    }
    let Some(binding) = object.get("binding").and_then(Value::as_object) else {
        return Some(MALFORMED);
    };
    let mode = binding.get("mode").and_then(Value::as_str);
    if !matches!(mode, Some("static" | "live")) {
        return Some(MALFORMED);
    }
    if mode == Some("live")
        && context.capabilities["snapshot_policy"]["supports_live"] != Value::Bool(true)
    {
        return Some(MISSING_CAPABILITY);
    }
    if mode == Some("static") {
        if binding["instance_ref"] != Value::Null
            || binding["snapshot_ref"] != Value::Null
            || object["parent_observation"] != Value::Null
            || object["target"]["instance_ref"] != Value::Null
        {
            return Some(MALFORMED);
        }
    } else if bounds_error(&binding["instance_ref"]["epoch"])
        || bounds_error(&binding["snapshot_ref"]["instance_ref"]["epoch"])
        || bounds_error(&object["target"]["instance_ref"]["epoch"])
        || bounds_error(&binding["snapshot_ref"]["state_generation"])
    {
        return Some(INVALID_BOUNDS);
    } else if !is_instance_ref(&binding["instance_ref"])
        || !binding["snapshot_ref"].is_object()
        || !is_instance_ref(&binding["snapshot_ref"]["instance_ref"])
        || !safe_integer(&binding["snapshot_ref"]["state_generation"])
        || !object["parent_observation"].is_object()
        || !is_instance_ref(&object["target"]["instance_ref"])
    {
        return Some(MALFORMED);
    }
    if let Some(error) = limit_error(&object["limits"], &context.capabilities) {
        return Some(error);
    }
    if object["query_kind"] == "search"
        && object["filters"]["display_name"] != Value::Null
        && object["filters"]["namespaced_ids"]
            .as_array()
            .is_some_and(Vec::is_empty)
        && object["filters"]["definition_refs"]
            .as_array()
            .is_some_and(Vec::is_empty)
        && object["filters"]["instance_ids"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Some(AMBIGUOUS_ID);
    }
    if object["cursor"] != Value::Null {
        let Some(cursor) = object["cursor"].as_str() else {
            return Some(MALFORMED);
        };
        let expected = context
            .cursor_bindings
            .iter()
            .find_map(|(known, binding)| (known == cursor).then_some(binding));
        if expected.is_none_or(|expected| expected != &binding_value(query)) {
            return Some(STALE_CURSOR);
        }
    }
    if mode == Some("live") {
        let fences_match = values_equal(
            &binding["instance_ref"],
            &binding["snapshot_ref"]["instance_ref"],
        ) && values_equal(
            &binding["instance_ref"],
            &object["target"]["instance_ref"],
        ) && values_equal(
            &object["parent_observation"]["instance_ref"],
            &binding["instance_ref"],
        ) && values_equal(
            &object["parent_observation"]["snapshot_ref"],
            &binding["snapshot_ref"],
        ) && object["parent_observation"]["state_generation"]
            == binding["snapshot_ref"]["state_generation"];
        if !fences_match {
            return if object["cursor"] != Value::Null {
                Some(STALE_CURSOR)
            } else {
                Some(STALE_SNAPSHOT)
            };
        }
    }
    None
}

pub(crate) fn values_equal(left: &Value, right: &Value) -> bool {
    left == right
}
