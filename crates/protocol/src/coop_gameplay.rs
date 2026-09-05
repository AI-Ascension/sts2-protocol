// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

pub const COOP_GAMEPLAY_PROTOCOL_VERSION: &str = "coop-gameplay-v1";
pub const COOP_GAMEPLAY_ARTIFACT: &str = "sts2-protocol/coop-gameplay-v1";
pub const COOP_GAMEPLAY_SCHEMA_SOURCE: &str = "schemas/coop-gameplay-v1.schema.json";
pub const COOP_GAMEPLAY_GENERATOR: &str = "hand-authored";
pub const COOP_GAMEPLAY_SCHEMA_DIGEST: &str =
    "85e0028c1ae20e49542791da165eeabaaea0cc2023626b5094b6660ebcc0cc81";
pub const COOP_GAMEPLAY_MAX_PEERS: usize = 4;
pub const COOP_GAMEPLAY_MAX_TEXT_BYTES: usize = 512;
pub const COOP_GAMEPLAY_MAX_GENERATION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopGameplayMessageKind {
    Observation,
    LocalActionRequest,
    SharedVoteRequest,
    SharedEffectResponse,
    SynchronizationResponse,
    RecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopPeerRole {
    Local,
    Ally,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopPlayer {
    pub peer_id: String,
    pub role: CoopPeerRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopLocalActionKind {
    PlayCard,
    EndTurn,
    SelectCard,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopLocalAction {
    pub action_id: String,
    pub kind: CoopLocalActionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoopVoteChoice {
    Approve,
    Reject,
    Abstain,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopSharedVote {
    pub vote_id: String,
    pub proposal_id: String,
    pub voter_id: String,
    pub choice: CoopVoteChoice,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopSharedEffect {
    pub effect_id: String,
    pub kind: String,
    pub from_generation: u64,
    pub to_generation: u64,
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

impl CoopSynchronization {
    /// Checks bounded synchronization metadata only; this never authorizes mutation.
    /// Peer identity, freshness, and host authority require independent consumer checks.
    #[must_use]
    pub const fn is_complete_synchronization(&self) -> bool {
        matches!(self.status, CoopSyncStatus::Synchronized)
            && self.generation <= COOP_GAMEPLAY_MAX_GENERATION
            && self.missing_peers.is_empty()
            && self.peer_count >= 2
            && self.peer_count <= COOP_GAMEPLAY_MAX_PEERS as u8
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoopGameplayMessage {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: CoopProvenance,
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
    pub kind: CoopGameplayMessageKind,
    pub players: Vec<CoopPlayer>,
    #[serde(deserialize_with = "required_nullable")]
    pub local_action: Option<CoopLocalAction>,
    #[serde(deserialize_with = "required_nullable")]
    pub shared_vote: Option<CoopSharedVote>,
    #[serde(deserialize_with = "required_nullable")]
    pub shared_effect: Option<CoopSharedEffect>,
    #[serde(deserialize_with = "required_nullable")]
    pub ally_target: Option<String>,
    pub synchronization: CoopSynchronization,
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
            artifact: COOP_GAMEPLAY_ARTIFACT.to_owned(),
            source: COOP_GAMEPLAY_SCHEMA_SOURCE.to_owned(),
            generator: COOP_GAMEPLAY_GENERATOR.to_owned(),
        }
    }
}

impl CoopGameplayMessage {
    pub fn validate(&self) -> Result<(), CoopGameplayValidationError> {
        if self.protocol_version != COOP_GAMEPLAY_PROTOCOL_VERSION
            || self.schema_digest != COOP_GAMEPLAY_SCHEMA_DIGEST
            || self.provenance != CoopProvenance::default()
            || self.lease_epoch > COOP_GAMEPLAY_MAX_GENERATION
            || self.generation > COOP_GAMEPLAY_MAX_GENERATION
            || [
                &self.correlation_id,
                &self.instance_id,
                &self.session_id,
                &self.lease_id,
            ]
            .iter()
            .any(|value| !valid_identity(value))
        {
            return Err(CoopGameplayValidationError::Metadata);
        }
        if self.players.len() < 2 || self.players.len() > COOP_GAMEPLAY_MAX_PEERS {
            return Err(CoopGameplayValidationError::PeerSet);
        }
        let mut peer_ids = BTreeSet::new();
        let mut local_peers = 0_u8;
        let mut ally_peers = BTreeSet::new();
        for player in &self.players {
            if !valid_identity(&player.peer_id) || !peer_ids.insert(player.peer_id.as_str()) {
                return Err(CoopGameplayValidationError::PeerSet);
            }
            match player.role {
                CoopPeerRole::Local => local_peers += 1,
                CoopPeerRole::Ally => {
                    let _ = ally_peers.insert(player.peer_id.as_str());
                }
            }
        }
        if local_peers != 1 || ally_peers.is_empty() {
            return Err(CoopGameplayValidationError::PeerSet);
        }
        let mut missing_peers = BTreeSet::new();
        if self.synchronization.peer_count as usize != self.players.len()
            || self.synchronization.generation != self.generation
            || self.synchronization.missing_peers.len() > COOP_GAMEPLAY_MAX_PEERS
            || self
                .synchronization
                .missing_peers
                .iter()
                .any(|id| !valid_identity(id))
            || self
                .synchronization
                .missing_peers
                .iter()
                .any(|id| !peer_ids.contains(id.as_str()))
            || self
                .synchronization
                .missing_peers
                .iter()
                .any(|id| !missing_peers.insert(id.as_str()))
            || (matches!(self.synchronization.status, CoopSyncStatus::Synchronized)
                && !self.synchronization.missing_peers.is_empty())
        {
            return Err(CoopGameplayValidationError::Synchronization);
        }
        if let Some(action) = &self.local_action
            && !valid_identity(&action.action_id)
        {
            return Err(CoopGameplayValidationError::Action);
        }
        if let Some(vote) = &self.shared_vote {
            if [&vote.vote_id, &vote.proposal_id, &vote.voter_id]
                .iter()
                .any(|id| !valid_identity(id))
            {
                return Err(CoopGameplayValidationError::Vote);
            }
            if !peer_ids.contains(vote.voter_id.as_str()) {
                return Err(CoopGameplayValidationError::Vote);
            }
        }
        if let Some(effect) = &self.shared_effect
            && (!valid_identity(&effect.effect_id)
                || !valid_identity(&effect.kind)
                || effect.from_generation >= effect.to_generation
                || effect.to_generation != self.generation)
        {
            return Err(CoopGameplayValidationError::Effect);
        }
        if self
            .ally_target
            .as_deref()
            .is_some_and(|id| !valid_identity(id) || !ally_peers.contains(id))
        {
            return Err(CoopGameplayValidationError::AllyTarget);
        }
        if !shape_is_valid(self) {
            return Err(CoopGameplayValidationError::Shape);
        }
        Ok(())
    }

    /// Checks complete message validity and synchronized local-action request shape.
    /// This never authorizes mutation or proves authenticated peers, legality, or host effects.
    #[must_use]
    pub fn is_synchronized_action_request(&self) -> bool {
        self.validate().is_ok()
            && matches!(self.kind, CoopGameplayMessageKind::LocalActionRequest)
            && self.synchronization.is_complete_synchronization()
    }
}

fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(deserializer)
}

fn shape_is_valid(message: &CoopGameplayMessage) -> bool {
    match message.kind {
        CoopGameplayMessageKind::Observation | CoopGameplayMessageKind::SynchronizationResponse => {
            message.local_action.is_none()
                && message.shared_vote.is_none()
                && message.shared_effect.is_none()
                && message.ally_target.is_none()
        }
        CoopGameplayMessageKind::LocalActionRequest => {
            message.local_action.is_some()
                && message.shared_vote.is_none()
                && message.shared_effect.is_none()
                && message.synchronization.is_complete_synchronization()
        }
        CoopGameplayMessageKind::SharedVoteRequest => {
            message.local_action.is_none()
                && message.shared_vote.is_some()
                && message.shared_effect.is_none()
                && message.ally_target.is_none()
        }
        CoopGameplayMessageKind::SharedEffectResponse => {
            message.local_action.is_none()
                && message.shared_vote.is_none()
                && message.shared_effect.is_some()
                && message.ally_target.is_none()
        }
        CoopGameplayMessageKind::RecoveryRequired => {
            !message.synchronization.is_complete_synchronization()
                && message.local_action.is_none()
                && message.shared_vote.is_none()
                && message.shared_effect.is_none()
                && message.ally_target.is_none()
        }
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= COOP_GAMEPLAY_MAX_TEXT_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoopGameplayValidationError {
    Metadata,
    PeerSet,
    Synchronization,
    Action,
    Vote,
    Effect,
    AllyTarget,
    Shape,
}

impl std::fmt::Display for CoopGameplayValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Metadata => "co-op metadata is unsupported",
            Self::PeerSet => "co-op peer set is invalid",
            Self::Synchronization => "co-op synchronization is invalid",
            Self::Action => "co-op local action is invalid",
            Self::Vote => "co-op vote is invalid",
            Self::Effect => "co-op shared effect is invalid",
            Self::AllyTarget => "co-op ally target is invalid",
            Self::Shape => "co-op message shape is invalid",
        })
    }
}

impl std::error::Error for CoopGameplayValidationError {}
