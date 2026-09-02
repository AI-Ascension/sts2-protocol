// SPDX-License-Identifier: MIT

use super::{PocAction, PocMessageKind, PocObservation, PocStatus};

struct RequiredOption<T>(Option<T>);

impl<'de, T> serde::Deserialize<'de> for RequiredOption<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        if value.is_null() {
            Ok(Self(None))
        } else {
            T::deserialize(value)
                .map(|value| Self(Some(value)))
                .map_err(<D::Error as serde::de::Error>::custom)
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PocMessageWire {
    protocol_version: String,
    schema_digest: String,
    provenance: super::super::metadata::PocProvenance,
    correlation_id: String,
    instance_id: String,
    generation: u64,
    kind: PocMessageKind,
    observation: RequiredOption<PocObservation>,
    action: RequiredOption<PocAction>,
    status: RequiredOption<PocStatus>,
    error_code: RequiredOption<String>,
}

impl<'de> serde::Deserialize<'de> for super::PocMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = PocMessageWire::deserialize(deserializer)?;
        Ok(Self {
            protocol_version: wire.protocol_version,
            schema_digest: wire.schema_digest,
            provenance: wire.provenance,
            correlation_id: wire.correlation_id,
            instance_id: wire.instance_id,
            generation: wire.generation,
            kind: wire.kind,
            observation: wire.observation.0,
            action: wire.action.0,
            status: wire.status.0,
            error_code: wire.error_code.0,
        })
    }
}
