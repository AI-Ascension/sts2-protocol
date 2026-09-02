// SPDX-License-Identifier: MIT

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Serializes a contract value to compact deterministic JSON.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Decodes JSON for a caller that will apply the value's explicit validation method.
pub fn decode_json<T: DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(text)
}
