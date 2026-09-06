// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

pub const COOP_SYNC_PROTOCOL_VERSION: &str = "coop-synchronization-v1";
pub const COOP_SYNC_ARTIFACT: &str = "sts2-protocol/coop-synchronization-v1";
pub const COOP_SYNC_SCHEMA_SOURCE: &str = "schemas/coop-synchronization-v1.schema.json";
pub const COOP_SYNC_SCHEMA_DIGEST: &str =
    "d410858cabbd38612345120c2196423130c7b21d788fd2b0d775cd82887087ec";
pub const COOP_SYNC_MAX_GENERATION: u64 = 9_007_199_254_740_991;
pub const COOP_SYNC_MAX_PEERS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopPeerRole {
    Local,
    Ally,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopPeer {
    pub peer_id: String,
    pub role: CoopPeerRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopSyncStatus {
    Synchronized,
    Disagreement,
    Disconnected,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopSynchronization {
    pub status: CoopSyncStatus,
    pub generation: u64,
    pub peer_count: u8,
    pub missing_peers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for CoopProvenance {
    fn default() -> Self {
        Self {
            artifact: COOP_SYNC_ARTIFACT.to_owned(),
            source: COOP_SYNC_SCHEMA_SOURCE.to_owned(),
            generator: "hand-authored".to_owned(),
        }
    }
}

/// Inert coordinator-reported metadata. No predicate here grants host authority.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopSynchronizationMessage {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: CoopProvenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: String,
    pub source: String,
    pub players: Vec<CoopPeer>,
    pub synchronization: CoopSynchronization,
}

impl CoopSynchronizationMessage {
    pub fn validate(&self) -> Result<(), CoopSynchronizationError> {
        if self.protocol_version != COOP_SYNC_PROTOCOL_VERSION
            || self.schema_digest != COOP_SYNC_SCHEMA_DIGEST
            || self.provenance != CoopProvenance::default()
            || self.kind != "synchronization_response"
            || self.source != "gateway_peer_reports"
            || self.lease_epoch > COOP_SYNC_MAX_GENERATION
            || self.generation > COOP_SYNC_MAX_GENERATION
            || [
                &self.correlation_id,
                &self.instance_id,
                &self.session_id,
                &self.lease_id,
            ]
            .iter()
            .any(|id| !valid_identity(id))
        {
            return Err(CoopSynchronizationError::Metadata);
        }
        let peers = self.valid_peers()?;
        let sync = &self.synchronization;
        let mut missing = BTreeSet::new();
        if sync.generation != self.generation
            || usize::from(sync.peer_count) != peers.len()
            || sync.missing_peers.len() > COOP_SYNC_MAX_PEERS
            || sync
                .missing_peers
                .iter()
                .any(|id| !peers.contains(id.as_str()) || !missing.insert(id.as_str()))
            || (sync.status == CoopSyncStatus::Disconnected) == missing.is_empty()
        {
            return Err(CoopSynchronizationError::Synchronization);
        }
        Ok(())
    }

    fn valid_peers(&self) -> Result<BTreeSet<&str>, CoopSynchronizationError> {
        if !(2..=COOP_SYNC_MAX_PEERS).contains(&self.players.len()) {
            return Err(CoopSynchronizationError::Peers);
        }
        let mut peers = BTreeSet::new();
        let mut locals = 0;
        for peer in &self.players {
            if !valid_identity(&peer.peer_id) || !peers.insert(peer.peer_id.as_str()) {
                return Err(CoopSynchronizationError::Peers);
            }
            if peer.role == CoopPeerRole::Local {
                locals += 1;
            }
        }
        if locals != 1 {
            return Err(CoopSynchronizationError::Peers);
        }
        Ok(peers)
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoopSynchronizationError {
    Metadata,
    Peers,
    Synchronization,
}

impl std::fmt::Display for CoopSynchronizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Metadata => "co-op synchronization metadata is invalid",
            Self::Peers => "co-op synchronization roster is invalid",
            Self::Synchronization => "co-op synchronization relationships are invalid",
        })
    }
}

impl std::error::Error for CoopSynchronizationError {}
