// SPDX-License-Identifier: MIT

//! What the player is already carrying: relics, potions, and the slots they occupy.
//!
//! These live apart from the message shape because they answer a different question from the
//! observation envelope. The envelope says when a decision is being made; this says what the
//! decision is being made *about*. A relic changes the rules for the run and a potion usable this
//! turn is exactly the fact a decision turns on, so an observation that cannot carry them reasons
//! about a different game from the one being played.

use serde::{Deserialize, Serialize};

/// Deserializes a member that may be absent but, when present, must carry a value of `T`.
///
/// A plain `Option<T>` cannot express this. `#[serde(default)]` supplies absence, and serde's
/// derived `Option` handling supplies `None` for an explicit JSON `null` too, so the parser would
/// accept `"relics": null` where the schema's optional-array rule refuses it. Absence and an
/// explicit null are different statements, and the schema already distinguishes them, so the parser
/// must as well or the two disagree about the same document.
///
/// This is the opposite rule from `required_nullable`, which admits a `null` because its member is
/// required on the wire; here the member is not required at all, so `null` may not stand in for
/// silence.
pub(crate) fn absent_unless<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
    if value.is_null() {
        return Err(serde::de::Error::custom(
            "member must be absent rather than null",
        ));
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

/// A relic the player currently holds.
///
/// A relic changes the rules for the run, so an observation that omits them reasons about a
/// different game from the one being played. Omitted entirely means the host disclosed none; an
/// empty array means the host disclosed that there are none.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayRelic {
    pub relic_id: String,
    pub name: String,
    /// What the relic does, when the host carries its own text.
    ///
    /// Absence means the host said nothing, which is not the same as a relic that does nothing.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

/// Who a potion can be used on.
///
/// `Unknown` is an explicit statement that the host does not know, which is not the same as
/// omitting the potion: a host that cannot answer may leave the potion out instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
// `self` is a Rust keyword, so the variant cannot be named after it directly. The wire form is
// `self`, matching runtime-v4, and the rename is explicit so the two cannot drift.
#[serde(rename_all = "snake_case")]
pub enum RuntimeV3GameplayPotionTargetMode {
    #[serde(rename = "self")]
    SelfTarget,
    AnyEnemy,
    AllEnemies,
    None,
    Unknown,
}

/// A potion in one slot of the player's potion belt.
///
/// Only `potion_id` and `name` are required. A host that knows the identity of what it is holding
/// and nothing else can still say so, which is more useful than staying silent about the whole
/// belt.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeV3GameplayPotion {
    pub potion_id: String,
    pub name: String,
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub slot: Option<u8>,
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub usable: Option<bool>,
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub target_mode: Option<RuntimeV3GameplayPotionTargetMode>,
    /// What the potion does, when the host carries its own text.
    #[serde(
        default,
        deserialize_with = "absent_unless",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}
