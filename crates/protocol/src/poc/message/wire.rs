// SPDX-License-Identifier: MIT

use super::{PocAction, PocMessageKind, PocObservation, PocStatus};

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
    #[serde(deserialize_with = "crate::serialization::required_option")]
    observation: Option<PocObservation>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    action: Option<PocAction>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    status: Option<PocStatus>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    error_code: Option<String>,
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
            observation: wire.observation,
            action: wire.action,
            status: wire.status,
            error_code: wire.error_code,
        })
    }
}
