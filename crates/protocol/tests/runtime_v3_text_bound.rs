// SPDX-License-Identifier: MIT

//! Conformance for the `#/$defs/text` character bound.
//!
//! JSON Schema's `maxLength` counts Unicode characters, not UTF-8 bytes. These cases exist because
//! the parser used to count bytes, which made it stricter than the schema: a 512-character
//! accented name was refused even though the contract admits it. Every case is checked against
//! both the schema and the parser, so neither side can drift back into a private reading of
//! `maxLength` without a test failing.

use serde_json::{Value, json};
use sts2_protocol::{RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS, RuntimeV3GameplayMessage};

const RESPONSE: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json");
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

fn message_with(state: Value) -> Value {
    let mut value: Value = serde_json::from_str(RESPONSE).expect("golden is JSON");
    value["observation"]["state"] = state;
    value
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

/// A valid combat state whose single enemy is named `name`.
fn combat_with_enemy(name: &str) -> Value {
    message_with(json!({
        "state": "combat",
        "turn_index": 1,
        "enemies": [{
            "enemy_id": "enemy-1",
            "name": name,
            "hp": 12,
            "max_hp": 20,
            "intent": { "kind": "defend" },
        }],
    }))
}

/// A valid shop state whose single item is named `name`.
fn shop_with_item(name: &str) -> Value {
    message_with(json!({
        "state": "shop",
        "items": [{ "item_id": "item-1", "name": name, "price": 50 }],
    }))
}

/// A valid combat state whose player's hand holds a single card named `name`.
fn combat_with_hand_card(name: &str) -> Value {
    let mut document = message_with(json!({
        "state": "combat",
        "turn_index": 1,
        "enemies": [],
    }));
    document["observation"]["player"]["hand"] =
        json!([{ "card_id": "card-1", "name": name, "cost": 1, "upgraded": false }]);
    document
}

/// A valid combat state whose `visible_seed` is `seed`.
fn combat_with_seed(seed: &str) -> Value {
    let mut document = message_with(json!({
        "state": "combat",
        "turn_index": 1,
        "enemies": [],
    }));
    document["observation"]["visible_seed"] = json!(seed);
    document
}

/// 512 characters of a 2-byte character: 1024 UTF-8 bytes, so a byte-counting parser refuses a
/// string the schema admits.
fn multibyte_at_bound() -> String {
    let name = "\u{00e9}".repeat(RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS);
    assert!(name.len() > RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS);
    assert_eq!(
        name.chars().count(),
        RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS
    );
    name
}

fn multibyte_past_bound() -> String {
    let name = "\u{00e9}".repeat(RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS + 1);
    assert!(name.len() > RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS);
    assert_eq!(
        name.chars().count(),
        RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS + 1
    );
    name
}

#[test]
fn a_multibyte_name_at_the_character_bound_is_accepted() {
    let name = multibyte_at_bound();
    assert_parity(
        &combat_with_enemy(&name),
        true,
        "enemy name of 512 multi-byte characters",
    );
    assert_parity(
        &shop_with_item(&name),
        true,
        "shop item name of 512 multi-byte characters",
    );
    assert_parity(
        &combat_with_hand_card(&name),
        true,
        "card name of 512 multi-byte characters",
    );
    assert_parity(
        &combat_with_seed(&name),
        true,
        "visible_seed of 512 multi-byte characters",
    );
}

#[test]
fn a_multibyte_name_past_the_character_bound_is_refused() {
    let name = multibyte_past_bound();
    assert_parity(
        &combat_with_enemy(&name),
        false,
        "enemy name of 513 multi-byte characters",
    );
    assert_parity(
        &shop_with_item(&name),
        false,
        "shop item name of 513 multi-byte characters",
    );
    assert_parity(
        &combat_with_hand_card(&name),
        false,
        "card name of 513 multi-byte characters",
    );
    assert_parity(
        &combat_with_seed(&name),
        false,
        "visible_seed of 513 multi-byte characters",
    );
}

#[test]
fn an_ascii_name_at_the_bound_is_accepted_and_one_past_it_refused() {
    // The character bound must not have been widened: ASCII is one byte per character, so the
    // boundary is identical in both readings and must not move.
    let at_bound = "a".repeat(RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS);
    let past_bound = "a".repeat(RUNTIME_V3_GAMEPLAY_MAX_TEXT_CHARACTERS + 1);
    assert_parity(
        &combat_with_enemy(&at_bound),
        true,
        "ASCII enemy name exactly at the bound",
    );
    assert_parity(
        &combat_with_enemy(&past_bound),
        false,
        "ASCII enemy name one past the bound",
    );
    assert_parity(
        &shop_with_item(&at_bound),
        true,
        "ASCII shop name exactly at the bound",
    );
    assert_parity(
        &shop_with_item(&past_bound),
        false,
        "ASCII shop name one past the bound",
    );
}

#[test]
fn a_control_character_is_still_refused_at_any_length() {
    assert_parity(
        &combat_with_enemy("line\u{7}break"),
        false,
        "a control character in an enemy name",
    );
    assert_parity(
        &shop_with_item("line\u{7}break"),
        false,
        "a control character in a shop item name",
    );
    assert_parity(
        &combat_with_hand_card("line\u{7}break"),
        false,
        "a control character in a card name",
    );
    assert_parity(
        &combat_with_seed("line\u{7}break"),
        false,
        "a control character in visible_seed",
    );
}
