// SPDX-License-Identifier: MIT

//! Semantic actions, transition witnesses, and recovery outcomes for the gameplay profile.

use super::{required_nullable, wire};

/// Semantic action payload. Coordinates, arbitrary input events, reflection paths, and process
/// commands are intentionally not variants of this enum.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", from = "wire::Action")]
pub enum RuntimeV3GameplayAction {
    StartRun {
        character_id: String,
    },
    SelectMapNode {
        node_id: String,
    },
    PlayCard {
        card_id: String,
        #[serde(deserialize_with = "required_nullable")]
        target_id: Option<String>,
    },
    EndTurn,
    ChooseReward {
        reward_id: String,
    },
    SkipReward,
    ShopPurchase {
        item_id: String,
    },
    ShopRemove {
        card_id: String,
    },
    Rest,
    Smith {
        card_id: String,
    },
    EventChoice {
        choice_id: String,
    },
    SelectCard {
        card_id: String,
    },
    ConfirmVictory,
    SaveQuit,
}

/// Host-generated action identity plus its typed semantic payload.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayLegalAction {
    pub action_id: String,
    pub action: RuntimeV3GameplayAction,
}

/// An independently checkable postcondition witness. It does not grant authority by itself.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayTransitionWitness {
    pub from_generation: u64,
    pub to_generation: u64,
    pub state_id: String,
    pub effect_kind: String,
}

/// Safe recovery operation. Recovery never selects a strategic gameplay action.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayRecoveryKind {
    Reobserve,
    Reconcile,
    ReleaseLease,
    StopEpisode,
}

/// Bounded recovery request metadata.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayRecovery {
    pub kind: RuntimeV3GameplayRecoveryKind,
    #[serde(deserialize_with = "required_nullable")]
    pub operation_id: Option<String>,
}

/// Wait outcome used to distinguish a stable observation from a timeout.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayWaitOutcome {
    Successor,
    SameStateMutation,
    Timeout,
    RecoveryRequired,
}

/// Lifecycle result used by mutating and recovery operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayStatus {
    Accepted,
    Settled,
    Rejected,
    Unknown,
    Cancelled,
}
