// SPDX-License-Identifier: MIT

#![allow(
    clippy::expect_used,
    reason = "fixture tests fail fast on invalid test data"
)]

use serde_json::{Value, json};
use sts2_protocol::{RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST, decode_json};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-v4-expert/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v4-expert.json");
const GOLDEN: &str = include_str!("../../../artifacts/runtime-v4-expert/golden/observation.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v4-expert/SHA256SUMS");

#[test]
fn runtime_v4_expert_schema_and_artifact_are_byte_identical() {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let schema: Value = decode_json(SOURCE_SCHEMA).expect("schema is JSON");
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    assert_eq!(schema["$id"], "sts2-runtime-v4-expert");
    assert_eq!(case["contract"], "sts2.protocol/runtime-v4-expert");
    assert_eq!(case["setup"]["live_runtime"], false);
    assert_eq!(case["setup"]["proprietary_data"], false);
}

#[test]
fn runtime_v4_expert_golden_accepts_enriched_visible_state() {
    let schema: Value = decode_json(SOURCE_SCHEMA).expect("schema is JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12");
    let golden: Value = decode_json(GOLDEN).expect("golden is JSON");
    assert!(validator.is_valid(&golden));
    assert_eq!(golden["profile"], "expert-state");
    assert_eq!(golden["player"]["block"], 6);
    assert_eq!(golden["player"]["potions"][0]["target_mode"], "any_enemy");
    assert_eq!(golden["legal_actions"][1]["action"]["kind"], "use_potion");

    let mut unknown = golden.clone();
    unknown["player"]["future_rng"] = json!(123);
    assert!(!validator.is_valid(&unknown));

    let mut unavailable = golden.clone();
    unavailable["player"]["powers"] = Value::Null;
    unavailable["player"]["deck"] = Value::Null;
    assert!(validator.is_valid(&unavailable));
}

#[test]
fn runtime_v4_expert_checksum_inventory_is_closed_and_v3_is_unchanged() {
    let paths = [
        "../../conformance/cases/runtime-v4-expert.json",
        "../../schemas/runtime-v4-expert.schema.json",
        "manifest.json",
        "schema.json",
        "golden/observation.json",
    ];
    for path in paths {
        let line = CHECKSUMS
            .lines()
            .find(|line| line.ends_with(&format!("  {path}")))
            .expect("checksum inventory contains every contract input");
        let (digest, _) = line.split_once("  ").expect("checksum has two columns");
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    assert_eq!(
        RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST,
        "8e99cea36b7ede97532348fd8efe302ca79260895265a7bf14ddf7e006d8ff63"
    );
}
