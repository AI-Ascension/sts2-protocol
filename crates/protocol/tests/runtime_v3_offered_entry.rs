// SPDX-License-Identifier: MIT

//! Conformance for the described offered-entry contract.
//!
//! These tests exist because the schema and the Rust parser must agree about exactly which offered
//! sets are legal. A disagreement in either direction is a real defect: the schema admits something
//! the parser refuses and no producer can emit it, or the parser admits something the schema refuses
//! and every consumer rejects it on arrival. Each case below is therefore checked against both, and
//! any case where they diverge fails the test rather than documenting the gap.

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V3_GAMEPLAY_MAX_ENTITIES, RUNTIME_V3_GAMEPLAY_MAX_OFFERED_ATTRIBUTE_CHARACTERS,
    RuntimeV3GameplayChoice, RuntimeV3GameplayMessage, RuntimeV3GameplayOfferedEntry,
    RuntimeV3GameplayState, RuntimeV3GameplayValidationError,
};

const RESPONSE: &str =
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json");
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

/// A whole valid message carrying `state` as its observation, so every case varies only the state.
fn message_with(state: Value) -> Value {
    let mut value: Value = serde_json::from_str(RESPONSE).expect("golden is JSON");
    value["observation"]["state"] = state;
    value
}

/// A reward state whose `options` is `options`.
fn reward(options: Value) -> Value {
    json!({ "state": "reward", "options": options })
}

/// Whether the Rust parser accepts the message, binding parsing and validation together.
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

/// Asserts the parser's verdict and records where the schema deliberately does not enforce it.
///
/// The schema is a structural vocabulary; uniqueness is a semantic rule this repository has always
/// left to `validate()` — no array in this contract carries `uniqueItems`, and duplicates are
/// caught by `DuplicateAction`, `DuplicateChoice` and their siblings. Duplicating that vocabulary
/// in the schema for one array and not the others would make two arrays in one contract disagree
/// about where a rule lives, so these cases assert the parser and say which layer owns the rule.
fn assert_parser_refuses(document: &Value, context: &str) {
    assert!(
        !rust_accepts(document),
        "the parser must refuse {context}: {document}"
    );
}

#[test]
fn a_bare_identity_stays_a_valid_offer() {
    // The form every producer emitted before this contract existed must keep validating, or
    // widening the vocabulary would break every conforming producer at once.
    assert_parity(
        &message_with(reward(json!(["card:21:Setup-Strike", "card:22:Tremble"]))),
        true,
        "bare identities",
    );
}

#[test]
fn a_fully_described_entry_validates_and_round_trips() {
    let entry = RuntimeV3GameplayOfferedEntry {
        choice_id: "card:23:Blood-Wall".to_owned(),
        name: Some("Blood Wall".to_owned()),
        cost: Some(2),
        upgraded: Some(true),
        rarity: Some("rare".to_owned()),
        description: Some("Gain 8 block.".to_owned()),
        contents: None,
    };
    let message = message_with(reward(json!([serde_json::to_value(&entry).unwrap()])));
    assert_parity(&message, true, "fully described entry");

    let choice: RuntimeV3GameplayChoice = serde_json::from_value(entry_value()).unwrap();
    let RuntimeV3GameplayChoice::Described(described) = &choice else {
        panic!("a described entry must not decode as a bare identity");
    };
    assert_eq!(described.name.as_deref(), Some("Blood Wall"));
    assert_eq!(described.cost, Some(2));
    assert_eq!(described.upgraded, Some(true));
    assert_eq!(described.rarity.as_deref(), Some("rare"));
    assert_eq!(described.description.as_deref(), Some("Gain 8 block."));
    assert_eq!(choice.choice_id(), "card:23:Blood-Wall");
    assert_eq!(
        serde_json::from_value::<RuntimeV3GameplayChoice>(serde_json::to_value(&choice).unwrap())
            .unwrap(),
        choice
    );
}

#[test]
fn an_omitted_attribute_stays_omitted_after_serialization() {
    // Absence is representable and distinct from a plausible value. If an omitted attribute came
    // back as `null`, `""`, or `false`, a reader could no longer tell "the host said nothing" from
    // "the host said the cheapest, rarest option", which is the exact inference this contract
    // refuses to make on its behalf.
    let entry = RuntimeV3GameplayOfferedEntry::named("card:24:Quiet");
    let serialized = serde_json::to_value(&entry).unwrap();
    assert_eq!(serialized, json!({ "choice_id": "card:24:Quiet" }));
    for attribute in [
        "name",
        "cost",
        "upgraded",
        "rarity",
        "description",
        "contents",
    ] {
        assert!(
            serialized.get(attribute).is_none(),
            "{attribute} must not appear when the host supplied nothing"
        );
    }
}

