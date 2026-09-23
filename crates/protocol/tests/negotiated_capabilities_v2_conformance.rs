// SPDX-License-Identifier: MIT

use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::draft202012;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PROFILE: &str = "sts2-gateway-negotiated-capabilities-v2";
const ARTIFACT: &str = "sts2-gateway/negotiated-capabilities-v2";
const SCHEMA_DIGEST: &str = "c6453f1a760675c7492261eb7b50be76cf87d8c7d4762a070d26693b15225b7f";
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/negotiated-capabilities-v2.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/negotiated-capabilities-v2/schema.json");
const MANIFEST: &str = include_str!("../../../artifacts/negotiated-capabilities-v2/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/negotiated-capabilities-v2/SHA256SUMS");
const CASE: &str = include_str!("../../../conformance/cases/negotiated-capabilities-v2.json");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn v2_is_an_additive_closed_extension_with_integral_artifact() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    assert_eq!(digest(SOURCE_SCHEMA.as_bytes()), SCHEMA_DIGEST);
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON");
    let validator = draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    assert_eq!(manifest["artifact"], ARTIFACT);
    assert_eq!(manifest["protocol_version"], PROFILE);
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(
        manifest["supersedes"],
        "sts2-gateway/negotiated-capabilities-v1"
    );
    assert_eq!(case["profile"], PROFILE);
    assert_eq!(
        case["extension"]["operation"],
        "game_information.live_observation_bootstrap"
    );
    assert_eq!(
        case["extension"]["revision"],
        "game-information-live-observation-bootstrap-v1"
    );

    let mut document = serde_json::json!({
        "schema_version": PROFILE,
        "gateway_revision": PROFILE,
        "correlation_id": "negotiated-capabilities-v2-test",
        "instance_id": "instance-1",
        "caller_id": "harness",
        "session_id": "session-1",
        "mcp_session_id": "mcp-session-1",
        "lease_id": "lease-1",
        "lease_epoch": 1,
        "caller_scopes": ["read"],
        "producer": {
            "profile": "game-information-query-v1",
            "schema_digest": "376845b0c86b4afcd2c79ffba753eb7e7e416f5410da26b4dae970cfee2221d9",
            "content_manifest_id": "content-1",
            "run_id": "run-42"
        },
        "lookup_binding_witness": {
            "profile": "game-information-lookup-binding-v1",
            "schema_digest": "f10f9af01d6be1de104069ba842e7971971e88f27553e782e81174ee7aa1cd58",
            "binding_id": "58fea90991138ea6fb635df1f5eadd08973ec63eba456d135578677ffee61cfc",
            "content_manifest_id": "content-1",
            "authority_epoch": 7
        },
        "runtime_v3_baseline_witness": null,
        "offers": [case["extension"].clone()]
    });
    assert!(validator.is_valid(&document));
    let mut missing_witness = document.clone();
    missing_witness["lookup_binding_witness"] = Value::Null;
    assert!(
        !validator.is_valid(&missing_witness),
        "live bootstrap requires a current lookup-binding witness"
    );
    let mut legacy = document.clone();
    legacy["offers"][0]["operation"] = json!("game_information.capabilities");
    legacy["offers"][0]["revision"] = json!("game-information-query-v1");
    assert!(
        validator.is_valid(&legacy),
        "v1 operation remains valid in v2"
    );
    document["offers"][0]["operation"] = json!("future.operation");
    assert!(!validator.is_valid(&document));
    let mut malformed = legacy;
    malformed["offers"][0]["wire_limits"]["max_request_bytes"] = json!(-1);
    assert!(!validator.is_valid(&malformed));
    malformed["offers"][0]["wire_limits"]["max_request_bytes"] = json!(16_384);
    malformed["offers"][0]["unexpected"] = json!(true);
    assert!(!validator.is_valid(&malformed));
}

#[test]
fn v1_operation_inventory_is_unchanged_and_live_requires_its_binding_witness() {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON");
    let operations = schema["properties"]["offers"]["items"]["properties"]["operation"]["enum"]
        .as_array()
        .expect("operation enum");
    let v1_operations = [
        "game_information.capabilities",
        "game_information.list",
        "game_information.search",
        "game_information.get",
        "game_information.detail",
        "game_information.availability",
        "game_information.lookup_binding.discovery",
        "game_information.lookup_binding.observe",
        "runtime_v3.state",
        "runtime_v3.legal_actions",
        "runtime_v3.dispatch_action",
        "runtime_v3.wait",
        "runtime_v3.reobserve",
        "runtime_v3.recover",
    ];
    assert_eq!(operations.len(), v1_operations.len() + 1);
    for operation in v1_operations {
        assert!(operations.iter().any(|value| value == operation));
    }
    assert!(
        operations
            .iter()
            .any(|value| value == "game_information.live_observation_bootstrap")
    );
    let revisions = schema["properties"]["offers"]["items"]["properties"]["revision"]["enum"]
        .as_array()
        .expect("revision enum");
    assert_eq!(revisions.len(), 4);
    for revision in [
        "game-information-query-v1",
        "game-information-lookup-binding-v1",
        "runtime-v3-gameplay",
        "game-information-live-observation-bootstrap-v1",
    ] {
        assert!(revisions.iter().any(|value| value == revision));
    }
}

#[test]
fn every_v2_artifact_member_matches_its_checksum() {
    let directory = root().join("artifacts").join("negotiated-capabilities-v2");
    let mut checked = 0;
    for line in CHECKSUMS.lines() {
        let (expected, name) = line.split_once("  ").expect("checksum line");
        let bytes = fs::read(directory.join(name.strip_prefix("./").unwrap_or(name)))
            .expect("artifact member");
        assert_eq!(digest(&bytes), expected, "checksum for {name}");
        checked += 1;
    }
    assert_eq!(checked, 4);
}
