// SPDX-License-Identifier: MIT

use sts2_protocol::{
    POC_ARTIFACT, POC_GENERATOR, POC_PROTOCOL_VERSION, POC_SCHEMA_DIGEST, POC_SCHEMA_SOURCE,
    PocAction, PocActionResult, PocMessage, PocMessageKind, PocStatus, canonical_json, decode_json,
};

const STATE: &str = include_str!("../../../artifacts/poc-v1/golden/state-response.json");
const ACCEPTED: &str = include_str!("../../../artifacts/poc-v1/golden/action-accepted.json");
const INVALID: &str = include_str!("../../../artifacts/poc-v1/fixtures/invalid-action.json");
const MANIFEST: &str = include_str!("../../../artifacts/poc-v1/manifest.json");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/poc-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/poc-v1/schema.json");

#[test]
fn valid_goldens_round_trip_with_stable_bytes() {
    for fixture in [STATE, ACCEPTED] {
        let message: PocMessage = decode_json(fixture).expect("golden POC JSON is valid");
        message.validate().expect("golden POC message is valid");
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            fixture.trim()
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
fn release_like_schema_is_json_and_semantically_equal_to_source() {
    let source: serde_json::Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: serde_json::Value =
        decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(source["$id"], "sts2-poc-v1");
}

#[test]
fn release_manifest_binds_version_digest_provenance_and_consumers() {
    let manifest: serde_json::Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(manifest["artifact"], POC_ARTIFACT);
    assert_eq!(manifest["protocol_version"], POC_PROTOCOL_VERSION);
    assert_eq!(manifest["schema_digest"], POC_SCHEMA_DIGEST);
    assert_eq!(manifest["provenance"]["generator"], POC_GENERATOR);
    assert_eq!(manifest["consumers"].as_array().map(Vec::len), Some(5));
}
