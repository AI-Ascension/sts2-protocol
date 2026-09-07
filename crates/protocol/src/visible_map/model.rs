// SPDX-License-Identifier: MIT

use super::required_nullable;

/// Whether the host could provide a map projection for this snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMapV1Availability {
    Available,
    Unavailable,
    NotObservable,
    Unsupported,
}

/// Whether the graph is known to contain every visible map relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMapV1Completeness {
    Complete,
    Incomplete,
    Unknown,
}

/// Whether the snapshot describes the current map or an explicitly retained prior view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMapV1Freshness {
    Current,
    Historical,
}

/// A player-visible room category. `Unknown` preserves an honest projection boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMapV1RoomCategory {
    Unknown,
    Start,
    Monster,
    Elite,
    Rest,
    Shop,
    Event,
    Treasure,
    Boss,
    Other,
}

/// The player position represented by a map snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeMapV1Position {
    PreStart {},
    Current { node_id: String },
    Unavailable {},
}

/// The read-only navigation payload associated with one host legal action.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeMapV1NavigationAction {
    SelectMapNode { node_id: String },
}

/// One stable graph node in the host-owned projection.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Node {
    pub id: String,
    pub row: i32,
    pub column: i32,
    pub category: RuntimeMapV1RoomCategory,
    pub visited: bool,
}

/// One directed relation between two stable graph node IDs.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Edge {
    pub from: String,
    pub to: String,
}

/// A generation-bound action binding. The graph ID and host action ID remain separate namespaces.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1ActionBinding {
    pub graph_node_id: String,
    pub host_action_id: String,
    pub action: RuntimeMapV1NavigationAction,
}

/// A bounded, host-owned, player-visible full-map projection.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Snapshot {
    pub state_id: String,
    pub generation: u64,
    pub schema_version: String,
    pub projection_version: String,
    pub game_build: String,
    pub mod_version: String,
    #[serde(deserialize_with = "required_nullable")]
    pub map_instance_id: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub act_id: Option<u32>,
    #[serde(deserialize_with = "required_nullable")]
    pub scope_id: Option<String>,
    pub availability: RuntimeMapV1Availability,
    pub completeness: RuntimeMapV1Completeness,
    pub freshness: RuntimeMapV1Freshness,
    #[serde(deserialize_with = "required_nullable")]
    pub reason: Option<String>,
    pub nodes: Vec<RuntimeMapV1Node>,
    pub edges: Vec<RuntimeMapV1Edge>,
    pub position: RuntimeMapV1Position,
    pub history: Vec<String>,
    pub terminal_node_ids: Vec<String>,
    pub bindings: Vec<RuntimeMapV1ActionBinding>,
}

/// A fixed read-only request/response discriminator.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMapV1MessageKind {
    SnapshotRequest,
    SnapshotResponse,
}

/// Release-like provenance carried by each map message.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Provenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for RuntimeMapV1Provenance {
    fn default() -> Self {
        Self {
            artifact: super::RUNTIME_MAP_V1_ARTIFACT.to_owned(),
            source: super::RUNTIME_MAP_V1_SCHEMA_SOURCE.to_owned(),
            generator: super::RUNTIME_MAP_V1_GENERATOR.to_owned(),
        }
    }
}

/// Envelope metadata shared by map requests and responses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeMapV1Context {
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
}

impl RuntimeMapV1Context {
    /// Creates identity metadata without granting authorization.
    #[must_use]
    pub fn new(
        correlation_id: impl Into<String>,
        instance_id: impl Into<String>,
        session_id: impl Into<String>,
        lease_id: impl Into<String>,
        lease_epoch: u64,
    ) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            instance_id: instance_id.into(),
            session_id: session_id.into(),
            lease_id: lease_id.into(),
            lease_epoch,
        }
    }
}

/// Bounded timeout information. Values are metadata; this crate never reads a clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Timeout {
    pub timeout_millis: u32,
    pub elapsed_millis: u32,
}

/// Complete map request/response envelope.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMapV1Message {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeMapV1Provenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: RuntimeMapV1MessageKind,
    #[serde(deserialize_with = "required_nullable")]
    pub snapshot: Option<RuntimeMapV1Snapshot>,
    #[serde(deserialize_with = "required_nullable")]
    pub timeout: Option<RuntimeMapV1Timeout>,
}

impl RuntimeMapV1Message {
    /// Creates a fixed-shape read-only snapshot request.
    #[must_use]
    pub fn snapshot_request(context: RuntimeMapV1Context, generation: u64) -> Self {
        Self {
            protocol_version: super::RUNTIME_MAP_V1_PROTOCOL_VERSION.to_owned(),
            schema_digest: super::RUNTIME_MAP_V1_SCHEMA_DIGEST.to_owned(),
            provenance: RuntimeMapV1Provenance::default(),
            correlation_id: context.correlation_id,
            instance_id: context.instance_id,
            session_id: context.session_id,
            lease_id: context.lease_id,
            lease_epoch: context.lease_epoch,
            generation,
            kind: RuntimeMapV1MessageKind::SnapshotRequest,
            snapshot: None,
            timeout: None,
        }
    }

    /// Creates a fixed-shape snapshot response bound to the snapshot generation.
    #[must_use]
    pub fn snapshot_response(
        context: RuntimeMapV1Context,
        snapshot: RuntimeMapV1Snapshot,
        timeout: Option<RuntimeMapV1Timeout>,
    ) -> Self {
        Self {
            protocol_version: super::RUNTIME_MAP_V1_PROTOCOL_VERSION.to_owned(),
            schema_digest: super::RUNTIME_MAP_V1_SCHEMA_DIGEST.to_owned(),
            provenance: RuntimeMapV1Provenance::default(),
            correlation_id: context.correlation_id,
            instance_id: context.instance_id,
            session_id: context.session_id,
            lease_id: context.lease_id,
            lease_epoch: context.lease_epoch,
            generation: snapshot.generation,
            kind: RuntimeMapV1MessageKind::SnapshotResponse,
            snapshot: Some(snapshot),
            timeout,
        }
    }
}

/// Names used by consumers that refer to the profile as `Map` rather than `RuntimeMapV1`.
pub type MapAvailability = RuntimeMapV1Availability;
pub type MapCompleteness = RuntimeMapV1Completeness;
pub type MapFreshness = RuntimeMapV1Freshness;
pub type MapRoomCategory = RuntimeMapV1RoomCategory;
pub type MapPosition = RuntimeMapV1Position;
pub type MapNavigationAction = RuntimeMapV1NavigationAction;
pub type MapNode = RuntimeMapV1Node;
pub type MapEdge = RuntimeMapV1Edge;
pub type MapActionBinding = RuntimeMapV1ActionBinding;
pub type MapSnapshot = RuntimeMapV1Snapshot;
pub type MapMessageKind = RuntimeMapV1MessageKind;
pub type MapProvenance = RuntimeMapV1Provenance;
pub type MapContext = RuntimeMapV1Context;
pub type MapTimeout = RuntimeMapV1Timeout;
pub type MapMessage = RuntimeMapV1Message;
