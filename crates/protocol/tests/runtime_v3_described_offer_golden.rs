// SPDX-License-Identifier: MIT

//! Conformance for the described-offer golden itself.
//!
//! The golden is the artifact a consumer is actually handed, so it is worth pinning separately from
//! the parser rules: a golden that merely parsed would prove the shape is tolerated, not that the
//! three entry forms and one level of disclosure are what a consumer will really see.

use serde_json::{Value, json};
use sts2_protocol::{RuntimeV3GameplayChoice, RuntimeV3GameplayMessage, RuntimeV3GameplayState};

const DESCRIBED_GOLDEN: &str = include_str!(
    "../../../artifacts/runtime-v3-gameplay/golden/state-response-described-offer.json"
);
const SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

fn golden() -> Value {
    serde_json::from_str(DESCRIBED_GOLDEN).expect("golden is JSON")
}

fn rust_accepts(value: &Value) -> bool {
    serde_json::from_value::<RuntimeV3GameplayMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

/// Asserts the schema and the parser agree about the golden as emitted, and as mutated.
fn assert_parity(document: &Value, expected: bool, context: &str) {
    let by_schema = validator().is_valid(document);
    assert_eq!(
        by_schema, expected,
        "schema disagreed with the expectation for {context}"
    );
    assert_eq!(
        by_schema,
        rust_accepts(document),
        "schema and parser disagreed for {context}"
    );
}

/// Asserts the parser refuses a document, recording that uniqueness is a `validate()` rule and
/// not a schema one, exactly as `DuplicateAction` is elsewhere in this contract.
fn assert_parser_refuses(document: &Value, context: &str) {
    assert!(!rust_accepts(document), "the parser must refuse {context}");
}

/// The reward options of the decoded golden.
fn options() -> Vec<RuntimeV3GameplayChoice> {
    let message = serde_json::from_value::<RuntimeV3GameplayMessage>(golden()).unwrap();
    let RuntimeV3GameplayState::Reward { options } = &message
        .observation
        .as_ref()
        .expect("golden carries an observation")
        .state
    else {
        panic!("the golden must decode as a reward state");
    };
    options.clone()
}

#[test]
fn the_described_offer_golden_carries_all_three_forms() {
    // The golden is the artifact a consumer is handed, so it has to exercise the bare form, the
    // fully described form, the described form with only some attributes supplied, and one level of
    // disclosure. A golden that only proved "it parses" would not pin the shape.
    let document: Value = serde_json::from_str(DESCRIBED_GOLDEN).expect("golden is JSON");
    assert_parity(&document, true, "the described-offer golden as emitted");

    let options = options();
    assert_eq!(options.len(), 3, "bare, described, and partially described");

    assert!(
        matches!(&options[0], RuntimeV3GameplayChoice::Named(identity) if identity == "card:21:Setup-Strike")
    );

    let described = options[1]
        .described()
        .expect("the second entry is described");
    assert_eq!(described.name.as_deref(), Some("Tremble"));
    assert_eq!(described.cost, Some(1));
    assert_eq!(described.rarity.as_deref(), Some("uncommon"));
    assert_eq!(
        described.upgraded, None,
        "the host supplied no upgrade state"
    );
    let contents = described
        .contents
        .as_ref()
        .expect("the entry discloses contents");
    assert!(
        matches!(&contents[0], RuntimeV3GameplayChoice::Named(identity) if identity == "card:23:Blood-Wall")
    );
    let nested = contents[1]
        .described()
        .expect("the disclosed entry is described");
    assert_eq!(nested.name.as_deref(), Some("Block"));
    assert!(nested.contents.is_none(), "disclosure stops at one level");

    let partial = options[2]
        .described()
        .expect("the third entry is described");
    assert_eq!(partial.upgraded, Some(true));
    assert_eq!(partial.name, None, "an unsupplied attribute stays absent");

    // The legal action names the identity a described entry is acted on under, which is what ties
    // the description to something actionable.
    assert_eq!(options[1].choice_id(), "card:22:Tremble");
}

#[test]
fn the_described_offer_golden_is_refused_once_broken() {
    let golden: Value = serde_json::from_str(DESCRIBED_GOLDEN).expect("golden is JSON");
    let mut nulled = golden.clone();
    nulled["observation"]["state"]["options"][1]["rarity"] = Value::Null;
    assert_parity(&nulled, false, "the golden with a nulled rarity");

    let mut unknown = golden.clone();
    unknown["observation"]["state"]["options"][1]["effect"] = json!("stun");
    assert_parity(&unknown, false, "the golden with an unknown member");

    let mut duplicated = golden.clone();
    let first = duplicated["observation"]["state"]["options"][0].clone();
    duplicated["observation"]["state"]["options"][2] = first;
    assert_parser_refuses(&duplicated, "the golden with a duplicated offered identity");
}
