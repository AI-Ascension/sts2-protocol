// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use super::*;

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub enum RuntimeV3GameplayCombatPhase {
    #[serde(rename = "outside_combat")]
    OutsideCombat,
    #[serde(rename = "combat/player_turn")]
    PlayerTurn,
    #[serde(rename = "combat/enemy_turn")]
    EnemyTurn,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayStatus {
    Accepted,
    Settled,
    Rejected,
    Unknown,
    Cancelled,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayMessageKind {
    StateRequest,
    StateResponse,
    ActionRequest,
    ActionResponse,
    ReconcileRequest,
    ReconcileResponse,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayProvenance {
    pub artifact: String,
    pub source: String,
    pub generator: String,
}

impl Default for RuntimeV3GameplayProvenance {
    fn default() -> Self {
        Self {
            artifact: RUNTIME_V3_GAMEPLAY_ARTIFACT.to_owned(),
            source: RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE.to_owned(),
            generator: RUNTIME_V3_GAMEPLAY_GENERATOR.to_owned(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayMetadata {
    pub protocol_version: String,
    pub schema_digest: String,
    pub provenance: RuntimeV3GameplayProvenance,
}

impl Default for RuntimeV3GameplayMetadata {
    fn default() -> Self {
        Self {
            protocol_version: RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION.to_owned(),
            schema_digest: RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST.to_owned(),
            provenance: RuntimeV3GameplayProvenance::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeV3GameplayContext {
    pub correlation_id: String,
    pub instance_id: String,
    pub session_id: String,
    pub lease_id: String,
    pub lease_epoch: u64,
    pub generation: u64,
}

impl RuntimeV3GameplayContext {
    #[must_use]
    pub fn new(
        correlation_id: impl Into<String>,
        instance_id: impl Into<String>,
        session_id: impl Into<String>,
        lease_id: impl Into<String>,
        lease_epoch: u64,
        generation: u64,
    ) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            instance_id: instance_id.into(),
            session_id: session_id.into(),
            lease_id: lease_id.into(),
            lease_epoch,
            generation,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayEnemy {
    pub target_id: String,
    pub alive: bool,
    pub hittable: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayObservation {
    pub combat_phase: RuntimeV3GameplayCombatPhase,
    pub turn_index: u16,
    pub host_ready: bool,
    pub generation: u64,
    pub hand_count: u16,
    pub energy: u16,
    pub draw_pile_count: u16,
    pub discard_pile_count: u16,
    pub exhaust_pile_count: u16,
    pub enemies: Vec<RuntimeV3GameplayEnemy>,
}

impl RuntimeV3GameplayObservation {
    pub fn validate(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        if self.turn_index > RUNTIME_V3_GAMEPLAY_MAX_TURN_INDEX
            || self.generation > RUNTIME_V3_GAMEPLAY_MAX_GENERATION
            || self.hand_count > RUNTIME_V3_GAMEPLAY_MAX_CARD_INDEX
            || self.energy > RUNTIME_V3_GAMEPLAY_MAX_ENERGY
            || self.draw_pile_count > RUNTIME_V3_GAMEPLAY_MAX_PILE_COUNT
            || self.discard_pile_count > RUNTIME_V3_GAMEPLAY_MAX_PILE_COUNT
            || self.exhaust_pile_count > RUNTIME_V3_GAMEPLAY_MAX_PILE_COUNT
            || self.enemies.len() > RUNTIME_V3_GAMEPLAY_MAX_ENEMIES
        {
            return Err(RuntimeV3GameplayValidationError::ObservationBounds);
        }
        let mut ids = BTreeSet::new();
        for enemy in &self.enemies {
            validate_identity(&enemy.target_id)?;
            if !ids.insert(&enemy.target_id) {
                return Err(RuntimeV3GameplayValidationError::DuplicateTarget);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayAction {
    pub action_id: String,
    pub card_index: u16,
    pub target_id: Option<String>,
}

impl RuntimeV3GameplayAction {
    pub fn validate(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        if self.action_id != RUNTIME_V3_GAMEPLAY_ACTION_ID
            || self.card_index > RUNTIME_V3_GAMEPLAY_MAX_CARD_INDEX
        {
            return Err(RuntimeV3GameplayValidationError::ActionBounds);
        }
        if let Some(target_id) = &self.target_id {
            validate_identity(target_id)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn play_card(card_index: u16, target_id: Option<String>) -> Self {
        Self {
            action_id: RUNTIME_V3_GAMEPLAY_ACTION_ID.to_owned(),
            card_index,
            target_id,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayEffectWitness {
    pub kind: String,
    pub generation: u64,
    pub card_index: u16,
    pub target_id: Option<String>,
}

impl RuntimeV3GameplayEffectWitness {
    pub fn validate(&self) -> Result<(), RuntimeV3GameplayValidationError> {
        if self.kind != RUNTIME_V3_GAMEPLAY_EFFECT_KIND
            || self.generation > RUNTIME_V3_GAMEPLAY_MAX_GENERATION
            || self.card_index > RUNTIME_V3_GAMEPLAY_MAX_CARD_INDEX
        {
            return Err(RuntimeV3GameplayValidationError::EffectBounds);
        }
        if let Some(target_id) = &self.target_id {
            validate_identity(target_id)?;
        }
        Ok(())
    }
}

fn validate_identity(value: &str) -> Result<(), RuntimeV3GameplayValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(RuntimeV3GameplayValidationError::InvalidIdentity);
    }
    Ok(())
}
