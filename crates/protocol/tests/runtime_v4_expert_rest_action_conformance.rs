// SPDX-License-Identifier: MIT

use std::fs;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sts2_protocol::{
    RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT,
    RUNTIME_V4_EXPERT_REST_ACTION_EFFECT_WITNESS_VERSION, RUNTIME_V4_EXPERT_REST_ACTION_GENERATOR,
    RUNTIME_V4_EXPERT_REST_ACTION_PROFILE, RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION,
    RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_SOURCE,
    RUNTIME_V4_EXPERT_SCHEMA_DIGEST, decode_json,
};

const SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/runtime-v4-expert-rest-action-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/runtime-v4-expert-rest-action/schema.json");
const EXPERT_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert.schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v4-expert-rest-action-v1.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/runtime-v4-expert-rest-action/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v4-expert-rest-action/SHA256SUMS");

#[path = "runtime_v4_expert_rest_action_conformance/support.rs"]
mod support;

use support::{
    PRODUCER_FIXTURES, artifact_root, fixture_names, golden, mutation, nested_observation_is_valid,
    producer_fixture, request_actions, schema_validator, selector_admissions, strict_semantics,
};

const SEMANTIC_GAP_MUTATIONS: [&str; 6] = [
    "action-completed-untested-option-witness.json",
    "action-completed-heal-noop.json",
    "action-selection-requested-no-choice-action.json",
    "action-selection-progressed-unlisted-choice.json",
    "action-selection-requested-option-kind-mismatch.json",
    "action-mend-selection-completed-absent-player.json",
];

#[test]
fn metadata_case_and_manifest_pin_candidate_identity_without_claiming_consumers() {
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    let manifest: Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(case["status"], "candidate");
    assert_eq!(case["consumers"], serde_json::json!([]));
    assert_eq!(manifest["status"], "candidate");
    assert_eq!(manifest["consumers"], serde_json::json!([]));
    assert_eq!(
        manifest["schema_digest"],
        RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST
    );
    assert_eq!(
        case["contract_assertions"]["old_potion_profile_unchanged"],
        true
    );
    assert_eq!(
        case["contract_assertions"]["old_potion_schema_digest"],
        RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST
    );
    assert_eq!(manifest["goldens"].as_array().unwrap().len(), 16);
    assert_eq!(manifest["negative_fixtures"].as_array().unwrap().len(), 22);
    assert_eq!(manifest["producer_fixtures"].as_array().unwrap().len(), 2);
    assert_eq!(case["fixtures"]["producer"].as_array().unwrap().len(), 2);
    assert_eq!(case["http_assignment"]["request"]["method"], "POST");
    assert_eq!(case["http_assignment"]["reconcile"]["method"], "GET");
    assert_eq!(case["http_assignment"]["reconcile"]["body"], Value::Null);
    assert_eq!(case["http_assignment"]["status_mapping"]["202"], "accepted");
    assert_eq!(case["http_assignment"]["status_mapping"]["200"], "settled");
}

#[test]
fn settled_goldens_enforce_cross_field_identity_counts_and_witnesses() {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let schema = schema_validator(SOURCE_SCHEMA);
    let expert = schema_validator(EXPERT_SCHEMA);
    let requests = request_actions();
    let admissions = selector_admissions();
    let names = fixture_names("golden");
    for name in names {
        let value = golden(&name);
        assert!(schema.is_valid(&value), "{name} schema");
        assert!(
            nested_observation_is_valid(&value, &expert),
            "{name} observation"
        );
        assert!(
            strict_semantics(&value, &requests, &admissions).is_some_and(|valid| valid),
            "{name} strict semantics"
        );
    }
}

#[test]
fn independent_mutations_are_rejected_by_schema_or_semantic_conformance() {
    let schema = schema_validator(SOURCE_SCHEMA);
    let requests = request_actions();
    let admissions = selector_admissions();
    let names = fixture_names("../../conformance/mutations/runtime-v4-expert-rest-action-v1");
    for name in names {
        let value = mutation(&name);
        if schema.is_valid(&value) {
            assert!(
                !strict_semantics(&value, &requests, &admissions).is_some_and(|valid| valid),
                "{name} bypassed strict semantics"
            );
        }
    }
}

