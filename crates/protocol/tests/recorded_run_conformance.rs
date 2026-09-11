// SPDX-License-Identifier: MIT

use serde_json::Value;

const SCHEMA: &str = include_str!("../../../schemas/recorded-run-bundle-v1-candidate3.schema.json");
const ARTIFACT: &str =
    include_str!("../../../artifacts/recorded-run-bundle-v1-candidate3/schema.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/recorded-run-bundle-v1-candidate3/golden/manifest.json");
const OMISSIONS: &str =
    include_str!("../../../artifacts/recorded-run-bundle-v1-candidate3/golden/omissions.json");
const EVENTS: &str =
    include_str!("../../../artifacts/recorded-run-bundle-v1-candidate3/golden/events.ndjson");
const ACCOUNTING: &str =
    include_str!("../../../artifacts/recorded-run-bundle-v1-candidate3/golden/accounting.ndjson");

#[test]
fn recorded_run_schema_closes_synthetic_documents_independently() {
    assert_eq!(SCHEMA, ARTIFACT);
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("Draft 2020-12 schema compiles");
    for text in [MANIFEST, OMISSIONS]
        .into_iter()
        .chain(EVENTS.lines())
        .chain(ACCOUNTING.lines())
    {
        let value: Value = serde_json::from_str(text).expect("synthetic JSON");
        assert!(validator.is_valid(&value), "valid synthetic document");
        let mut extra = value.clone();
        extra["raw_prompt"] = Value::String("SENTINEL_PRIVATE".into());
        assert!(
            !validator.is_valid(&extra),
            "closed fields reject private text"
        );
    }
}

#[test]
fn recorded_run_schema_requires_identity_evidence_and_precision() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    let value: Value = serde_json::from_str(EVENTS.lines().next().expect("event")).expect("event");
    let mut missing = value.clone();
    missing["evidence"]
        .as_object_mut()
        .expect("evidence")
        .remove("gameplay");
    assert!(!validator.is_valid(&missing));
    let mut numeric = value;
    numeric["time"]["unix_ns"] = serde_json::json!(1_789_090_000_123_456_789_u64);
    assert!(!validator.is_valid(&numeric));
    let mut manifest: Value = serde_json::from_str(MANIFEST).expect("manifest");
    manifest["format_version"] = serde_json::json!("2.0.0");
    assert!(!validator.is_valid(&manifest));
}

#[test]
fn candidate3_schema_closes_reviewed_digests_and_disposition_pairs() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    let mut event: Value =
        serde_json::from_str(EVENTS.lines().last().expect("diagnostic")).expect("event");
    event["payload"]["value"] = serde_json::json!({"code":"unsupported_source_event"});
    assert!(!validator.is_valid(&event));
    event["payload"]["value"]["value_digest"] = serde_json::json!("a".repeat(64));
    assert!(validator.is_valid(&event));
    event["payload"] = serde_json::json!({
        "profile": "Example.optional+v2:Test_1-0",
        "kind": "opaque",
        "value": {"content_digest":"b".repeat(64),"bytes":1,"media_type":"application/json"}
    });
    assert!(validator.is_valid(&event));
    let mut omissions: Value = serde_json::from_str(OMISSIONS).expect("report");
    omissions["streams"][1]["dispositions"][0]["disposition"] = serde_json::json!("rejected");
    assert!(!validator.is_valid(&omissions));
}
