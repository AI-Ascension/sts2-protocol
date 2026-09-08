// SPDX-License-Identifier: MIT

use sha2::{Digest, Sha256};

use super::model::{RuntimeMapV1Message, RuntimeMapV1Snapshot};
use super::validation::RuntimeMapV1ValidationError;

/// Errors returned while validating or canonically encoding a map value.
#[derive(Debug)]
pub enum RuntimeMapV1CanonicalError {
    Validation(RuntimeMapV1ValidationError),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for RuntimeMapV1CanonicalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::Serialization(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RuntimeMapV1CanonicalError {}

impl From<RuntimeMapV1ValidationError> for RuntimeMapV1CanonicalError {
    fn from(error: RuntimeMapV1ValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<serde_json::Error> for RuntimeMapV1CanonicalError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

/// Returns lowercase hexadecimal SHA-256 for arbitrary bytes.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Encodes a validated snapshot as compact deterministic UTF-8 JSON.
pub fn canonical_map_snapshot_json(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<String, RuntimeMapV1CanonicalError> {
    snapshot.validate()?;
    canonical_value(&normalize_snapshot(snapshot))
}

/// Encodes a validated envelope as compact deterministic UTF-8 JSON.
pub fn canonical_map_message_json(
    message: &RuntimeMapV1Message,
) -> Result<String, RuntimeMapV1CanonicalError> {
    message.validate()?;
    canonical_value(&normalize_message(message))
}

/// Hashes the complete canonical snapshot, including all visible projection sections.
pub fn map_snapshot_digest(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<String, RuntimeMapV1CanonicalError> {
    Ok(sha256_hex(
        canonical_map_snapshot_json(snapshot)?.as_bytes(),
    ))
}

/// Hashes only visible node content and projection identity.
pub fn map_content_digest(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<String, RuntimeMapV1CanonicalError> {
    snapshot.validate()?;
    let normalized = normalize_snapshot(snapshot);
    digest_value(&serde_json::json!({
        "domain": "runtime-map-v1/content",
        "state_id": normalized.state_id,
        "generation": normalized.generation,
        "schema_version": normalized.schema_version,
        "projection_version": normalized.projection_version,
        "game_build": normalized.game_build,
        "mod_version": normalized.mod_version,
        "map_instance_id": normalized.map_instance_id,
        "act_id": normalized.act_id,
        "scope_id": normalized.scope_id,
        "availability": normalized.availability,
        "completeness": normalized.completeness,
        "freshness": normalized.freshness,
        "reason": normalized.reason,
        "nodes": normalized.nodes,
    }))
}

/// Hashes only graph coordinates, stable IDs, directed edges, and terminal identity.
pub fn map_topology_digest(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<String, RuntimeMapV1CanonicalError> {
    snapshot.validate()?;
    let normalized = normalize_snapshot(snapshot);
    digest_value(&serde_json::json!({
        "domain": "runtime-map-v1/topology",
        "nodes": normalized
            .nodes
            .iter()
            .map(|node| serde_json::json!({
                "id": node.id,
                "row": node.row,
                "column": node.column,
            }))
            .collect::<Vec<_>>(),
        "edges": normalized.edges,
        "terminal_node_ids": normalized.terminal_node_ids,
    }))
}

/// Hashes only generation-bound position, history, and host navigation bindings.
pub fn map_navigation_digest(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<String, RuntimeMapV1CanonicalError> {
    snapshot.validate()?;
    let normalized = normalize_snapshot(snapshot);
    digest_value(&serde_json::json!({
        "domain": "runtime-map-v1/navigation",
        "state_id": normalized.state_id,
        "generation": normalized.generation,
        "position": normalized.position,
        "history": normalized.history,
        "bindings": normalized.bindings,
    }))
}

impl RuntimeMapV1Snapshot {
    /// Encodes this validated snapshot as canonical compact JSON.
    pub fn canonical_json(&self) -> Result<String, RuntimeMapV1CanonicalError> {
        canonical_map_snapshot_json(self)
    }

    /// Returns the SHA-256 identity of the complete canonical snapshot.
    pub fn digest(&self) -> Result<String, RuntimeMapV1CanonicalError> {
        map_snapshot_digest(self)
    }

    /// Returns the content-only digest.
    pub fn content_digest(&self) -> Result<String, RuntimeMapV1CanonicalError> {
        map_content_digest(self)
    }

    /// Returns the topology-only digest.
    pub fn topology_digest(&self) -> Result<String, RuntimeMapV1CanonicalError> {
        map_topology_digest(self)
    }

    /// Returns the generation-bound navigation digest.
    pub fn navigation_digest(&self) -> Result<String, RuntimeMapV1CanonicalError> {
        map_navigation_digest(self)
    }
}

fn digest_value(value: &serde_json::Value) -> Result<String, RuntimeMapV1CanonicalError> {
    Ok(sha256_hex(canonical_value(value)?.as_bytes()))
}

fn canonical_value<T: serde::Serialize>(value: &T) -> Result<String, RuntimeMapV1CanonicalError> {
    let value = serde_json::to_value(value)?;
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn normalize_snapshot(snapshot: &RuntimeMapV1Snapshot) -> RuntimeMapV1Snapshot {
    let mut normalized = snapshot.clone();
    normalized
        .nodes
        .sort_by(|left, right| left.id.cmp(&right.id));
    normalized
        .edges
        .sort_by(|left, right| (&left.from, &left.to).cmp(&(&right.from, &right.to)));
    normalized.terminal_node_ids.sort();
    normalized.bindings.sort_by(|left, right| {
        (&left.graph_node_id, &left.host_action_id)
            .cmp(&(&right.graph_node_id, &right.host_action_id))
    });
    normalized
}

fn normalize_message(message: &RuntimeMapV1Message) -> RuntimeMapV1Message {
    let mut normalized = message.clone();
    if let Some(snapshot) = &normalized.snapshot {
        normalized.snapshot = Some(normalize_snapshot(snapshot));
    }
    normalized
}
