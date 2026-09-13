// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PROFILE: &str = "game-information-query-v1";
const SCHEMA_DIGEST: &str = "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9";
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

const ENTITY_KINDS: &[&str] = &[
    "card",
    "character",
    "enemy",
    "event",
    "map_node",
    "potion",
    "power",
    "relic",
    "room",
    "status",
];
const QUERY_KINDS: &[&str] = &["list", "search", "get", "detail", "availability"];
const PROJECTIONS: &[&str] = &["summary", "standard", "full"];
const DETAIL_LEVELS: &[&str] = &["summary", "standard", "full"];
const FIELD_NAMES: &[&str] = &[
    "amount",
    "cost",
    "description",
    "display_name",
    "flags",
    "owner",
    "position",
    "rarity",
    "source_id",
    "tags",
];
const FIELD_KINDS: &[&str] = &[
    "boolean",
    "definition_ref",
    "integer",
    "instance_ref",
    "text",
    "text_list",
];
const AVAILABILITIES: &[&str] = &[
    "available",
    "unavailable",
    "not_observable",
    "redacted",
    "unsupported",
    "missing",
];

const AMBIGUOUS_ID: &str = "ambiguous_id";
const INVALID_BOUNDS: &str = "invalid_bounds";
const MALFORMED: &str = "malformed";
const MIXED_GENERATION: &str = "mixed_generation";
const MISSING_CAPABILITY: &str = "missing_capability";
const READ_ONLY_VIOLATION: &str = "read_only_violation";
const RESULT_LIMIT_EXCEEDED: &str = "result_limit_exceeded";
const STALE_CURSOR: &str = "stale_cursor";
const STALE_SNAPSHOT: &str = "stale_snapshot";
const UNKNOWN_KIND: &str = "unknown_kind";
const UNSUPPORTED_FIELD: &str = "unsupported_field";
const UNSUPPORTED_PROJECTION: &str = "unsupported_projection";
const UNSUPPORTED_VERSION: &str = "unsupported_version";

#[derive(Clone)]
struct SemanticContext {
    capabilities: Value,
    cursor_bindings: Vec<(String, Value)>,
}

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

mod game_information_query_v1_extra;
mod game_information_query_v1_semantics;
use game_information_query_v1_semantics::{
    binding_value, canonical_bytes, semantic_rejection, text_bytes,
};

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
        let semantic = semantic_rejection(
            &value,
            value["query"]["binding"]
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("static"),
        );
        if value["kind"] == "error_response" {
            assert_eq!(semantic, value["error"]["code"].as_str());
        } else {
            assert_eq!(semantic, None, "{name} semantic rejection");
        }
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
        binding_value(&first_request["query"])
    );
    assert!(second_response["result"]["page"]["cursor_binding"].is_null());
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
            assert_eq!(fixture["expected_error"], MALFORMED);
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
        assert_eq!(
            semantic_rejection(
                &mutated,
                fixture["context"]
                    .as_str()
                    .or_else(|| descriptor["context"].as_str())
                    .unwrap_or("static"),
            ),
            fixture["expected_error"].as_str(),
            "semantic result for {}",
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
