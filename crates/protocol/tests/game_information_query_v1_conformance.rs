// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PROFILE: &str = "game-information-query-v1";
const SCHEMA_DIGEST: &str = "e5ba81b0520687cf59db6a94aea3b38606e86300f6eb2b0e858f55704e62f76c";
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/game-information-query-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/game-information-query-v1/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/game-information-query-v1.json");
const MANIFEST: &str = include_str!("../../../artifacts/game-information-query-v1/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/game-information-query-v1/SHA256SUMS");

const GOLDENS: &[(&str, &str)] = &[
    (
        "capabilities-response",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/capabilities-response.json"
        ),
    ),
    (
        "error-stale-cursor",
        include_str!("../../../artifacts/game-information-query-v1/golden/error-stale-cursor.json"),
    ),
    (
        "live-detail-request",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/live-detail-request.json"
        ),
    ),
    (
        "live-detail-response",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/live-detail-response.json"
        ),
    ),
    (
        "static-page-1-request",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/static-page-1-request.json"
        ),
    ),
    (
        "static-page-1-response",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/static-page-1-response.json"
        ),
    ),
    (
        "static-page-2-request",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/static-page-2-request.json"
        ),
    ),
    (
        "static-page-2-response",
        include_str!(
            "../../../artifacts/game-information-query-v1/golden/static-page-2-response.json"
        ),
    ),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn value(name: &str) -> Value {
    GOLDENS
        .iter()
        .find_map(|(golden_name, text)| {
            (*golden_name == name).then(|| serde_json::from_str(text).unwrap())
        })
        .expect("named game-information golden exists")
}

fn schema_validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn artifact_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts/game-information-query-v1")
}

fn value_at_mut<'a>(value: &'a mut Value, segment: &str) -> Option<&'a mut Value> {
    match value {
        Value::Object(map) => map.get_mut(segment),
        Value::Array(items) => segment
            .parse::<usize>()
            .ok()
            .and_then(|index| items.get_mut(index)),
        _ => None,
    }
}

fn apply_mutations(mut value: Value, mutations: &[Value]) -> Value {
    for mutation in mutations {
        let path = mutation["path"].as_str().expect("mutation path");
        let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut cursor = &mut value;
        for segment in &segments[..segments.len().saturating_sub(1)] {
            cursor = value_at_mut(cursor, segment)
                .unwrap_or_else(|| panic!("mutation path exists: {path} at {segment}"));
        }
        let leaf = segments.last().expect("mutation has leaf");
        *value_at_mut(cursor, leaf)
            .unwrap_or_else(|| panic!("mutation path exists: {path} at {leaf}")) =
            mutation["value"].clone();
    }
    value
}

#[test]
fn source_artifact_and_every_golden_are_schema_valid_and_canonical() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    let validator = schema_validator();
    for (name, text) in GOLDENS {
        let value: Value = serde_json::from_str(text).expect("golden JSON");
        if !validator.is_valid(&value) {
            eprintln!("{name}: {:?}", validator.validate(&value));
        }
        assert!(validator.is_valid(&value), "{name} violates the schema");
        assert_eq!(
            serde_json::to_string(&value).expect("canonical JSON"),
            payload(text),
            "{name} is not compact sorted JSON"
        );
        assert_eq!(value["protocol_version"], PROFILE);
        assert_eq!(value["schema_digest"], SCHEMA_DIGEST);
        assert_eq!(
            value["provenance"]["artifact"],
            "sts2-protocol/game-information-query-v1"
        );
    }
}

