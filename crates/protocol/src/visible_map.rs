// SPDX-License-Identifier: MIT

mod decode;
mod digest;
mod graph;
mod model;
mod validation;

pub use decode::{
    RuntimeMapV1DecodeError, decode_map_message, decode_map_snapshot, decode_runtime_map_message,
};
pub use digest::{
    RuntimeMapV1CanonicalError, canonical_map_message_json, canonical_map_snapshot_json,
    map_content_digest, map_navigation_digest, map_snapshot_digest, map_topology_digest,
    sha256_hex,
};
pub use model::{
    MapActionBinding, MapAvailability, MapCompleteness, MapContext, MapEdge, MapFreshness,
    MapMessage, MapMessageKind, MapNavigationAction, MapNode, MapPosition, MapProvenance,
    MapRoomCategory, MapSnapshot, MapTimeout, RuntimeMapV1ActionBinding, RuntimeMapV1Availability,
    RuntimeMapV1Completeness, RuntimeMapV1Context, RuntimeMapV1Edge, RuntimeMapV1Freshness,
    RuntimeMapV1Message, RuntimeMapV1MessageKind, RuntimeMapV1NavigationAction, RuntimeMapV1Node,
    RuntimeMapV1Position, RuntimeMapV1Provenance, RuntimeMapV1RoomCategory, RuntimeMapV1Snapshot,
    RuntimeMapV1Timeout,
};
pub use validation::RuntimeMapV1ValidationError;

/// Versioned, read-only host map projection profile.
pub const RUNTIME_MAP_V1_PROTOCOL_VERSION: &str = "runtime-map-v1";
/// Release-like artifact identity for the profile.
pub const RUNTIME_MAP_V1_ARTIFACT: &str = "sts2-protocol/runtime-map-v1";
/// Normative source schema path.
pub const RUNTIME_MAP_V1_SCHEMA_SOURCE: &str = "schemas/runtime-map-v1.schema.json";
/// Generator recorded by the hand-authored artifact.
pub const RUNTIME_MAP_V1_GENERATOR: &str = "hand-authored";
/// Schema identity carried by every snapshot independently of producer projection version.
pub const RUNTIME_MAP_V1_SNAPSHOT_SCHEMA_VERSION: &str = "visible-map-v1";
/// Filled from the checked-in normative schema bytes.
pub const RUNTIME_MAP_V1_SCHEMA_DIGEST: &str =
    "ceab0d2dfc471d1ec36d12edaf4654b8c7fdced06548bf47265e11c63f98115b";

/// Maximum JSON-safe generation and lease epoch.
pub const RUNTIME_MAP_V1_MAX_GENERATION: u64 = 9_007_199_254_740_991;
/// Maximum number of nodes in one bounded map projection.
pub const RUNTIME_MAP_V1_MAX_NODES: usize = 256;
/// Maximum number of directed edges in one bounded map projection.
pub const RUNTIME_MAP_V1_MAX_EDGES: usize = 1_024;
/// Maximum number of generation-bound action bindings.
pub const RUNTIME_MAP_V1_MAX_BINDINGS: usize = 256;
/// Maximum number of visible history IDs and terminal IDs.
pub const RUNTIME_MAP_V1_MAX_HISTORY: usize = 256;
/// Maximum complete message size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_MESSAGE_BYTES: usize = 256 * 1024;
/// Maximum ordinary identity field size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_ID_BYTES: usize = 128;
/// Maximum serialized host action-option identity size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_ACTION_OPTION_ID_BYTES: usize = 128;
/// Maximum host legal-action ID size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_HOST_ACTION_ID_BYTES: usize = 512;
/// Maximum reason text size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_REASON_BYTES: usize = 256;
/// Maximum build, version, and projection text size in UTF-8 bytes.
pub const RUNTIME_MAP_V1_MAX_TEXT_BYTES: usize = 128;
/// Inclusive logical coordinate bounds. They are deliberately narrower than JSON-safe integers.
pub const RUNTIME_MAP_V1_MIN_COORDINATE: i32 = -32_768;
pub const RUNTIME_MAP_V1_MAX_COORDINATE: i32 = 32_767;
/// Maximum request/response timeout metadata.
pub const RUNTIME_MAP_V1_MAX_TIMEOUT_MILLIS: u32 = 120_000;
/// Maximum JSON nesting accepted by the bounded pre-parser.
pub const RUNTIME_MAP_V1_MAX_JSON_DEPTH: usize = 32;

pub(super) fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer)
}

pub(super) fn valid_identity(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

pub(super) fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}
