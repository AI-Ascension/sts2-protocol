// SPDX-License-Identifier: MIT

//! Conformance for the relic, potion, and card-description members.
//!
//! These exist because the harness consumer already admitted all of this (sts2-harness#316, PR
//! #314) while the wire contract refused it, so a producer that knew its relics and potions could
//! not say so: every affected object is `additionalProperties: false`, so the whole observation was
//! refused, not trimmed. Each case is checked against both the schema and the parser, because a
//! disagreement in either direction is a real defect.

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V3_GAMEPLAY_MAX_ENTITIES, RuntimeV3GameplayAction, RuntimeV3GameplayMessage,
    RuntimeV3GameplayPotionTargetMode,
};

const BELT: &str = include_str!(
    "../../../artifacts/runtime-v3-gameplay/golden/state-response-described-belt.json"
);
const RESPONSE: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json");
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

fn base() -> Value {
    serde_json::from_str(RESPONSE).expect("golden is JSON")
}

fn rust_accepts(value: &Value) -> bool {
    serde_json::from_value::<RuntimeV3GameplayMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

/// Asserts the schema and the parser reach the same verdict, which is the property under test.
fn assert_parity(document: &Value, expected: bool, context: &str) {
    let by_schema = validator().is_valid(document);
    let by_rust = rust_accepts(document);
    assert_eq!(
        by_schema, expected,
        "schema disagreed with the expectation for {context}: {document}"
    );
    assert_eq!(
        by_schema, by_rust,
        "schema and parser disagreed for {context}: schema={by_schema} rust={by_rust}"
    );
}

/// The player object of `document`, so a case can vary only the belt.
fn player_of(document: &Value) -> &Value {
    &document["observation"]["player"]
}

fn with_player(player: Value) -> Value {
    let mut document = base();
    document["observation"]["player"] = player;
    document
}

fn full_player() -> Value {
    player_of(&serde_json::from_str::<Value>(BELT).expect("belt golden is JSON")).clone()
}

#[test]
fn the_belt_golden_is_admitted_by_the_schema_and_the_parser_alike() {
    let document: Value = serde_json::from_str(BELT).expect("belt golden is JSON");
    assert_parity(&document, true, "the described-belt golden");

    // It must also round-trip byte-for-byte, so a producer emitting exactly this is stable.
    let message: RuntimeV3GameplayMessage = serde_json::from_str(BELT).expect("parses");
    message.validate().expect("validates");
    assert_eq!(
        serde_json::to_string(&message).expect("serializes"),
        BELT.trim_end(),
        "the golden must be canonical for the parser that reads it"
    );
}

#[test]
fn a_host_that_discloses_nothing_is_still_byte_valid() {
    // Every new member is optional. A host that knows none of this must be unaffected, and its
    // serialized form must not grow invented empty collections.
    let document = base();
    assert_parity(&document, true, "a player with no belt members");
    let message: RuntimeV3GameplayMessage =
        serde_json::from_value(document).expect("parses without the new members");
    let player = &message.observation.as_ref().expect("an observation").player;
    assert!(player.relics.is_none(), "absence must stay absent");
    assert!(player.potions.is_none(), "absence must stay absent");
    assert!(player.potion_slots.is_none(), "absence must stay absent");
    assert!(
        player.max_potion_slots.is_none(),
        "absence must stay absent"
    );
    for card in &player.hand {
        assert!(
            card.description.is_none(),
            "an undisclosed card description must not become a value"
        );
    }
}

#[test]
fn an_absent_belt_and_an_empty_belt_are_different_statements() {
    // Omitted means the host disclosed none. An empty array means the host disclosed that it has
    // none. Collapsing them would let a reader mistake silence for a fact.
    let mut player = full_player();
    player["relics"] = json!([]);
    player["potions"] = json!([]);
    let empty = with_player(player);
    assert_parity(&empty, true, "an explicitly empty belt");

    let message: RuntimeV3GameplayMessage =
        serde_json::from_value(empty.clone()).expect("an empty belt parses");
    let player = &message.observation.as_ref().expect("an observation").player;
    assert_eq!(
        player.relics.as_ref().map(Vec::len),
        Some(0),
        "an empty array must survive as an empty array, not as absence"
    );
    assert_eq!(
        player.potions.as_ref().map(Vec::len),
        Some(0),
        "an empty array must survive as an empty array, not as absence"
    );
}

#[test]
fn relics_and_potions_are_admitted_with_every_optional_member_omitted() {
    // The harness reference requires only `relic_id`/`name` and `potion_id`/`name`. A host that
    // knows the identity and nothing else must still be able to say so.
    let mut player = full_player();
    player["relics"] = json!([{ "relic_id": "relic:1:Anchor", "name": "Anchor" }]);
    player["potions"] = json!([{ "potion_id": "potion:1:Fire", "name": "Fire Potion" }]);
    let document = with_player(player);
    assert_parity(&document, true, "the minimal relic and potion shapes");
}

#[test]
fn every_potion_target_mode_round_trips_including_the_keyword_one() {
    // `self` is the value the schema declares and the one runtime-v4 already uses. It is also a
    // Rust keyword, so this pins the explicit rename rather than letting a default derive it.
    for (wire, expected) in [
        ("self", RuntimeV3GameplayPotionTargetMode::SelfTarget),
        ("any_enemy", RuntimeV3GameplayPotionTargetMode::AnyEnemy),
        ("all_enemies", RuntimeV3GameplayPotionTargetMode::AllEnemies),
        ("none", RuntimeV3GameplayPotionTargetMode::None),
        ("unknown", RuntimeV3GameplayPotionTargetMode::Unknown),
    ] {
        let mut player = full_player();
        player["potions"] =
            json!([{ "potion_id": "potion:1:Fire", "name": "Fire", "target_mode": wire }]);
        let document = with_player(player);
        assert_parity(&document, true, &format!("target_mode {wire}"));
        let message: RuntimeV3GameplayMessage = serde_json::from_value(document).expect("parses");
        let player = message.observation.as_ref().expect("an observation");
        assert_eq!(
            player.player.potions.as_ref().expect("present")[0].target_mode,
            Some(expected),
            "target_mode {wire} must decode to the declared variant"
        );
    }
}

#[test]
fn an_unknown_member_on_a_relic_or_a_potion_is_refused() {
    for (collection, entry) in [
        (
            "relics",
            json!({ "relic_id": "relic:1:Anchor", "name": "Anchor", "charges": 3 }),
        ),
        (
            "potions",
            json!({ "potion_id": "potion:1:Fire", "name": "Fire", "rarity": "rare" }),
        ),
    ] {
        let mut player = full_player();
        player[collection] = json!([entry]);
        assert_parity(
            &with_player(player),
            false,
            &format!("an unknown member on a {collection} entry"),
        );
    }
}

#[test]
fn an_oversized_description_is_refused_on_a_relic_a_potion_and_a_card() {
    let long = "a".repeat(513);
    let at_bound = "a".repeat(512);
    for value in [at_bound, long] {
        let expected = value.chars().count() <= 512;
        for (collection, entry) in [
            (
                "relics",
                json!({ "relic_id": "relic:1:Anchor", "name": "Anchor", "description": value }),
            ),
            (
                "potions",
                json!({ "potion_id": "potion:1:Fire", "name": "Fire", "description": value }),
            ),
        ] {
            let mut player = full_player();
            player[collection] = json!([entry]);
            assert_parity(
                &with_player(player),
                expected,
                &format!("a {} -character description", value.chars().count()),
            );
        }
    }

    // The same bound governs a description on a card the player holds.
    for (value, expected) in [("a".repeat(512), true), ("a".repeat(513), false)] {
        let mut player = full_player();
        player["hand"] = json!([{
            "card_id": "card:31:Blood-Wall",
            "name": "Blood Wall",
            "cost": 2,
            "upgraded": true,
            "description": value,
        }]);
        assert_parity(
            &with_player(player),
            expected,
            &format!("a {} -character card description", value.chars().count()),
        );
    }
}

#[test]
fn a_potion_slot_count_outside_its_bound_is_refused() {
    for value in [json!(0), json!(255)] {
        let mut player = full_player();
        player["potion_slots"] = value.clone();
        assert_parity(
            &with_player(player),
            true,
            &format!("potion_slots at {value}"),
        );
    }
    for value in [json!(-1), json!(256), json!("3"), json!(1.5)] {
        let mut player = full_player();
        player["max_potion_slots"] = value.clone();
        assert_parity(
            &with_player(player),
            false,
            &format!("max_potion_slots at {value}"),
        );
    }
}

#[test]
fn more_relics_or_potions_than_the_bound_are_refused() {
    let too_many = RUNTIME_V3_GAMEPLAY_MAX_ENTITIES + 1;
    for collection in ["relics", "potions"] {
        let entry = if collection == "relics" {
            json!({ "relic_id": "relic:1:Anchor", "name": "Anchor" })
        } else {
            json!({ "potion_id": "potion:1:Fire", "name": "Fire" })
        };
        let mut player = full_player();
        player[collection] = json!(vec![entry; too_many]);
        assert_parity(
            &with_player(player),
            false,
            &format!("{too_many} {collection}"),
        );
    }
}

#[test]
fn both_potion_actions_are_admitted_and_carry_their_identities() {
    let use_potion = json!({
        "kind": "use_potion",
        "potion_id": "potion:1:Fire",
        "target_id": "enemy-1",
    });
    let discard = json!({
        "kind": "discard_potion",
        "potion_id": "potion:3:Unknown",
    });
    for action in [use_potion.clone(), discard.clone()] {
        let mut document = base();
        document["legal_actions"] =
            json!([{ "action_id": "potion.act", "action": action.clone() }]);
        assert_parity(&document, true, &format!("the {} action", action["kind"]));
        let message: RuntimeV3GameplayMessage =
            serde_json::from_value(document).expect("the action parses");
        let actions = message.legal_actions.as_ref().expect("legal actions");
        match &actions[0].action {
            RuntimeV3GameplayAction::UsePotion {
                potion_id,
                target_id,
                ..
            } => {
                assert_eq!(potion_id, "potion:1:Fire");
                assert_eq!(target_id.as_deref(), Some("enemy-1"));
            }
            RuntimeV3GameplayAction::DiscardPotion { potion_id } => {
                assert_eq!(potion_id, "potion:3:Unknown");
            }
            other => panic!("unexpected action {other:?}"),
        }
    }
}

#[test]
fn a_potion_action_with_a_bad_identity_or_an_unknown_member_is_refused() {
    for action in [
        json!({ "kind": "use_potion", "potion_id": "potion 1", "target_id": null }),
        json!({ "kind": "discard_potion", "potion_id": "" }),
        json!({ "kind": "use_potion", "potion_id": "potion:1:Fire", "amount": 2 }),
        json!({ "kind": "discard_potion" }),
    ] {
        let mut document = base();
        document["legal_actions"] = json!([{ "action_id": "potion.act", "action": action }]);
        assert_parity(&document, false, "a malformed potion action");
    }
}

#[test]
fn use_potion_accepts_an_explicit_null_target() {
    // `target_id` is required on the wire but nullable, exactly like `play_card`: a potion with
    // no target says so with null rather than omitting the member.
    let mut document = base();
    document["legal_actions"] = json!([{
        "action_id": "potion.use.self",
        "action": { "kind": "use_potion", "potion_id": "potion:2:Block", "target_id": Value::Null },
    }]);
    assert_parity(&document, true, "use_potion with a null target");
}
