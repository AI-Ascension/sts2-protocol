// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::draft202012;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PROFILE: &str = "game-information-live-observation-bootstrap-v1";
const ARTIFACT: &str = "sts2-protocol/game-information-live-observation-bootstrap-v1";
const SCHEMA_DIGEST: &str = "6041a282ffda8757af4e3eb6ab551e082f136fe53138ab8ac17db9fab52765c2";
const QUERY_V1_DIGEST: &str = "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9";

const SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/game-information-live-observation-bootstrap-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/game-information-live-observation-bootstrap-v1/schema.json");
const QUERY_V1_SCHEMA: &str =
    include_str!("../../../schemas/game-information-query-v1.schema.json");
const CASE: &str =
    include_str!("../../../conformance/cases/game-information-live-observation-bootstrap-v1.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/game-information-live-observation-bootstrap-v1/manifest.json");
const CHECKSUMS: &str =
    include_str!("../../../artifacts/game-information-live-observation-bootstrap-v1/SHA256SUMS");

const GOLDENS: &[(&str, &str)] = &[
    (
        "bootstrap-request",
        include_str!(
            "../../../artifacts/game-information-live-observation-bootstrap-v1/golden/bootstrap-request.json"
        ),
    ),
    (
        "bootstrap-response",
        include_str!(
            "../../../artifacts/game-information-live-observation-bootstrap-v1/golden/bootstrap-response.json"
        ),
    ),
    (
        "error-native-unavailable",
        include_str!(
            "../../../artifacts/game-information-live-observation-bootstrap-v1/golden/error-native-unavailable.json"
        ),
    ),
];

const INVALID: &[(&str, &str)] = &[
    (
        "foreign-instance",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/foreign-instance.json"
        ),
    ),
    (
        "duplicate-visible-entity",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/duplicate-visible-entity.json"
        ),
    ),
    (
        "missing-native-ref",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/missing-native-ref.json"
        ),
    ),
    (
        "selector-instance-mismatch",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/selector-instance-mismatch.json"
        ),
    ),
    (
        "stale-generation",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/stale-generation.json"
        ),
    ),
    (
        "cross-run",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/cross-run.json"
        ),
    ),
    (
        "foreign-manifest",
        include_str!(
            "../../../conformance/fixtures/game-information-live-observation-bootstrap-v1/invalid/foreign-manifest.json"
        ),
    ),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn schema_validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON");
    draft202012::options()
        .build(&schema)
        .expect("schema compiles")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn value(text: &str) -> Value {
    serde_json::from_str(text).expect("JSON")
}

fn ref_key(reference: &Value) -> Value {
    json!([
        reference["instance_id"],
        reference["run_id"],
        reference["epoch"],
        reference["entity_kind"],
        reference["entity_id"]
    ])
}

fn limits_within(response: &Value, request: &Value) -> bool {
    [
        "max_visible_entities",
        "max_item_bytes",
        "max_message_bytes",
    ]
    .into_iter()
    .all(|field| {
        response["limits"][field].as_u64().unwrap() <= request["limits"][field].as_u64().unwrap()
    })
}

