// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    GameObservation, LegalAction, RuntimeV3GameplayAction, RuntimeV3GameplayLegalAction,
    RuntimeV3GameplayMessage, RuntimeV3GameplayValidationError, canonical_json, decode_json,
};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-v3-gameplay/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v3-gameplay.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v3-gameplay/SHA256SUMS");
const GOLDENS: &[(&str, &str)] = &[
    (
        "state-request",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-request.json"),
    ),
    (
        "state-response",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json"),
    ),
    (
        "dispatch-action-request",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/dispatch-action-request.json"),
    ),
    (
        "dispatch-action-settled",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/dispatch-action-settled.json"),
    ),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn checksum_for<'a>(inventory: &'a str, path: &str) -> &'a str {
    inventory
        .lines()
        .find_map(|line| {
            let (digest, listed_path) = line.split_once("  ")?;
            (listed_path == path).then_some(digest)
        })
        .expect("checksum inventory contains the requested path")
}

#[test]
fn runtime_v3_gameplay_goldens_validate_and_round_trip() {
    for (name, text) in GOLDENS {
        let message: RuntimeV3GameplayMessage = decode_json(text).expect("golden is JSON");
        message
            .validate()
            .unwrap_or_else(|error| panic!("{name} must validate: {error}"));
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            payload(text)
        );
    }
}

#[test]
fn runtime_v3_gameplay_schema_and_case_are_bound_to_artifact() {
    let source: Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source, artifact);
    assert_eq!(source["$id"], "sts2-runtime-v3-gameplay");
    assert_eq!(case["contract"], "sts2.protocol/runtime-v3-gameplay");
    assert_eq!(
        case["checksums"],
        "artifacts/runtime-v3-gameplay/SHA256SUMS"
    );
    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("schema compiles as Draft 2020-12");
    for (_, text) in GOLDENS {
        let value: Value = decode_json(text).expect("golden is JSON");
        assert!(validator.is_valid(&value));
    }
    let mut unknown: Value = decode_json(GOLDENS[0].1).expect("request is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));
    let mut nested: Value = decode_json(GOLDENS[1].1).expect("response is JSON");
    nested["observation"]["privileged_rng"] = json!(123);
    assert!(!validator.is_valid(&nested));
}

#[test]
fn runtime_v3_gameplay_checksum_inventory_covers_contract_inputs() {
    for path in [
        "../../conformance/cases/runtime-v3-gameplay.json",
        "../../schemas/runtime-v3-gameplay.schema.json",
        "manifest.json",
        "schema.json",
        "golden/state-request.json",
        "golden/state-response.json",
        "golden/dispatch-action-request.json",
        "golden/dispatch-action-settled.json",
    ] {
        assert_eq!(checksum_for(CHECKSUMS, path).len(), 64, "{path}");
    }
}

#[test]
fn runtime_v3_gameplay_rejects_duplicate_or_malformed_legal_actions() {
    let action = LegalAction {
        action_id: "combat.end-turn".to_owned(),
        action: RuntimeV3GameplayAction::EndTurn,
    };
    let duplicate = RuntimeV3GameplayMessage {
        legal_actions: Some(vec![action.clone(), action]),
        ..decode_json(GOLDENS[1].1).expect("response is JSON")
    };
    assert_eq!(
        duplicate.validate(),
        Err(RuntimeV3GameplayValidationError::DuplicateAction)
    );
    let invalid = RuntimeV3GameplayLegalAction {
        action_id: "not valid".to_owned(),
        action: RuntimeV3GameplayAction::EndTurn,
    };
    assert_eq!(
        invalid.validate(),
        Err(RuntimeV3GameplayValidationError::InvalidIdentity)
    );
    let observation: GameObservation = decode_json(
        &serde_json::to_string(
            &decode_json::<RuntimeV3GameplayMessage>(GOLDENS[1].1)
                .expect("response is JSON")
                .observation,
        )
        .expect("observation encodes"),
    )
    .expect("observation decodes");
    assert_eq!(observation.state_id, "combat-1");
}
