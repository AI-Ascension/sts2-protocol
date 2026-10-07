// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

mod inputs;
pub use self::inputs::{
    Applicability, Availability, Observation, RuleInput, SourceKind, SourceRef, Unit, Value,
};

/// Closed envelope shared by the four message kinds.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: GameFactsProvenance,
    pub correlation_id: String,
    pub kind: MessageKind,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub query: Option<Query>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub result: Option<QueryResult>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub capabilities: Option<Capabilities>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub error: Option<ErrorBody>,
}

/// Exact profile discriminator on the outer envelope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    QueryRequest,
    QueryResponse,
    CapabilitiesResponse,
    ErrorResponse,
}

/// Schema identity metadata, not a claim that the source is installed or accepted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GameFactsProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

/// Unpaged request; rule ID membership and core input order belong to the producer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub rules_reference_version: u32,
    pub rule_ids: Vec<String>,
    pub binding: Binding,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub scope: Option<AuthenticatedScope>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub parent_observation: Option<ParentObservation>,
}

/// Static or generation-bound live content scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub mode: BindingMode,
    pub content_manifest_id: String,
    pub locale: String,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub visibility_scope: Option<String>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub instance_ref: Option<InstanceRef>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub snapshot_ref: Option<SnapshotRef>,
}

/// Static content or live state query mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingMode {
    Static,
    Live,
}

/// Authenticated bootstrap scope metadata; authentication remains external.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticatedScope {
    pub instance_id: String,
    pub run_id: String,
    pub authority_epoch: u64,
    pub content_manifest_id: String,
    pub locale: String,
}

/// Game-mod-owned instance identity from query-v2/bootstrap-v1.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceRef {
    pub instance_id: String,
    pub run_id: String,
    pub epoch: u64,
    pub entity_kind: EntityKind,
    pub entity_id: String,
}

/// Closed entity vocabulary reused from query-v2.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Card,
    Character,
    Enemy,
    Event,
    MapNode,
    Potion,
    Power,
    Relic,
    Room,
    Status,
    RestOption,
}

/// Generation-specific snapshot identity owned by the producer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRef {
    pub snapshot_id: String,
    pub instance_ref: InstanceRef,
    pub state_generation: u64,
}

/// Parent capture echo carried by a live query.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParentObservation {
    pub instance_ref: InstanceRef,
    pub snapshot_ref: SnapshotRef,
    pub state_generation: u64,
}

/// Producer capability declaration; null snapshot policy means no live offer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub profile: String,
    pub rules_reference_version: u32,
    pub max_rule_ids: u16,
    pub max_inputs_per_rule: u16,
    pub max_unsupported_combinations: u16,
    pub max_message_bytes: u32,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub snapshot_policy: Option<SnapshotPolicy>,
}

/// Bounded generation lifetime; this does not assert current producer support.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotPolicy {
    pub supports_live: bool,
    pub lifetime_generations: u32,
    pub max_retained_snapshots: u16,
    pub expiry_behavior: ExpiryBehavior,
    pub invalidated_by: Vec<SnapshotInvalidator>,
}

/// A stale generation is rejected, never silently refreshed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryBehavior {
    RejectStaleSnapshot,
}

/// State changes invalidating a retained snapshot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotInvalidator {
    Restore,
    Restart,
    ProfileChange,
    ContentChange,
    RunChange,
    EpochChange,
}

/// Result metadata and exactly one ordered result for each requested rule ID.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QueryResult {
    pub producer_version: String,
    pub rules_reference_version: u32,
    pub inventory_binding: InventoryBinding,
    pub results: Vec<QueryResultEntry>,
    pub unsupported_combinations: Vec<UnsupportedCombination>,
}

/// Producer's selected installed-build/mode and content inventory witness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryBinding {
    pub build_id: String,
    pub mode_id: String,
    pub manifest: InventoryManifest,
}

/// Opaque, generation-scoped catalog and content revisions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryManifest {
    pub catalog_generation: u64,
    pub adapter_compatibility: String,
    pub content_set_revision: String,
    pub localized_text_revision: String,
    pub inventory_revision: String,
}

/// Tagged per-rule outcome. Protocol checks structure and query alignment only.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryResultEntry {
    Found {
        rule_id: String,
        evidence_status: EvidenceStatus,
        inputs: Vec<RuleInput>,
    },
    Unsupported {
        rule_id: String,
        reason_code: UnsupportedReason,
    },
}

/// Evidence classification; only producer-owned evidence can be source-backed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Confirmed,
    SourceDerived,
    Proposed,
    Inferred,
    Unverified,
}

/// Closed refusal reason for an unsupported known rule.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedReason {
    MissingCapability,
    UnsupportedField,
}

/// Explicit producer-declared combinations that cannot be evaluated together.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsupportedCombination {
    pub rule_ids: Vec<String>,
    pub reason: String,
}

/// Closed error response; text is bounded and never contains host paths/exceptions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorBody {
    pub code: ErrorCode,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub field: Option<String>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub reason: Option<String>,
    pub retryable: bool,
}

/// Stable v1 refusal vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnknownKind,
    UnknownId,
    AmbiguousId,
    UnsupportedFilter,
    UnsupportedProjection,
    UnsupportedVersion,
    DeniedScope,
    StaleSnapshot,
    StaleCursor,
    ResultLimitExceeded,
    MissingCapability,
    UnsupportedField,
    InvalidIdentity,
    InvalidBounds,
    MixedGeneration,
    Malformed,
    ReadOnlyViolation,
    InvalidRequest,
    InvalidBinding,
    AmbiguousEntity,
}