fn semantic_error(document: &Value) -> Option<&'static str> {
    if document["kind"] == "error_response" {
        return None;
    }
    if document["kind"] != "bootstrap_response" {
        return None;
    }
    let parent = &document["parent_observation"];
    if parent["instance_ref"] != parent["snapshot_ref"]["instance_ref"]
        || parent["state_generation"] != parent["snapshot_ref"]["state_generation"]
    {
        return Some("invalid_binding");
    }
    let visible = document["visible_entities"].as_array()?;
    let limit = document["limits"]["max_visible_entities"].as_u64()?;
    if visible.len() as u64 > limit {
        return Some("invalid_bounds");
    }
    let parent_ref = &parent["instance_ref"];
    let scope = &document["scope"];
    if scope["instance_id"] != parent_ref["instance_id"]
        || scope["run_id"] != parent_ref["run_id"]
        || scope["content_manifest_id"]
            != document["selector"]["definition_ref"]["content_manifest_id"]
    {
        return Some("invalid_binding");
    }
    if document["owner_provenance"]
        != json!({
            "authority_epoch_owner": "sts2-harness",
            "content_manifest_owner": "sts2-game-mod",
            "instance_fence_owner": "sts2-gateway",
            "instance_ref_epoch_owner": "sts2-game-mod",
            "native_snapshot_owner": "sts2-game-mod",
            "transport_lease_epoch_role": "fence_only"
        })
    {
        return Some("invalid_binding");
    }
    let definition = &document["selector"]["definition_ref"];
    let selector_ref = &document["selector"]["instance_ref"];
    let mut keys = Vec::with_capacity(visible.len());
    for entity in visible {
        let reference = &entity["instance_ref"];
        let entity_snapshot = &entity["snapshot_ref"];
        for field in ["instance_id", "run_id", "epoch"] {
            if reference[field] != parent_ref[field] {
                return Some("invalid_binding");
            }
        }
        if entity_snapshot["instance_ref"] != *reference
            || entity_snapshot["snapshot_id"] != parent["snapshot_ref"]["snapshot_id"]
        {
            return Some("invalid_binding");
        }
        if entity_snapshot["state_generation"] != parent["state_generation"] {
            return Some("stale_snapshot");
        }
        if entity["definition_ref"].is_object() && entity["definition_ref"] != *definition {
            return Some("invalid_binding");
        }
        let key = ref_key(reference);
        if keys.iter().any(|existing| existing == &key) {
            return Some("invalid_binding");
        }
        keys.push(key);
    }
    if !visible
        .iter()
        .any(|entity| entity["instance_ref"] == *parent_ref)
    {
        return Some("invalid_binding");
    }
    if !selector_ref.is_null()
        && (selector_ref != parent_ref
            || visible
                .iter()
                .filter(|entity| entity["instance_ref"] == *selector_ref)
                .count()
                != 1)
    {
        return Some("invalid_binding");
    }
    let max_item_bytes = document["limits"]["max_item_bytes"].as_u64()?;
    for entity in visible {
        if serde_json::to_vec(entity).ok()?.len() as u64 > max_item_bytes {
            return Some("invalid_bounds");
        }
    }
    if serde_json::to_vec(document).ok()?.len() as u64
        > document["limits"]["max_message_bytes"].as_u64()?
    {
        return Some("invalid_bounds");
    }
    None
}

#[test]
fn source_artifact_and_goldens_are_schema_valid_and_canonical() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    assert_eq!(digest(SOURCE_SCHEMA.as_bytes()), SCHEMA_DIGEST);
    let validator = schema_validator();
    for (name, text) in GOLDENS {
        let document = value(text);
        assert!(validator.is_valid(&document), "{name} violates schema");
        assert_eq!(
            serde_json::to_string(&document).expect("canonical JSON"),
            payload(text),
            "{name} is not compact sorted JSON"
        );
        assert_eq!(document["protocol_version"], PROFILE);
        assert_eq!(document["schema_digest"], SCHEMA_DIGEST);
        assert_eq!(document["provenance"]["artifact"], ARTIFACT);
        assert_eq!(semantic_error(&document), None, "{name} semantic error");
    }
}

#[test]
fn the_artifact_manifest_and_case_name_the_contract_and_consumers() {
    let manifest = value(MANIFEST);
    let case = value(CASE);
    assert_eq!(manifest["artifact"], ARTIFACT);
    assert_eq!(manifest["protocol_version"], PROFILE);
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(manifest["status"], "candidate");
    assert_eq!(manifest["prospective_producer"], "sts2-game-mod");
    assert_eq!(
        manifest["prospective_consumers"],
        json!(["sts2-gateway", "sts2-harness", "sts2-mcp-server"])
    );
    assert_eq!(
        case["case_id"],
        "CT-GAME-INFORMATION-LIVE-OBSERVATION-BOOTSTRAP-V1-001"
    );
    assert_eq!(case["profile"], PROFILE);
    assert_eq!(case["consumers"], manifest["prospective_consumers"]);
    assert_eq!(case["valid_vectors"].as_array().map(Vec::len), Some(3));
    assert_eq!(case["invalid_vectors"].as_array().map(Vec::len), Some(7));
}

