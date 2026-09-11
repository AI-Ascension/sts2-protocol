// SPDX-License-Identifier: MIT

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const PRODUCER_CAPTURE: &str =
    include_str!("../../../artifacts/coop-native-v1/producer-capture.json");

macro_rules! captured {
    ($name:literal) => {
        (
            concat!("capture/", $name),
            include_str!(concat!("../../../artifacts/coop-native-v1/capture/", $name)),
        )
    };
}

macro_rules! golden {
    ($name:literal) => {
        (
            concat!("golden/", $name),
            include_str!(concat!("../../../artifacts/coop-native-v1/golden/", $name)),
        )
    };
}

const CAPTURED: &[(&str, &str)] = &[
    captured!("observation.json"),
    captured!("legal-catalog.json"),
    captured!("local-action-settled.json"),
    captured!("local-action-rejected.json"),
    captured!("local-action-unknown.json"),
    captured!("local-action-recovered.json"),
    captured!("shared-vote-settled.json"),
    captured!("rejoin-pending.json"),
    captured!("rejoin-recovered.json"),
];

const GOLDENS: &[(&str, &str)] = &[
    golden!("observation-response.json"),
    golden!("legal-catalog-request.json"),
    golden!("legal-catalog-response.json"),
    golden!("local-action-settled-request.json"),
    golden!("local-action-settled-response.json"),
    golden!("local-action-rejected-request.json"),
    golden!("local-action-rejected-response.json"),
    golden!("local-action-unknown-request.json"),
    golden!("local-action-unknown-response.json"),
    golden!("local-action-recovered-request.json"),
    golden!("local-action-recovered-response.json"),
    golden!("shared-vote-settled-request.json"),
    golden!("shared-vote-settled-response.json"),
    golden!("rejoin-pending-request.json"),
    golden!("rejoin-pending-response.json"),
    golden!("rejoin-recovered-request.json"),
    golden!("rejoin-recovered-response.json"),
];

fn json(text: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(text)
}

fn captured(path: &str) -> &'static str {
    CAPTURED
        .iter()
        .find_map(|(listed, text)| (*listed == path).then_some(*text))
        .unwrap_or_else(|| panic!("unknown captured wrapper {path}"))
}

fn golden(path: &str) -> &'static str {
    GOLDENS
        .iter()
        .find_map(|(listed, text)| (*listed == path).then_some(*text))
        .unwrap_or_else(|| panic!("unknown golden projection {path}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn fresh_serialized_wrappers_match_recorded_projections() -> Result<(), Box<dyn std::error::Error>>
{
    let capture = json(PRODUCER_CAPTURE)?;
    assert_eq!(capture["report_version"], 3);
    assert_eq!(capture["status"], "source_derived_component_accepted");
    assert_eq!(
        capture["source"]["commit"],
        "d23ca838a7be875f32242123955b4a27782bac04"
    );
    assert_eq!(
        capture["source"]["tree"],
        "23336ca834b5870d15ee6369c101d5c67ff34caf"
    );
    assert_eq!(
        capture["capture_run"]["id"],
        "coop-native-source-only-20260910-r7"
    );
    assert_eq!(capture["capture_run"]["status"], "fresh_source_only_pass");
    assert_eq!(capture["capture_run"]["sdk"], "9.0.317");
    assert_eq!(capture["capture_run"]["output_directory"], "capture");
    assert_eq!(capture["capture_run"]["wrapper_count"], CAPTURED.len());
    assert_eq!(capture["capture_run"]["projection_match"], true);

    let records = capture["captures"].as_array().ok_or("missing captures")?;
    assert_eq!(records.len(), CAPTURED.len());
    let mut seen_wrappers = HashSet::new();
    let mut seen_goldens = HashSet::new();
    for record in records {
        let wrapper_path = record["serialized_wrapper"]
            .as_str()
            .ok_or("missing wrapper")?;
        assert!(
            seen_wrappers.insert(wrapper_path),
            "duplicate wrapper {wrapper_path}"
        );
        let wrapper_text = captured(wrapper_path);
        assert_eq!(
            sha256_hex(wrapper_text.as_bytes()),
            record["wrapper_sha256"]
        );
        let wrapper = json(wrapper_text)?;
        assert_eq!(wrapper["capture"]["name"], record["name"]);
        assert_eq!(
            wrapper["schema_digest"],
            capture["producer"]["declared_schema_digest"]
        );

        for member in record["members"].as_array().ok_or("missing members")? {
            let pointer = member["source_pointer"]
                .as_str()
                .ok_or("missing source pointer")?;
            let golden_path = member["golden"].as_str().ok_or("missing golden")?;
            assert!(
                seen_goldens.insert(golden_path),
                "duplicate golden {golden_path}"
            );
            let captured_member = wrapper.pointer(pointer).ok_or("missing captured member")?;
            let golden_member = json(golden(golden_path))?;
            assert_eq!(captured_member, &golden_member, "{golden_path}");
        }
    }
    assert_eq!(seen_goldens.len(), GOLDENS.len());
    Ok(())
}
