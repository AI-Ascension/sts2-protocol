// SPDX-License-Identifier: MIT

mod message;
mod types;
mod validation;

pub use message::{RuntimeV3GameplayActionResult, RuntimeV3GameplayMessage};
pub use types::{
    RuntimeV3GameplayAction, RuntimeV3GameplayCombatPhase, RuntimeV3GameplayContext,
    RuntimeV3GameplayEffectWitness, RuntimeV3GameplayEnemy, RuntimeV3GameplayMessageKind,
    RuntimeV3GameplayMetadata, RuntimeV3GameplayObservation, RuntimeV3GameplayProvenance,
    RuntimeV3GameplayStatus,
};
pub use validation::RuntimeV3GameplayValidationError;

pub const RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION: &str = "runtime-v3-gameplay";
pub const RUNTIME_V3_GAMEPLAY_ARTIFACT: &str = "sts2-protocol/runtime-v3-gameplay";
pub const RUNTIME_V3_GAMEPLAY_SCHEMA_SOURCE: &str = "schemas/runtime-v3-gameplay.schema.json";
pub const RUNTIME_V3_GAMEPLAY_GENERATOR: &str = "hand-authored";
pub const RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST: &str =
    "c961bbde893f0422f80233d14ea9ae8b648ee9032136e5370aa5f6b949f6575e";
pub const RUNTIME_V3_GAMEPLAY_ACTION_ID: &str = "play_card";
pub const RUNTIME_V3_GAMEPLAY_EFFECT_KIND: &str = "play_card_settled";
pub const RUNTIME_V3_GAMEPLAY_MAX_GENERATION: u64 = 9_007_199_254_740_991;
pub const RUNTIME_V3_GAMEPLAY_MAX_TURN_INDEX: u16 = 1024;
pub const RUNTIME_V3_GAMEPLAY_MAX_CARD_INDEX: u16 = 64;
pub const RUNTIME_V3_GAMEPLAY_MAX_ENERGY: u16 = 999;
pub const RUNTIME_V3_GAMEPLAY_MAX_PILE_COUNT: u16 = 1024;
pub const RUNTIME_V3_GAMEPLAY_MAX_ENEMIES: usize = 16;