#[test]
fn checksums_cover_every_artifact_member() {
    let directory = root().join("artifacts").join(PROFILE);
    let mut checked = 0;
    for line in CHECKSUMS.lines() {
        let Some((expected, name)) = line.split_once("  ") else {
            continue;
        };
        let bytes = fs::read(directory.join(name))
            .unwrap_or_else(|error| panic!("checksums cover {name}: {error}"));
        assert_eq!(digest(&bytes), expected, "checksum for {name}");
        checked += 1;
    }
    assert!(checked >= 7, "checksum inventory lists {checked} members");
}

#[test]
fn duplicate_definitions_keep_distinct_instances_and_bootstrap_the_query_v1_shape() {
    let bootstrap = value(GOLDENS[1].1);
    let query = value(
        &fs::read_to_string(
            root()
                .join("artifacts")
                .join(PROFILE)
                .join("golden/live-query-v1-request.json"),
        )
        .expect("query-v1 request"),
    );
    let visible = bootstrap["visible_entities"]
        .as_array()
        .expect("visible entities");
    assert_eq!(visible.len(), 2);
    assert_eq!(
        visible[0]["definition_ref"], visible[1]["definition_ref"],
        "same definition has multiple occurrences"
    );
    assert_ne!(
        visible[0]["instance_ref"]["entity_id"], visible[1]["instance_ref"]["entity_id"],
        "occurrences retain distinct native IDs"
    );
    let selected = &visible[1]["instance_ref"];
    let query_validator = draft202012::options()
        .build(&serde_json::from_str::<Value>(QUERY_V1_SCHEMA).expect("query-v1 schema"))
        .expect("query-v1 schema compiles");
    assert!(
        query_validator.is_valid(&query),
        "selected query remains valid query-v1"
    );
    assert_eq!(
        query["schema_digest"], QUERY_V1_DIGEST,
        "bootstrap does not rewrite query-v1"
    );
    assert_eq!(query["query"]["binding"]["instance_ref"], *selected);
    assert_eq!(
        query["query"]["binding"]["snapshot_ref"]["snapshot_id"],
        bootstrap["parent_observation"]["snapshot_ref"]["snapshot_id"]
    );
    assert_eq!(
        query["query"]["binding"]["snapshot_ref"]["state_generation"],
        bootstrap["parent_observation"]["state_generation"]
    );
    assert_eq!(
        query["query"]["parent_observation"]["instance_ref"],
        *selected
    );
}

#[test]
fn response_echoes_request_identity_and_never_expands_limits() {
    let request = value(GOLDENS[0].1);
    let response = value(GOLDENS[1].1);
    for field in ["correlation_id", "scope", "selector", "limits"] {
        assert_eq!(response[field], request[field], "response echoes {field}");
    }
    let mut expanded = response.clone();
    expanded["limits"]["max_visible_entities"] =
        json!(request["limits"]["max_visible_entities"].as_u64().unwrap() + 1);
    assert!(
        !limits_within(&expanded, &request),
        "expanded response limits are rejected"
    );
}

#[test]
fn semantic_negative_vectors_fail_closed() {
    let validator = schema_validator();
    let expected = [
        ("foreign-instance", "invalid_binding", true),
        ("duplicate-visible-entity", "invalid_binding", true),
        ("missing-native-ref", "invalid_request", false),
        ("selector-instance-mismatch", "invalid_binding", true),
        ("stale-generation", "stale_snapshot", true),
        ("cross-run", "invalid_binding", true),
        ("foreign-manifest", "invalid_binding", true),
    ];
    for ((name, text), (expected_name, expected_error, schema_valid)) in
        INVALID.iter().zip(expected)
    {
        assert_eq!(*name, expected_name);
        let document = value(text);
        assert_eq!(
            validator.is_valid(&document),
            schema_valid,
            "{name} schema validity"
        );
        if schema_valid {
            assert_eq!(semantic_error(&document), Some(expected_error), "{name}");
        }
    }
}
