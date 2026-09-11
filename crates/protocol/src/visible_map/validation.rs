// SPDX-License-Identifier: MIT

use super::model::{
    RuntimeMapV1Availability, RuntimeMapV1Completeness, RuntimeMapV1Message,
    RuntimeMapV1MessageKind, RuntimeMapV1Snapshot,
};
use super::{
    RUNTIME_MAP_V1_ARTIFACT, RUNTIME_MAP_V1_GENERATOR, RUNTIME_MAP_V1_MAX_GENERATION,
    RUNTIME_MAP_V1_MAX_ID_BYTES, RUNTIME_MAP_V1_MAX_REASON_BYTES, RUNTIME_MAP_V1_MAX_TEXT_BYTES,
    RUNTIME_MAP_V1_MAX_TIMEOUT_MILLIS, RUNTIME_MAP_V1_PROTOCOL_VERSION,
    RUNTIME_MAP_V1_SCHEMA_DIGEST, RUNTIME_MAP_V1_SCHEMA_SOURCE,
    RUNTIME_MAP_V1_SNAPSHOT_SCHEMA_VERSION, valid_identity, valid_text,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMapV1ValidationError {
    Metadata,
    Provenance,
    InvalidIdentity,
    InvalidText,
    GenerationBounds,
    CoordinateBounds,
    CollectionBounds,
    ReasonRequired,
    AvailabilityShape,
    InvalidNode,
    DuplicateNode,
    DuplicateEdge,
    UnknownEdgeEndpoint,
    InvalidEdge,
    CyclicGraph,
    InvalidPosition,
    InvalidHistory,
    InvalidTerminal,
    DuplicateTerminal,
    DuplicateBinding,
    DuplicateActionOption,
    UnknownBindingNode,
    InvalidActionOption,
    InvalidBinding,
    TimeoutBounds,
    RequestShape,
    ResponseShape,
}

impl std::fmt::Display for RuntimeMapV1ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Metadata => "runtime-map-v1 metadata is unsupported",
            Self::Provenance => "runtime-map-v1 provenance is unsupported",
            Self::InvalidIdentity => "runtime-map-v1 identity is invalid",
            Self::InvalidText => "runtime-map-v1 text is invalid",
            Self::GenerationBounds => "runtime-map-v1 generation is outside the bound",
            Self::CoordinateBounds => "runtime-map-v1 coordinate is outside the bound",
            Self::CollectionBounds => "runtime-map-v1 collection exceeds its bound",
            Self::ReasonRequired => {
                "runtime-map-v1 reason is required for an unavailable or incomplete projection"
            }
            Self::AvailabilityShape => {
                "runtime-map-v1 availability and completeness are inconsistent"
            }
            Self::InvalidNode => "runtime-map-v1 node is invalid",
            Self::DuplicateNode => "runtime-map-v1 node IDs must be unique",
            Self::DuplicateEdge => "runtime-map-v1 directed edges must be unique",
            Self::UnknownEdgeEndpoint => "runtime-map-v1 edge endpoint is unknown",
            Self::InvalidEdge => "runtime-map-v1 edge is invalid",
            Self::CyclicGraph => "runtime-map-v1 graph must be acyclic",
            Self::InvalidPosition => "runtime-map-v1 position is invalid",
            Self::InvalidHistory => "runtime-map-v1 visible history is invalid",
            Self::InvalidTerminal => "runtime-map-v1 terminal node is invalid",
            Self::DuplicateTerminal => "runtime-map-v1 terminal node IDs must be unique",
            Self::DuplicateBinding => "runtime-map-v1 action bindings must be unique",
            Self::DuplicateActionOption => {
                "runtime-map-v1 serialized action option IDs must be unique"
            }
            Self::UnknownBindingNode => "runtime-map-v1 action binding references an unknown node",
            Self::InvalidActionOption => "runtime-map-v1 serialized action option ID is invalid",
            Self::InvalidBinding => "runtime-map-v1 action binding is invalid",
            Self::TimeoutBounds => "runtime-map-v1 timeout metadata is outside the bound",
            Self::RequestShape => "runtime-map-v1 request shape is invalid",
            Self::ResponseShape => "runtime-map-v1 response shape is invalid",
        })
    }
}

impl std::error::Error for RuntimeMapV1ValidationError {}

