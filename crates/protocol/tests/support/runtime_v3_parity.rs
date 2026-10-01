// SPDX-License-Identifier: MIT

// Shared schema/parser parity helpers for the runtime-v3 gameplay conformance tests.
//
// A case is only meaningful if the schema and the parser reach the *same* verdict, because the two
// are separately normative and a disagreement in either direction is a real defect: the schema
// admitting what the parser refuses loses fail-closed behaviour, and the parser admitting what the
// schema refuses widens what the boundary takes. These helpers exist so each test file can state
// its case and have both sides checked.
//
// This file is pulled in with `include!`, so it carries no inner attributes and every item is
// public; unused helpers are expected across the two test files that share it.

use serde_json::Value;
use sts2_protocol::RuntimeV3GameplayMessage;

#[allow(dead_code)]
pub const BELT: &str = include_str!(
    "../../../../artifacts/runtime-v3-gameplay/golden/state-response-described-belt.json"
);
#[allow(dead_code)]
pub const RESPONSE: &str =
    include_str!("../../../../artifacts/runtime-v3-gameplay/golden/state-response.json");
#[allow(dead_code)]
pub const SCHEMA: &str = include_str!("../../../../schemas/runtime-v3-gameplay.schema.json");

#[allow(dead_code)]
pub fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

/// The plain `state_response` golden, which is the base every case varies one member of.
pub fn base() -> Value {
    serde_json::from_str(RESPONSE).expect("golden is JSON")
}

/// The player object of the fully described-belt golden, so a case can keep every optional member
/// present while changing one of them.
pub fn full_player() -> Value {
    base()["observation"]["player"].clone()
}

/// `base()` with `observation.player` replaced, so a case varies only the belt.
pub fn with_player(player: Value) -> Value {
    let mut document = base();
    document["observation"]["player"] = player;
    document
}

/// `base()` with `legal_actions` replaced, so a case varies only the offered actions.
#[allow(dead_code)]
pub fn with_legal_actions(legal_actions: Value) -> Value {
    let mut document = base();
    document["legal_actions"] = legal_actions;
    document
}

fn rust_accepts(value: &Value) -> bool {
    serde_json::from_value::<RuntimeV3GameplayMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

/// Asserts the schema and the parser reach the same verdict, which is the property under test.
pub fn assert_parity(document: &Value, expected: bool, context: &str) {
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
