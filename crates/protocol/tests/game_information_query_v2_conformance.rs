// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use sts2_protocol::game_information_v2::{
    PROFILE, Rejection, SCHEMA, SCHEMA_DIGEST, ValidationContext, Validator,
};

mod game_information_query_v2_vectors;
use game_information_query_v2_vectors::{apply, canonical, recount};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(path)).expect("fixture file"))
        .expect("original fixture JSON")
}

fn golden(name: &str) -> Value {
    read(&format!("artifacts/{PROFILE}/golden/{name}.json"))
}

fn context(name: &str) -> Value {
    read(&format!(
        "conformance/fixtures/{PROFILE}/contexts/{name}.json"
    ))
}

fn typed_context(value: Value) -> ValidationContext {
    serde_json::from_value(value).expect("closed validation context")
}

fn digest_hex(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes.as_ref())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn source_artifact_digest_and_candidate_status() {
    assert_eq!(
        SCHEMA.as_bytes(),
        fs::read(root().join(format!("artifacts/{PROFILE}/schema.json"))).expect("artifact schema")
    );
    assert_eq!(digest_hex(SCHEMA.as_bytes()), SCHEMA_DIGEST);
    let manifest = read(&format!("artifacts/{PROFILE}/manifest.json"));
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(manifest["consumers"], json!([]));
    assert_eq!(manifest["status"], "candidate");
    assert_eq!(manifest["consumer_evidence"], "unverified");
    assert_eq!(manifest["live_status"], "unverified");
    assert_eq!(
        manifest["prospective_consumers"],
        json!(["sts2-gateway", "sts2-mcp-server"])
    );
    assert_eq!(manifest["prospective_producer"], "sts2-game-mod");
    assert_eq!(
        digest_hex(
            fs::read(root().join("schemas/game-information-query-v1.schema.json"))
                .expect("immutable v1 schema")
        ),
        "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9"
    );
}

#[test]
fn standalone_schema_is_closed_local_and_preserves_inherited_definitions() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema");
    let old = read("schemas/game-information-query-v1.schema.json");
    let changed = [
        "entity_kind",
        "field_name",
        "provenance",
        "field",
        "item",
        "query",
        "cursor_binding",
        "capabilities",
        "envelope",
    ];
    for (name, definition) in old["$defs"].as_object().expect("v1 definitions") {
        if !changed.contains(&name.as_str()) {
            assert_eq!(
                &schema["$defs"][name], definition,
                "unchanged inherited {name}"
            );
        }
    }
    fn walk(value: &Value) {
        if let Some(object) = value.as_object() {
            if let Some(reference) = object.get("$ref") {
                assert!(
                    reference
                        .as_str()
                        .expect("reference")
                        .starts_with("#/$defs/")
                );
            }
            if object.get("type") == Some(&json!("object")) {
                assert_eq!(object.get("additionalProperties"), Some(&json!(false)));
            }
            for member in object.values() {
                walk(member);
            }
        } else if let Some(array) = value.as_array() {
            for member in array {
                walk(member);
            }
        }
    }
    walk(&schema);
}

#[test]
fn every_original_golden_passes_schema_and_production_boundary() {
    let cases = read(&format!("conformance/cases/{PROFILE}.json"));
    let schema =
        jsonschema::validator_for(&serde_json::from_str::<Value>(SCHEMA).expect("schema JSON"))
            .expect("standalone schema compilation");
    let validator = Validator::new().expect("production embedded schema");
    let mut count = 0;
    for (name, context_name) in cases["golden_contexts"].as_object().expect("golden map") {
        let value = golden(name);
        assert!(
            schema.is_valid(&value),
            "schema rejected {name}: {:?}",
            schema.iter_errors(&value).collect::<Vec<_>>()
        );
        let context = typed_context(context(context_name.as_str().expect("context name")));
        let raw = fs::read(root().join(format!("artifacts/{PROFILE}/golden/{name}.json")))
            .expect("golden bytes");
        assert_eq!(
            canonical(&value) + "\n",
            String::from_utf8(raw.clone()).expect("UTF-8")
        );
        let decoded = validator
            .decode(&raw, &context)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(decoded, value);
        count += 1;
    }
    assert_eq!(count, 32);
}

