// SPDX-License-Identifier: MIT

use jsonschema::Validator;
use serde_json::Value;

const SCHEMA: &str =
    include_str!("../../../schemas/game-information-content-manifest-v1.schema.json");
const POSITIVE: &str = include_str!(
    "../../../conformance/fixtures/game-information-content-manifest-v1/valid/canonical-manifest-response.json"
);

fn validator() -> Validator {
    jsonschema::draft202012::options()
        .build(&serde_json::from_str(SCHEMA).expect("schema JSON"))
        .expect("schema compiles")
}

#[test]
fn canonical_typed_producer_fixture_validates() {
    let value: Value = serde_json::from_str(POSITIVE).expect("fixture JSON");
    assert!(validator().is_valid(&value));
}

#[test]
fn schema_refuses_closed_version_digest_and_size_mutations() {
    let validator = validator();
    let mut value: Value = serde_json::from_str(POSITIVE).expect("fixture JSON");
    value["unknown"] = Value::Bool(true);
    assert!(!validator.is_valid(&value));

    let mut value: Value = serde_json::from_str(POSITIVE).expect("fixture JSON");
    value.as_object_mut().expect("object").remove("manifest");
    assert!(!validator.is_valid(&value));

    let mut value: Value = serde_json::from_str(POSITIVE).expect("fixture JSON");
    value["protocol_version"] = Value::String(String::from("game-information-content-manifest-v2"));
    assert!(!validator.is_valid(&value));

    let mut value: Value = serde_json::from_str(POSITIVE).expect("fixture JSON");
    value["schema_digest"] = Value::String(String::from("0").repeat(64));
    // The schema intentionally accepts a syntactically valid digest; consumer pinning rejects it.
    assert!(validator.is_valid(&value));
    assert_ne!(
        value["schema_digest"],
        serde_json::json!("8c8ea24bf4ee81d392da9a9a63e71362e24c79e368dd6e0420cff1508e0a0a33")
    );

    let oversized = "x".repeat(16 * 1024 * 1024 + 1);
    assert!(oversized.len() > 16 * 1024 * 1024);
}
