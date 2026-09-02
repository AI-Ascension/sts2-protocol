// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    POC_ARTIFACT, POC_GENERATOR, POC_MAX_GENERATION, POC_PROTOCOL_VERSION, POC_SCHEMA_DIGEST,
    POC_SCHEMA_SOURCE, PocAction, PocActionResult, PocMessage, PocMessageKind, PocStatus,
    PocValidationError, canonical_json, decode_json,
};

const CASE: &str = include_str!("../../../conformance/cases/poc-v1.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/poc-v1/SHA256SUMS");
const STATE_REQUEST: &str = include_str!("../../../artifacts/poc-v1/golden/state-request.json");
const STATE: &str = include_str!("../../../artifacts/poc-v1/golden/state-response.json");
const ACTION_REQUEST: &str = include_str!("../../../artifacts/poc-v1/golden/action-request.json");
const ACCEPTED: &str = include_str!("../../../artifacts/poc-v1/golden/action-accepted.json");
const REJECTED: &str = include_str!("../../../artifacts/poc-v1/golden/action-rejected.json");
const INVALID: &str = include_str!("../../../artifacts/poc-v1/fixtures/invalid-action.json");
const MANIFEST: &str = include_str!("../../../artifacts/poc-v1/manifest.json");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/poc-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/poc-v1/schema.json");

fn golden_payload(golden: &str) -> &str {
    golden.strip_suffix('\n').unwrap_or(golden)
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
fn valid_goldens_round_trip_with_stable_bytes() {
    for fixture in [STATE_REQUEST, STATE, ACTION_REQUEST, ACCEPTED, REJECTED] {
        let message: PocMessage = decode_json(fixture).expect("golden POC JSON is valid");
        message.validate().expect("golden POC message is valid");
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            golden_payload(fixture)
        );
    }
}

#[test]
fn invalid_action_fixture_is_structural_and_reserved_for_core_legality() {
    let message: PocMessage = decode_json(INVALID).expect("invalid-action JSON is valid");
    message
        .validate()
        .expect("typed zero argument is in the wire range");
    assert_eq!(message.kind, PocMessageKind::ActionRequest);
    assert_eq!(
        message.action,
        Some(PocAction {
            action_id: "use_budget".to_owned(),
            units: 0
        })
    );
}

#[test]
fn constructors_preserve_contract_metadata_and_result_identity() {
    let metadata = sts2_protocol::PocMetadata {
        protocol_version: POC_PROTOCOL_VERSION.to_owned(),
        schema_digest: POC_SCHEMA_DIGEST.to_owned(),
        provenance: sts2_protocol::PocProvenance {
            artifact: POC_ARTIFACT.to_owned(),
            source: POC_SCHEMA_SOURCE.to_owned(),
            generator: POC_GENERATOR.to_owned(),
        },
    };
    let message = PocMessage::action_response(
        metadata,
        "corr-test",
        "instance-test",
        1,
        PocAction {
            action_id: "use_budget".to_owned(),
            units: 0,
        },
        PocActionResult {
            status: PocStatus::Rejected,
            observation: sts2_protocol::PocObservation {
                available_units: 3,
                settled_effects: 0,
            },
            error_code: Some("sts2.game-core/zero_units".to_owned()),
        },
    );
    message
        .validate()
        .expect("rejected result has an error identity");
    assert_eq!(
        message.error_code.as_deref(),
        Some("sts2.game-core/zero_units")
    );
}

#[test]
fn release_like_schema_validates_all_fixtures_and_rejects_ambiguous_shapes() {
    let source: serde_json::Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: serde_json::Value =
        decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-poc-v1");
    assert_eq!(
        source["$defs"]["base"]["properties"]["generation"]["maximum"],
        json!(POC_MAX_GENERATION)
    );

    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("POC schema compiles as Draft 2020-12");
    for fixture in [
        STATE_REQUEST,
        STATE,
        ACTION_REQUEST,
        ACCEPTED,
        REJECTED,
        INVALID,
    ] {
        let value: Value = decode_json(fixture).expect("fixture is JSON");
        assert!(
            validator.is_valid(&value),
            "fixture must satisfy the schema"
        );
    }

    let mut unknown: Value = decode_json(STATE).expect("golden is JSON");
    unknown
        .as_object_mut()
        .expect("message is an object")
        .insert("unexpected".to_owned(), json!(true));
    assert!(!validator.is_valid(&unknown));

    let mut missing: Value = decode_json(STATE).expect("golden is JSON");
    missing
        .as_object_mut()
        .expect("message is an object")
        .remove("error_code");
    assert!(!validator.is_valid(&missing));

    let mut over_generation: Value = decode_json(STATE).expect("golden is JSON");
    over_generation["generation"] = json!(POC_MAX_GENERATION + 1);
    assert!(!validator.is_valid(&over_generation));
}

