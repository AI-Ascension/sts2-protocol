// SPDX-License-Identifier: MIT

use super::{Rejection, Result};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::fmt;

struct Unique;

impl<'de> DeserializeSeed<'de> for Unique {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        decoder: D,
    ) -> std::result::Result<Value, D::Error> {
        decoder.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Unique {
    type Value = Value;
    fn expecting(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("bounded JSON with unique members and normalized integers")
    }
    fn visit_bool<E>(self, value: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E>(self, value: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_u64<E>(self, value: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_str<E>(self, value: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }
    fn visit_string<E>(self, value: String) -> std::result::Result<Value, E> {
        Ok(Value::String(value))
    }
    fn visit_unit<E>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> std::result::Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = input.next_element_seed(Unique)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> std::result::Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = input.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom("duplicate member"));
            }
            values.insert(key, input.next_value_seed(Unique)?);
        }
        Ok(Value::Object(values))
    }
}

pub(super) fn unique(bytes: &[u8]) -> Result<Value> {
    let normalized = super::numeric::normalize(bytes)?;
    let mut decoder = serde_json::Deserializer::from_slice(&normalized);
    let value = Unique
        .deserialize(&mut decoder)
        .map_err(|_| Rejection::Malformed)?;
    decoder.end().map_err(|_| Rejection::Malformed)?;
    Ok(value)
}
