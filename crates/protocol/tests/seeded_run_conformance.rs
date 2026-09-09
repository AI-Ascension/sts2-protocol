// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    SEEDED_RUN_ARTIFACT, SEEDED_RUN_EFFECT_KIND, SEEDED_RUN_MAX_GENERATION,
    SEEDED_RUN_PROTOCOL_VERSION, SEEDED_RUN_SCHEMA_DIGEST, SEEDED_RUN_SCHEMA_SOURCE,
    SeededRunMessage, SeededRunMode, SeededRunStatus, SeededRunValidationError, canonical_json,
    decode_json,
};

const CASE: &str = include_str!("../../../conformance/cases/seeded-run-v1.json");
const MANIFEST: &str = include_str!("../../../artifacts/seeded-run-v1/manifest.json");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/seeded-run-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/seeded-run-v1/schema.json");

const GOLDENS: &[(&str, &str)] = &[
    (
        "start-request",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-request.json"),
    ),
    (
        "start-accepted",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-accepted.json"),
    ),
    (
        "start-settled",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-settled.json"),
    ),
    (
        "start-rejected",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-rejected.json"),
    ),
    (
        "start-unknown",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-unknown.json"),
    ),
    (
        "start-cancelled",
        include_str!("../../../artifacts/seeded-run-v1/golden/start-cancelled.json"),
    ),
    (
        "reconcile-request",
        include_str!("../../../artifacts/seeded-run-v1/golden/reconcile-request.json"),
    ),
    (
        "reconcile-settled",
        include_str!("../../../artifacts/seeded-run-v1/golden/reconcile-settled.json"),
    ),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn golden(name: &str) -> &str {
    GOLDENS
        .iter()
        .find_map(|(golden_name, text)| (*golden_name == name).then_some(*text))
        .expect("named seeded-run golden exists")
}

#[test]
fn seeded_run_goldens_round_trip_with_stable_bytes() {
    for (name, text) in GOLDENS {
        let message: SeededRunMessage = decode_json(text).expect("golden JSON is valid");
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
fn seeded_run_schema_and_artifact_bind_authoritative_metadata() {
    let source: Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    let manifest: Value = decode_json(MANIFEST).expect("manifest is JSON");
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-seeded-run-v1");
    assert_eq!(manifest["artifact"], SEEDED_RUN_ARTIFACT);
    assert_eq!(manifest["protocol_version"], SEEDED_RUN_PROTOCOL_VERSION);
    assert_eq!(manifest["schema_digest"], SEEDED_RUN_SCHEMA_DIGEST);
    assert_eq!(case["profile"], SEEDED_RUN_PROTOCOL_VERSION);
    assert_eq!(case["schema"], SEEDED_RUN_SCHEMA_SOURCE);
    assert_eq!(
        case["contract_assertions"]["selected_context_required_on_start_and_result"],
        json!(true)
    );
    assert_eq!(
        case["contract_assertions"]["nullable_wire_members_must_be_explicit"],
        json!(true)
    );

    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("seeded-run schema compiles as Draft 2020-12");
    for (name, text) in GOLDENS {
        let value: Value = decode_json(text).expect("golden is JSON");
        assert!(validator.is_valid(&value), "{name} must satisfy the schema");
    }

    let mut unknown: Value = decode_json(golden("start-settled")).expect("settled is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));

    let mut missing_nullable: Value =
        decode_json(golden("start-settled")).expect("settled is JSON");
    missing_nullable
        .as_object_mut()
        .expect("message is an object")
        .remove("selected_context");
    assert!(!validator.is_valid(&missing_nullable));
    let missing_text = serde_json::to_string(&missing_nullable).expect("JSON encodes");
    assert!(decode_json::<SeededRunMessage>(&missing_text).is_err());

    let mut missing_nested_nullable: Value =
        decode_json(golden("start-settled")).expect("settled is JSON");
    missing_nested_nullable["observation"]
        .as_object_mut()
        .expect("observation is an object")
        .remove("selected_context_digest");
    assert!(!validator.is_valid(&missing_nested_nullable));
    let missing_nested_text =
        serde_json::to_string(&missing_nested_nullable).expect("JSON encodes");
    assert!(decode_json::<SeededRunMessage>(&missing_nested_text).is_err());
}

#[test]
fn seeded_run_statuses_keep_admission_and_settlement_distinct() {
    let accepted: SeededRunMessage = decode_json(golden("start-accepted")).expect("accepted");
    assert_eq!(accepted.status, Some(SeededRunStatus::Accepted));
    assert!(accepted.observation.is_none());
    assert!(accepted.effect_witness.is_none());

    let settled: SeededRunMessage = decode_json(golden("start-settled")).expect("settled");
    let observation = settled.observation.as_ref().expect("observation");
    let witness = settled.effect_witness.as_ref().expect("witness");
    assert!(observation.run_started);
    assert_eq!(witness.kind, SEEDED_RUN_EFFECT_KIND);
    assert!(observation.generation > settled.generation);

    let mut without_witness = settled;
    without_witness.effect_witness = None;
    assert!(without_witness.validate().is_err());

    let mut over_bound: SeededRunMessage =
        decode_json(golden("start-settled")).expect("settled is JSON");
    over_bound.generation = SEEDED_RUN_MAX_GENERATION + 1;
    assert!(over_bound.validate().is_err());
}

#[test]
fn selected_context_binds_native_configuration_and_digest() {
    let settled: SeededRunMessage = decode_json(golden("start-settled")).expect("settled");
    let selected = settled.selected_context.as_ref().expect("selected context");
    assert_eq!(
        selected.game_mode,
        sts2_protocol::SeededRunGameMode::Standard
    );
    assert_eq!(
        selected.character,
        sts2_protocol::SeededRunCharacter::Ironclad
    );
    assert_eq!(selected.ascension, 0);
    assert!(selected.modifiers.is_empty());
    assert_eq!(selected.acts, ["act_1", "act_2", "act_3", "act_4"]);
    assert_eq!(
        selected.save_policy,
        sts2_protocol::SeededRunSavePolicy::Disabled
    );
    assert_eq!(
        selected.canonical_digest().unwrap(),
        selected.context_digest
    );
    assert_eq!(
        settled.context_digest.as_deref(),
        Some(selected.context_digest.as_str())
    );

    let mut reordered = selected.clone();
    reordered.acts.reverse();
    assert_eq!(
        reordered.validate(),
        Err(SeededRunValidationError::ContextDigestMismatch)
    );

    let mut tampered = settled.clone();
    tampered.selected_context.as_mut().unwrap().ascension = 1;
    assert_eq!(
        tampered.validate(),
        Err(SeededRunValidationError::ContextDigestMismatch)
    );

    let mut legacy_training = settled;
    legacy_training.selected_context = None;
    legacy_training.context_digest = None;
    assert_eq!(
        legacy_training.validate(),
        Err(SeededRunValidationError::ContextRequired)
    );

    assert_eq!(
        SeededRunMode::SeededTraining,
        legacy_training.run_mode.unwrap()
    );
}
