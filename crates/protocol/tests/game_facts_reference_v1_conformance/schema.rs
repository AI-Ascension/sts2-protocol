// SPDX-License-Identifier: MIT

use super::support::{self, TestResult, VECTORS};
use serde_json::Value;
use std::collections::BTreeSet;
use sts2_protocol::game_facts_reference_v1::{
    GAME_FACTS_REFERENCE_V1_ARTIFACT, GAME_FACTS_REFERENCE_V1_PROFILE,
    GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST, GameFactsReferenceV1Codec,
};

#[test]
fn schema_bundle_and_manifest_are_pinned_and_exhaustive() -> TestResult {
    assert_eq!(support::SOURCE_SCHEMA, support::ARTIFACT_SCHEMA);
    assert_eq!(
        support::digest(support::SOURCE_SCHEMA.as_bytes()),
        GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST
    );
    let schema: Value =
        serde_json::from_str(support::SOURCE_SCHEMA).map_err(|error| error.to_string())?;
    let _validator = jsonschema::validator_for(&schema).map_err(|error| error.to_string())?;
    let manifest: Value =
        serde_json::from_str(support::MANIFEST).map_err(|error| error.to_string())?;
    let cases: Value = serde_json::from_str(support::CASE).map_err(|error| error.to_string())?;
    assert_eq!(manifest["artifact"], GAME_FACTS_REFERENCE_V1_ARTIFACT);
    assert_eq!(
        manifest["protocol_version"],
        GAME_FACTS_REFERENCE_V1_PROFILE
    );
    assert_eq!(
        manifest["schema_digest"],
        GAME_FACTS_REFERENCE_V1_SCHEMA_DIGEST
    );
    assert_eq!(manifest["consumers"], serde_json::json!([]));
    assert_eq!(manifest["adoption_evidence"], "unverified");
    assert_eq!(manifest["live_support_evidence"], "unverified");
    assert_eq!(cases["candidate_only"], true);
    assert_eq!(cases["consumer_adoption"], serde_json::json!([]));
    let case_vectors: Vec<_> = cases["vectors"]
        .as_array()
        .ok_or("vectors missing")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let expected_case: Vec<_> = VECTORS.to_vec();
    assert_eq!(case_vectors, expected_case);
    let manifest_vectors: Vec<_> = manifest["goldens"]
        .as_array()
        .ok_or("goldens missing")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let expected_manifest: Vec<_> = VECTORS
        .iter()
        .map(|name| format!("golden/{name}"))
        .collect();
    assert_eq!(
        manifest_vectors,
        expected_manifest
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );

    let expected: BTreeSet<String> = ["README.md", "manifest.json", "schema.json"]
        .into_iter()
        .map(str::to_owned)
        .chain(VECTORS.iter().map(|name| format!("golden/{name}")))
        .collect();
    let mut found = BTreeSet::new();
    let mut previous = None;
    for line in support::CHECKSUMS.lines() {
        let (hash, path) = line.split_once("  ").ok_or("invalid checksum line")?;
        let path = path.trim_start_matches('*');
        assert!(
            previous.is_none_or(|prior: &str| prior < path),
            "checksum paths must be sorted"
        );
        previous = Some(path);
        assert!(expected.contains(path), "unexpected checksum path {path}");
        assert!(
            found.insert(path.to_owned()),
            "duplicate checksum path {path}"
        );
        let bytes = std::fs::read(
            support::root()
                .join("artifacts/game-facts-reference-v1")
                .join(path),
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(support::digest(&bytes), hash, "checksum {path}");
    }
    assert_eq!(found, expected);
    Ok(())
}

#[test]
fn error_golden_kind_is_required_by_schema_and_codec() -> TestResult {
    let bytes = support::golden("error-unknown-id.json")?;
    let baseline: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let schema: Value =
        serde_json::from_str(support::SOURCE_SCHEMA).map_err(|error| error.to_string())?;
    let validator = jsonschema::validator_for(&schema).map_err(|error| error.to_string())?;
    assert!(
        validator.is_valid(&baseline),
        "baseline error golden must satisfy standalone schema"
    );
    assert!(
        GameFactsReferenceV1Codec::decode(&bytes).is_ok(),
        "baseline error golden must decode"
    );

    for kind in ["query_request", "query_response", "capabilities_response"] {
        let mut changed = baseline.clone();
        changed["kind"] = Value::String(kind.to_owned());
        assert!(
            !validator.is_valid(&changed),
            "error payload must reject kind {kind}"
        );
        let changed_bytes = serde_json::to_vec(&changed).map_err(|error| error.to_string())?;
        assert!(
            GameFactsReferenceV1Codec::decode(&changed_bytes).is_err(),
            "codec must reject kind {kind}"
        );
    }
    Ok(())
}
