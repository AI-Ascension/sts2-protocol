// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use super::EntityKind;

/// One named core input, preserving numerator/denominator as independent integers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleInput {
    pub name: String,
    pub unit: Unit,
    pub applicability: Applicability,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub observation: Option<Observation>,
    pub source_ref: SourceRef,
}

/// Core rule input unit; membership and exact ordering are producer-owned.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Damage,
    HitPoints,
    Block,
    Energy,
    Count,
    Turns,
    Rounds,
    Gold,
    Multiplier,
    Ratio,
    Entity,
    Boolean,
    Dimensionless,
}

/// Required, conditional, or owner-unresolved applicability annotation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    Required,
    Conditional,
    Unknown,
}

/// Typed live observation; non-available states carry a null value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub availability: Availability,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub value: Option<Value>,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub reason: Option<String>,
}

/// Availability is distinct from false, zero, or an absent input.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    Unavailable,
    NotObservable,
    Redacted,
    Unsupported,
    Missing,
}

/// Closed exact scalar/reference union. Integer values are safe in JSON consumers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Integer { value: i64 },
    Boolean { value: bool },
    EntityRef { entity_kind: EntityKind, id: String },
}

/// Source category for an opaque bounded reference token.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    ContentManifest,
    GameMod,
    GatewayProjection,
    Synthetic,
}

/// Opaque reference token; it is never interpreted as a path or host exception.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    pub kind: SourceKind,
    #[serde(deserialize_with = "crate::serialization::required_option")]
    pub r#ref: Option<String>,
}