#[test]
fn semantic_gap_mutations_are_schema_valid_and_rejected_by_rust_helpers() {
    let schema = schema_validator(SOURCE_SCHEMA);
    let requests = request_actions();
    let admissions = selector_admissions();
    for name in SEMANTIC_GAP_MUTATIONS {
        let value = mutation(name);
        assert!(schema.is_valid(&value), "{name} schema");
        assert!(
            !strict_semantics(&value, &requests, &admissions).is_some_and(|valid| valid),
            "{name} bypassed Rust strict semantics"
        );
    }
}

#[test]
fn serialized_producer_fixtures_bind_lifecycle_and_admission_catalogs() {
    let schema = schema_validator(SOURCE_SCHEMA);
    let expert = schema_validator(EXPERT_SCHEMA);
    let requests = request_actions();
    let admissions = selector_admissions();
    for name in PRODUCER_FIXTURES {
        let fixture = producer_fixture(name);
        assert_eq!(
            fixture["protocol_version"],
            RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION
        );
        assert_eq!(fixture["profile"], RUNTIME_V4_EXPERT_REST_ACTION_PROFILE);
        assert_eq!(
            fixture["schema_digest"],
            RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST
        );
        assert_eq!(
            fixture["provenance"]["artifact"],
            RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT
        );
        let messages = fixture["messages"].as_array().expect("producer messages");
        assert!(!messages.is_empty(), "{name}");
        let mut previous_after = None;
        for (index, message) in messages.iter().enumerate() {
            let request = &message["request"];
            let response = &message["response"];
            assert!(schema.is_valid(request), "{name} request {index}");
            assert!(schema.is_valid(response), "{name} response {index}");
            assert!(
                nested_observation_is_valid(response, &expert),
                "{name} response {index} observation"
            );
            assert_eq!(request["kind"], "action_request");
            assert_eq!(response["kind"], "action_response");
            assert_eq!(request["operation_id"], response["operation_id"]);
            assert_eq!(request["action"], response["action"]);
            let transition = response["transition"]
                .as_object()
                .expect("settled transition");
            assert_eq!(request["generation"], transition["before_generation"]);
            assert_eq!(response["generation"], transition["after_generation"]);
            assert!(
                transition["after_generation"].as_u64().unwrap()
                    > transition["before_generation"].as_u64().unwrap()
            );
            if let Some(previous_after) = previous_after {
                assert_eq!(
                    request["generation"], previous_after,
                    "{name} request {index} continuity"
                );
            }
            assert!(strict_semantics(response, &requests, &admissions).is_some_and(|valid| valid));
            if transition["kind"] == "rest_option_selection_completed" {
                let observation = response["observation"]
                    .as_object()
                    .expect("completed observation");
                assert_eq!(observation["state"]["state"], "rest");
                assert!(
                    observation["state"]["choices"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|choice| choice["kind"] != "selection")
                );
            }
            previous_after = Some(response["generation"].clone());
        }
    }
}

#[test]
fn checksum_inventory_covers_schema_case_manifest_goldens_mutations_and_producers() {
    let root = artifact_root();
    for line in CHECKSUMS.lines().filter(|line| !line.trim().is_empty()) {
        let (expected, relative) = line.split_once("  ").expect("checksum has two columns");
        assert_eq!(expected.len(), 64, "{relative}");
        let bytes =
            fs::read(root.join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"));
        let actual = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(actual, expected, "{relative}");
    }
    assert_eq!(
        CHECKSUMS
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        44
    );
    assert!(CHECKSUMS.contains("../../schemas/runtime-v4-expert-rest-action-v1.schema.json"));
    assert!(CHECKSUMS.contains("../../conformance/cases/runtime-v4-expert-rest-action-v1.json"));
    assert!(CHECKSUMS.contains("producer/mend-selection-lifecycle.json"));
    assert!(CHECKSUMS.contains("producer/smith-selection-lifecycle.json"));
}