#[test]
fn decoder_rejects_missing_nullable_fields_and_unknown_fields() {
    let explicit_null = decode_json::<PocMessage>(STATE).expect("explicit null is accepted");
    assert_eq!(explicit_null.error_code, None);

    let mut missing: Value = decode_json(STATE).expect("golden is JSON");
    missing
        .as_object_mut()
        .expect("message is an object")
        .remove("error_code");
    let missing_result = decode_json::<PocMessage>(&missing.to_string());
    assert!(
        missing_result.is_err(),
        "missing decoded as {missing_result:?}"
    );

    let mut unknown: Value = decode_json(STATE).expect("golden is JSON");
    unknown
        .as_object_mut()
        .expect("message is an object")
        .insert("unexpected".to_owned(), json!(true));
    assert!(decode_json::<PocMessage>(&unknown.to_string()).is_err());

    let mut nested_unknown: Value = decode_json(ACCEPTED).expect("golden is JSON");
    nested_unknown["action"]["unexpected"] = json!(true);
    assert!(decode_json::<PocMessage>(&nested_unknown.to_string()).is_err());
}

#[test]
fn generation_bound_is_enforced_by_rust() {
    let metadata = sts2_protocol::PocMetadata {
        protocol_version: POC_PROTOCOL_VERSION.to_owned(),
        schema_digest: POC_SCHEMA_DIGEST.to_owned(),
        provenance: sts2_protocol::PocProvenance {
            artifact: POC_ARTIFACT.to_owned(),
            source: POC_SCHEMA_SOURCE.to_owned(),
            generator: POC_GENERATOR.to_owned(),
        },
    };
    let mut message = PocMessage::state_request(metadata, "corr-test", "instance-test");
    message.generation = POC_MAX_GENERATION;
    message.validate().expect("maximum generation is valid");
    message.generation = POC_MAX_GENERATION + 1;
    assert_eq!(
        message.validate(),
        Err(PocValidationError::GenerationBounds)
    );
}

#[test]
fn conformance_case_and_manifest_bind_the_complete_artifact() {
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    assert_eq!(case["case_id"], "CT-POC-V1-001");
    assert_eq!(case["schema"], POC_SCHEMA_SOURCE);
    assert_eq!(case["checksums"], "artifacts/poc-v1/SHA256SUMS");
    assert_eq!(case["goldens"].as_array().map(Vec::len), Some(5));

    let manifest: serde_json::Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(manifest["artifact"], POC_ARTIFACT);
    assert_eq!(manifest["protocol_version"], POC_PROTOCOL_VERSION);
    assert_eq!(manifest["schema"], "schema.json");
    assert_eq!(manifest["schema_digest"], POC_SCHEMA_DIGEST);
    assert_eq!(manifest["provenance"]["source"], POC_SCHEMA_SOURCE);
    assert_eq!(manifest["provenance"]["generator"], POC_GENERATOR);
    assert_eq!(manifest["provenance"]["license"], "MIT");
    assert_eq!(
        manifest["consumers"],
        json!([
            "sts2-game-core",
            "sts2-game-mod",
            "sts2-gateway",
            "sts2-harness",
            "sts2-mcp-server"
        ])
    );
    assert_eq!(checksum_for(CHECKSUMS, "schema.json"), POC_SCHEMA_DIGEST);
    assert_eq!(
        checksum_for(CHECKSUMS, "../../schemas/poc-v1.schema.json"),
        POC_SCHEMA_DIGEST
    );
}
