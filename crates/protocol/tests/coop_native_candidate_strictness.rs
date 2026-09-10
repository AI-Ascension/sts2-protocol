// SPDX-License-Identifier: MIT

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::fmt;

const OBSERVATION: &str =
    include_str!("../../../artifacts/coop-native-v1/golden/observation-response.json");

struct StrictValue;

impl<'de> DeserializeSeed<'de> for StrictValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StrictValue {
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
        StrictValue.deserialize(deserializer)
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(StrictValue)? {
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
            values.insert(key, map.next_value_seed(StrictValue)?);
        }
        Ok(Value::Object(values))
    }
}

fn strict_json(text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = deserializer.deserialize_any(StrictValue)?;
    deserializer.end()?;
    Ok(value)
}

#[test]
fn duplicate_members_are_rejected_at_any_depth() {
    let duplicate_top = OBSERVATION.replacen(
        "\"kind\":\"observation\"",
        "\"kind\":\"observation\",\"kind\":\"observation\"",
        1,
    );
    assert!(strict_json(&duplicate_top).is_err());
    let duplicate_nested = OBSERVATION.replacen(
        "\"role\":\"local\"",
        "\"role\":\"local\",\"role\":\"local\"",
        1,
    );
    assert!(strict_json(&duplicate_nested).is_err());
}
