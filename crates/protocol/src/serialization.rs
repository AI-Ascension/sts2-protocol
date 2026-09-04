// SPDX-License-Identifier: MIT

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Deserializes an explicitly present nullable member without erasing duplicate keys.
pub(crate) fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer)
}

/// Serializes a contract value to compact deterministic JSON.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Decodes JSON for a caller that will apply the value's explicit validation method.
pub fn decode_json<T: DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(text)
}