#[test]
fn static_two_page_and_live_detail_vectors_preserve_their_fences() {
    let first_request = value("static-page-1-request");
    let first_response = value("static-page-1-response");
    let second_request = value("static-page-2-request");
    let second_response = value("static-page-2-response");
    assert_eq!(
        first_request["query"]["binding"],
        first_response["query"]["binding"]
    );
    assert_eq!(
        first_response["result"]["page"]["next_cursor"],
        second_request["query"]["cursor"]
    );
    assert_eq!(
        first_response["result"]["page"]["cursor_binding"],
        second_response["result"]["page"]["cursor_binding"]
    );
    assert!(
        !first_response["result"]["page"]["final_page"]
            .as_bool()
            .unwrap()
    );
    assert!(
        second_response["result"]["page"]["final_page"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(second_response["result"]["page"]["total_count"], json!(3));
    assert!(
        second_response["result"]["page"]["total_count_known"]
            .as_bool()
            .unwrap()
    );

    let request = value("live-detail-request");
    let response = value("live-detail-response");
    let binding = &request["query"]["binding"];
    let snapshot = &binding["snapshot_ref"];
    assert_eq!(binding["mode"], "live");
    assert_eq!(
        snapshot["state_generation"],
        response["result"]["result_generation"]
    );
    assert_eq!(
        request["query"]["parent_observation"],
        response["result"]["parent_observation"]
    );
    assert_eq!(
        response["result"]["page"]["items"][0]["instance_ref"],
        request["query"]["target"]["instance_ref"]
    );
}

#[test]
fn fields_keep_zero_empty_missing_unavailable_and_provenance_distinct() {
    let response = value("live-detail-response");
    let fields = response["result"]["page"]["items"][0]["fields"]
        .as_array()
        .expect("fields");
    let unavailable = fields
        .iter()
        .find(|field| field["name"] == "description")
        .expect("unavailable field");
    assert_eq!(unavailable["availability"], "not_observable");
    assert!(unavailable["value"].is_null());
    assert_eq!(unavailable["source"]["kind"], "game_mod");
    assert_eq!(unavailable["source"]["ref"], "instance-1");

    let static_page = value("static-page-2-response");
    let tags = &static_page["result"]["page"]["items"][0]["fields"][1];
    assert_eq!(tags["name"], "tags");
    assert_eq!(tags["availability"], "available");
    assert_eq!(tags["value"], json!([]));
    assert_eq!(tags["source"]["kind"], "content_manifest");
}

#[test]
fn capabilities_and_error_vectors_are_explicit() {
    let capabilities = value("capabilities-response");
    assert_eq!(capabilities["kind"], "capabilities_response");
    assert_eq!(
        capabilities["capabilities"]["snapshot_policy"]["expiry_behavior"],
        "reject_stale_snapshot"
    );
    assert_eq!(
        capabilities["capabilities"]["snapshot_policy"]["invalidated_by"]
            .as_array()
            .map(Vec::len),
        Some(6)
    );
    let error = value("error-stale-cursor");
    assert_eq!(error["kind"], "error_response");
    assert_eq!(error["error"]["code"], "stale_cursor");
    assert!(!error["error"]["retryable"].as_bool().unwrap());
}

#[test]
fn invalid_vectors_match_schema_expectations_and_typed_errors() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    let validator = schema_validator();
    let invalid = case["invalid_vectors"].as_array().expect("invalid vectors");
    assert!(invalid.len() >= 14);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for descriptor in invalid {
        let path = root.join(descriptor["fixture"].as_str().expect("fixture path"));
        let fixture: Value = serde_json::from_str(&fs::read_to_string(path).expect("fixture file"))
            .expect("fixture");
        assert_eq!(
            fixture["expected_error"], descriptor["expected_error"],
            "vector ID {}",
            descriptor["id"]
        );
        if let Some(raw) = fixture["raw"].as_str() {
            assert!(raw.matches("\"protocol_version\"").count() > 1);
            continue;
        }
        let base = fixture["base_fixture"].as_str().expect("base fixture");
        let base_path = root.join(base);
        let base_value: Value =
            serde_json::from_str(&fs::read_to_string(base_path).expect("base fixture"))
                .expect("base");
        let mutated = apply_mutations(
            base_value,
            fixture["mutations"].as_array().expect("mutations"),
        );
        let schema_valid = validator.is_valid(&mutated);
        assert_eq!(
            schema_valid,
            fixture["schema_valid"].as_bool().expect("schema_valid"),
            "schema result for {}",
            fixture["id"]
        );
        if fixture["expected_error"] == "mixed_generation" {
            assert_ne!(
                mutated["result"]["result_generation"],
                mutated["query"]["binding"]["snapshot_ref"]["state_generation"]
            );
        }
        if fixture["expected_error"] == "malformed"
            && fixture["id"] == "GIQ-INVALID-MISSING-VS-EMPTY"
        {
            assert_eq!(
                mutated["result"]["page"]["items"][0]["fields"][0]["value"],
                "Defend"
            );
            assert_eq!(
                mutated["result"]["page"]["items"][0]["fields"][0]["availability"],
                "unavailable"
            );
        }
    }
}

#[test]
fn field_and_wire_vectors_are_present_and_inventory_is_exact() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    for vector in case["valid_vectors"].as_array().expect("valid vectors") {
        if let Some(path) = vector["fixture"].as_str() {
            let fixture: Value = serde_json::from_str(
                &fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(path),
                )
                .expect("field fixture"),
            )
            .expect("field fixture JSON");
            assert_eq!(fixture["id"], vector["id"]);
        }
    }
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(manifest["protocol_version"], PROFILE);
    assert_eq!(manifest["checksums"], "SHA256SUMS");
    assert_eq!(
        manifest["goldens"].as_array().map(Vec::len),
        Some(GOLDENS.len())
    );
    let root = artifact_root();
    let mut checked = 0;
    for line in CHECKSUMS.lines().filter(|line| !line.is_empty()) {
        let (expected, path) = line.split_once("  ").expect("checksum columns");
        let bytes = fs::read(root.join(path)).expect("inventory path");
        assert_eq!(digest(&bytes), expected, "inventory digest for {path}");
        checked += 1;
    }
    assert!(checked >= 12);
}