#[test]
fn an_explicit_null_attribute_is_refused() {
    // `#[serde(default)]` on its own would accept `null`, but the schema's optional member does not.
    // Absent means the host said nothing; `null` is not a second spelling of absent.
    for attribute in [
        "name",
        "cost",
        "upgraded",
        "rarity",
        "description",
        "contents",
    ] {
        let entry = json!({ "choice_id": "card:25:Nullish", attribute: Value::Null });
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            &format!("null {attribute}"),
        );
    }
}

#[test]
fn a_cost_outside_the_bound_or_outside_its_type_is_refused() {
    for cost in [json!(256), json!(-1), json!("2"), json!(1.5)] {
        let entry = json!({ "choice_id": "card:39:Costly", "cost": cost });
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            "out-of-range cost",
        );
    }
    assert_parity(
        &message_with(reward(
            json!([{"choice_id": "card:39:Costly", "cost": 255}]),
        )),
        true,
        "a cost at the bound",
    );
}

#[test]
fn an_unknown_member_is_refused() {
    for member in ["effect", "tags", "exhausts", "Choice_id"] {
        let mut entry = json!({ "choice_id": "card:26:Extra" });
        entry[member] = json!("anything");
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            &format!("unknown member {member}"),
        );
    }
}

#[test]
fn an_empty_or_invalid_identity_is_refused() {
    for choice_id in ["", "card 21", "card:21\nInjected"] {
        let entry = json!({ "choice_id": choice_id });
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            &format!("identity {choice_id:?}"),
        );
    }
    // `card/../escape` is inside the identity alphabet this contract has always used for the bare
    // form, so both sides admit it. It is listed here to record that the widening did not quietly
    // tighten what an identity may contain.
    assert_parity(
        &message_with(reward(json!([{ "choice_id": "card/../escape" }]))),
        true,
        "an identity inside the established alphabet",
    );
}

#[test]
fn oversized_or_control_attribute_text_is_refused() {
    let long = "a".repeat(RUNTIME_V3_GAMEPLAY_MAX_OFFERED_ATTRIBUTE_CHARACTERS + 1);
    let at_bound = "a".repeat(RUNTIME_V3_GAMEPLAY_MAX_OFFERED_ATTRIBUTE_CHARACTERS);
    // A character-count bound must admit exactly the bound, or the parser would be stricter than
    // the schema it is written against.
    assert_parity(
        &message_with(reward(
            json!([{"choice_id": "card:27:Bound", "name": at_bound}]),
        )),
        true,
        "attribute exactly at the character bound",
    );
    for attribute in ["name", "rarity", "description"] {
        let entry = json!({ "choice_id": "card:27:Long", attribute: long });
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            &format!("oversized {attribute}"),
        );
        let entry = json!({ "choice_id": "card:27:Control", attribute: "line\u{7}break" });
        assert_parity(
            &message_with(reward(json!([entry]))),
            false,
            &format!("control character in {attribute}"),
        );
    }
}

#[test]
fn a_non_ascii_attribute_is_bounded_by_characters_not_bytes() {
    // JSON Schema's maxLength counts characters. If the parser counted UTF-8 bytes it would refuse
    // a 512-character non-ASCII name that the schema admits, and the two would disagree.
    let name = "\u{00e9}".repeat(RUNTIME_V3_GAMEPLAY_MAX_OFFERED_ATTRIBUTE_CHARACTERS);
    assert!(name.len() > RUNTIME_V3_GAMEPLAY_MAX_OFFERED_ATTRIBUTE_CHARACTERS);
    assert_parity(
        &message_with(reward(
            json!([{"choice_id": "card:28:Accented", "name": name}]),
        )),
        true,
        "512-character non-ASCII name",
    );
}

#[test]
fn a_duplicate_identity_is_refused_by_rust_validation() {
    let document = message_with(reward(json!(["card:29:Same", "card:29:Same"])));
    assert_parser_refuses(&document, "a duplicate identity in a bare list");

    // The same identity offered twice is still one identity offered twice when one copy describes
    // it, so the duplicate rule has to hold across the mixed form as well.
    let document = message_with(reward(json!([
        "card:29:Same",
        { "choice_id": "card:29:Same", "rarity": "rare" }
    ])));
    assert_parser_refuses(
        &document,
        "a duplicate identity across bare and described forms",
    );

    let message = serde_json::from_value::<RuntimeV3GameplayMessage>(document).unwrap();
    assert_eq!(
        message.validate(),
        Err(RuntimeV3GameplayValidationError::DuplicateChoice)
    );
}