#[test]
fn explicit_positive_negative_vectors_compare_schema_and_semantic_outcomes() {
    let case_index = read(&format!("conformance/cases/{PROFILE}.json"));
    let schema =
        jsonschema::validator_for(&serde_json::from_str::<Value>(SCHEMA).expect("schema JSON"))
            .expect("schema compilation");
    let validator = Validator::new().expect("production embedded schema");
    let mut vectors = Vec::new();
    for path in case_index["cases"].as_array().expect("cases") {
        let path = path
            .as_str()
            .expect("case path")
            .strip_prefix("../../")
            .expect("repository path");
        let case = read(path);
        let id = case["id"].as_str().expect("case identity");
        let mut value = golden(case["golden"].as_str().expect("golden name"));
        let mut source = context(case["context"].as_str().expect("context name"));
        apply(&mut value, &case["mutations"]);
        apply(&mut source, &case["context_mutations"]);
        if case["recount"] == true {
            recount(&mut value);
        }
        let mut raw = canonical(&value);
        if case["raw_replace"].is_object() {
            let from = case["raw_replace"]["from"].as_str().expect("raw source");
            let to = case["raw_replace"]["to"].as_str().expect("raw replacement");
            assert!(raw.contains(from), "{id}: replacement exists");
            raw = if case["raw_replace"]["all"] == true {
                raw.replace(from, to)
            } else {
                raw.replacen(from, to, 1)
            };
        }
        if let Some(spaces) = case["raw_prefix_spaces"].as_u64() {
            raw = " ".repeat(spaces as usize) + &raw;
        }
        vectors.push((case, source, raw));
    }
    let raw: Vec<_> = vectors.iter().map(|(_, _, raw)| raw.clone()).collect();
    let references = game_information_query_v2_vectors::oracle::exact_reference(&root(), &raw);
    for ((case, source, raw), reference) in vectors.into_iter().zip(references) {
        let id = case["id"].as_str().expect("case identity");
        let exact: Option<Value> = reference
            .canonical
            .map(|raw| serde_json::from_str(&raw).expect("exact normalized reference"));
        assert_eq!(reference.valid, exact.is_some(), "{id} exact oracle");
        assert_eq!(
            exact.as_ref().is_some_and(|value| schema.is_valid(value)),
            case["schema_valid"].as_bool().expect("schema expectation"),
            "{id} actual raw schema"
        );
        let expected: Option<Rejection> =
            serde_json::from_value(case["expected"].clone()).expect("fixed rejection code");
        let outcome = validator.decode(raw.as_bytes(), &typed_context(source));
        assert_eq!(outcome.as_ref().err().copied(), expected, "{id}");
        if let Ok(decoded) = outcome {
            assert_eq!(Some(decoded), exact, "{id} exact numeric value");
        }
    }
}

#[test]
fn exact_checksum_inventory_covers_all_original_cases_contexts_and_goldens() {
    let base = root().join(format!("artifacts/{PROFILE}"));
    let inventory = fs::read_to_string(base.join("SHA256SUMS")).expect("inventory");
    let case_index = read(&format!("conformance/cases/{PROFILE}.json"));
    for path in case_index["cases"].as_array().expect("cases") {
        assert!(inventory.contains(path.as_str().expect("path")));
    }
    assert!(inventory.contains("contexts/default.json"));
    assert!(inventory.contains("contexts/unavailable.json"));
    for line in inventory.lines() {
        let (digest, path) = line.split_once("  ").expect("checksum line");
        let bytes = fs::read(base.join(path)).unwrap_or_else(|_| panic!("missing {path}"));
        assert_eq!(digest_hex(bytes), digest, "{path}");
    }
}

#[test]
fn v2_is_rejected_by_v1_schema_and_v1_is_rejected_before_v2_semantics() {
    let old = jsonschema::validator_for(&read("schemas/game-information-query-v1.schema.json"))
        .expect("v1 schema");
    assert!(!old.is_valid(&golden("capabilities-response")));
    assert!(!old.is_valid(&golden("live-detail-heal-response")));
    let old_response = read("artifacts/game-information-query-v1/golden/live-detail-response.json");
    assert_eq!(
        Validator::new()
            .expect("validator")
            .validate(&old_response, &typed_context(context("default"))),
        Err(Rejection::UnsupportedVersion)
    );
}

#[test]
fn parser_rejects_trailing_data_control_text_duplicate_nested_members_and_depth() {
    let validator = Validator::new().expect("validator");
    let context = typed_context(context("default"));
    let raw = canonical(&golden("live-detail-heal-response"));
    assert_eq!(
        validator
            .decode((raw.clone() + " null").as_bytes(), &context)
            .err(),
        Some(Rejection::Malformed)
    );
    let duplicate = raw.replacen(
        "\"kind\":\"rest_option\"",
        "\"kind\":\"rest_option\",\"kind\":\"rest_option\"",
        1,
    );
    assert_ne!(duplicate, raw);
    assert_eq!(
        validator.decode(duplicate.as_bytes(), &context).err(),
        Some(Rejection::Malformed)
    );
    let nested = "[".repeat(200) + &"]".repeat(200);
    assert_eq!(
        validator.decode(nested.as_bytes(), &context).err(),
        Some(Rejection::Malformed)
    );
}
