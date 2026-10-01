// SPDX-License-Identifier: MIT

//! The entities an observation is made of: the cards a player holds, the enemies they face, and the
//! player's own resources.
//!
//! These are separated from the message envelope because they answer a different question. The
//! envelope says when a decision is being made; these say what the decision is being made *about*.
//! Everything here is player-visible and bounded. No host object, save, RNG, or unrevealed outcome
//! is representable in these types, which is what keeps the projection fair-play.

use super::{
    RuntimeV3GameplayEnemyIntent, RuntimeV3GameplayPotion, RuntimeV3GameplayRelic, absent_unless,
};

/// A bounded player-visible card description; draw order and unrevealed outcomes are absent.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayCard {
    pub card_id: String,
    pub name: String,
    pub cost: u8,
    pub upgraded: bool,
    /// The host's own card text, when it carries it.
    ///
    /// Absence means the host said nothing, which is different from a card with an empty effect.
    /// A decision that turns on what a card does cannot be made without it, so a host that knows
    /// it should say it; a host that does not is unaffected.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

/// A bounded player-visible enemy description.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayEnemy {
    pub enemy_id: String,
    pub name: String,
    pub hp: u16,
    pub max_hp: u16,
    pub intent: RuntimeV3GameplayEnemyIntent,
}

/// Player-visible resources and known card contents.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayPlayer {
    pub hp: u16,
    pub max_hp: u16,
    pub energy: u8,
    pub gold: u32,
    pub hand: Vec<RuntimeV3GameplayCard>,
    pub deck: Vec<RuntimeV3GameplayCard>,
    pub discard: Vec<RuntimeV3GameplayCard>,
    pub exhaust: Vec<RuntimeV3GameplayCard>,
    /// Relics disclosed by the host. Absent means it disclosed none; `[]` means it disclosed
    /// that it has none.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub relics: Option<Vec<RuntimeV3GameplayRelic>>,
    /// Potions disclosed by the host. Absent and empty are distinct, as above.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub potions: Option<Vec<RuntimeV3GameplayPotion>>,
    /// Potion slots currently occupied.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub potion_slots: Option<u8>,
    /// Potion slots the player has in total.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_potion_slots: Option<u8>,
}