#[test]
fn a_mixed_list_of_bare_and_described_entries_validates() {
    assert_parity(
        &message_with(reward(json!([
            "card:30:Bare",
            { "choice_id": "card:31:Described", "name": "Described" },
            "card:32:AlsoBare"
        ]))),
        true,
        "mixed bare and described list",
    );
}

#[test]
fn contents_are_admitted_exactly_one_level() {
    let one_level = json!([{
        "choice_id": "card:33:Chest",
        "name": "Chest",
        "contents": [
            "card:34:Gold",
            { "choice_id": "card:35:Relic", "rarity": "starter" }
        ]
    }]);
    assert_parity(
        &message_with(reward(one_level)),
        true,
        "one level of disclosed contents",
    );

    // A second level is refused by both sides: the schema's disclosed entry has no `contents`
    // member, and the parser refuses rather than ignoring what a producer tried to nest.
    let two_levels = json!([{
        "choice_id": "card:33:Chest",
        "contents": [{
            "choice_id": "card:36:Inner",
            "contents": ["card:37:TooDeep"]
        }]
    }]);
    let document = message_with(reward(two_levels));
    assert_parity(&document, false, "contents nested two levels deep");
    let message = serde_json::from_value::<RuntimeV3GameplayMessage>(document).unwrap();
    assert_eq!(
        message.validate(),
        Err(RuntimeV3GameplayValidationError::OfferedDisclosureTooDeep)
    );
}

#[test]
fn contents_are_bounded() {
    let too_many: Vec<String> = (0..=RUNTIME_V3_GAMEPLAY_MAX_ENTITIES)
        .map(|index| format!("card:{index}:Item"))
        .collect();
    let document = message_with(reward(
        json!([{ "choice_id": "card:38:Overflow", "contents": too_many }]),
    ));
    assert_parity(&document, false, "disclosed contents beyond the bound");

    let too_many_top: Vec<String> = (0..=RUNTIME_V3_GAMEPLAY_MAX_ENTITIES)
        .map(|index| format!("card:{index}:Top"))
        .collect();
    assert_parity(
        &message_with(reward(json!(too_many_top))),
        false,
        "offered set beyond the bound",
    );
}

#[test]
fn an_event_and_a_selection_carry_the_same_offered_vocabulary() {
    // The three offered members are one decision surface, so the same list must be legal in each.
    let options = json!(["event:1:Leave", { "choice_id": "event:2:Search", "rarity": "uncommon", "cost": 1 }]);
    for kind in ["event", "selection", "reward"] {
        let member = if kind == "reward" {
            "options"
        } else {
            "choices"
        };
        let state = json!({ "state": kind, member: options.clone() });
        assert_parity(
            &message_with(state),
            true,
            &format!("offered set in {kind}"),
        );
    }
}

#[test]
fn the_rest_screen_stays_identity_only_and_says_why() {
    // The rest options are deliberately not widened. No producer discloses that set today, so a
    // described form there would be capacity with no producer, and it is refused rather than
    // silently accepted to keep the record honest about what was deliberately left alone.
    assert_parity(
        &message_with(json!({ "state": "rest", "options": ["rest:1:Sleep"] })),
        true,
        "bare rest options",
    );
    assert_parity(
        &message_with(json!({
            "state": "rest",
            "options": [{ "choice_id": "rest:1:Sleep", "rarity": "rare" }]
        })),
        false,
        "described rest options",
    );
}

#[test]
fn a_bare_identity_is_never_widened_into_a_described_entry() {
    // Round-tripping a bare choice must not invent the description shape, or a reader could no
    // longer distinguish a host that described an entry from one that only named it.
    let document = message_with(reward(json!(["card:40:Plain"])));
    let message = serde_json::from_value::<RuntimeV3GameplayMessage>(document).unwrap();
    let RuntimeV3GameplayState::Reward { options } = &message
        .observation
        .as_ref()
        .expect("golden carries an observation")
        .state
    else {
        panic!("fixture must decode as a reward state");
    };
    assert_eq!(options.len(), 1);
    assert!(matches!(options[0], RuntimeV3GameplayChoice::Named(_)));
    assert_eq!(options[0].described(), None);
    assert_eq!(options[0].choice_id(), "card:40:Plain");

    let serialized = serde_json::to_value(&options[0]).unwrap();
    assert_eq!(serialized, json!("card:40:Plain"));
}

fn entry_value() -> Value {
    json!({
        "choice_id": "card:23:Blood-Wall",
        "name": "Blood Wall",
        "cost": 2,
        "upgraded": true,
        "rarity": "rare",
        "description": "Gain 8 block."
    })
}