impl RuntimeMapV1Snapshot {
    /// Validates the complete snapshot without consulting a host or clock.
    pub fn validate(&self) -> Result<(), RuntimeMapV1ValidationError> {
        if self.generation > RUNTIME_MAP_V1_MAX_GENERATION {
            return Err(RuntimeMapV1ValidationError::GenerationBounds);
        }
        if self.schema_version != RUNTIME_MAP_V1_SNAPSHOT_SCHEMA_VERSION {
            return Err(RuntimeMapV1ValidationError::Metadata);
        }
        for value in [
            &self.state_id,
            &self.schema_version,
            &self.projection_version,
        ] {
            if !valid_identity(value, RUNTIME_MAP_V1_MAX_ID_BYTES) {
                return Err(RuntimeMapV1ValidationError::InvalidIdentity);
            }
        }
        for value in [&self.game_build, &self.mod_version] {
            if !valid_text(value, RUNTIME_MAP_V1_MAX_TEXT_BYTES) {
                return Err(RuntimeMapV1ValidationError::InvalidText);
            }
        }
        for value in [&self.map_instance_id, &self.scope_id]
            .into_iter()
            .flatten()
        {
            if !valid_identity(value, RUNTIME_MAP_V1_MAX_ID_BYTES) {
                return Err(RuntimeMapV1ValidationError::InvalidIdentity);
            }
        }
        if let Some(reason) = &self.reason
            && !valid_text(reason, RUNTIME_MAP_V1_MAX_REASON_BYTES)
        {
            return Err(RuntimeMapV1ValidationError::InvalidText);
        }
        let reason_required = self.availability != RuntimeMapV1Availability::Available
            || self.completeness != RuntimeMapV1Completeness::Complete;
        if reason_required != self.reason.is_some() {
            return Err(RuntimeMapV1ValidationError::ReasonRequired);
        }
        if self.availability != RuntimeMapV1Availability::Available
            && self.completeness == RuntimeMapV1Completeness::Complete
        {
            return Err(RuntimeMapV1ValidationError::AvailabilityShape);
        }
        if self.availability == RuntimeMapV1Availability::Available
            && (self.map_instance_id.is_none() || self.act_id.is_none() || self.scope_id.is_none())
        {
            return Err(RuntimeMapV1ValidationError::AvailabilityShape);
        }
        self.validate_graph()
    }

    fn validate_graph(&self) -> Result<(), RuntimeMapV1ValidationError> {
        super::graph::validate_graph(self)
    }
}

impl RuntimeMapV1Message {
    /// Validates metadata, identity, bounded timeout metadata, and fixed request/response shape.
    pub fn validate(&self) -> Result<(), RuntimeMapV1ValidationError> {
        if self.protocol_version != RUNTIME_MAP_V1_PROTOCOL_VERSION
            || self.schema_digest != RUNTIME_MAP_V1_SCHEMA_DIGEST
        {
            return Err(RuntimeMapV1ValidationError::Metadata);
        }
        let provenance = &self.provenance;
        if provenance.artifact != RUNTIME_MAP_V1_ARTIFACT
            || provenance.source != RUNTIME_MAP_V1_SCHEMA_SOURCE
            || provenance.generator != RUNTIME_MAP_V1_GENERATOR
        {
            return Err(RuntimeMapV1ValidationError::Provenance);
        }
        for value in [
            &self.correlation_id,
            &self.instance_id,
            &self.session_id,
            &self.lease_id,
        ] {
            if !valid_identity(value, RUNTIME_MAP_V1_MAX_ID_BYTES) {
                return Err(RuntimeMapV1ValidationError::InvalidIdentity);
            }
        }
        if self.lease_epoch > RUNTIME_MAP_V1_MAX_GENERATION
            || self.generation > RUNTIME_MAP_V1_MAX_GENERATION
        {
            return Err(RuntimeMapV1ValidationError::GenerationBounds);
        }
        if let Some(timeout) = self.timeout
            && (timeout.timeout_millis == 0
                || timeout.timeout_millis > RUNTIME_MAP_V1_MAX_TIMEOUT_MILLIS
                || timeout.elapsed_millis > timeout.timeout_millis)
        {
            return Err(RuntimeMapV1ValidationError::TimeoutBounds);
        }
        match self.kind {
            RuntimeMapV1MessageKind::SnapshotRequest => {
                if self.snapshot.is_some() || self.timeout.is_some() {
                    return Err(RuntimeMapV1ValidationError::RequestShape);
                }
            }
            RuntimeMapV1MessageKind::SnapshotResponse => {
                let Some(snapshot) = &self.snapshot else {
                    return Err(RuntimeMapV1ValidationError::ResponseShape);
                };
                if snapshot.generation != self.generation {
                    return Err(RuntimeMapV1ValidationError::ResponseShape);
                }
                snapshot.validate()?;
            }
        }
        Ok(())
    }

    /// Validates that this envelope is a request.
    pub fn validate_request(&self) -> Result<(), RuntimeMapV1ValidationError> {
        self.validate()?;
        (self.kind == RuntimeMapV1MessageKind::SnapshotRequest)
            .then_some(())
            .ok_or(RuntimeMapV1ValidationError::RequestShape)
    }

    /// Validates that this envelope is a response.
    pub fn validate_response(&self) -> Result<(), RuntimeMapV1ValidationError> {
        self.validate()?;
        (self.kind == RuntimeMapV1MessageKind::SnapshotResponse)
            .then_some(())
            .ok_or(RuntimeMapV1ValidationError::ResponseShape)
    }
}
