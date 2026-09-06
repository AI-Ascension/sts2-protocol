// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{CoopSynchronizationMessage, canonical_json, decode_json};

const SCHEMA: &str = include_str!("../../../schemas/coop-synchronization-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/coop-synchronization-v1/schema.json");
const CASES: &str = include_str!("../../../conformance/cases/coop-synchronization-v1.json");
const GOLDEN: &str =
    include_str!("../../../artifacts/coop-synchronization-v1/golden/synchronized.json");

fn valid(value: &Value) -> bool {
    serde_json::from_value::<CoopSynchronizationMessage>(value.clone())
        .is_ok_and(|message| message.validate().is_ok())
}

#[test]
fn every_shared_vector_matches_schema_and_semantic_oracles()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(SCHEMA, ARTIFACT_SCHEMA);
    assert_eq!(
        CASES,
        include_str!("../../../artifacts/coop-synchronization-v1/conformance.json")
    );
    let schema: Value = serde_json::from_str(SCHEMA)?;
    let validator = jsonschema::draft202012::options().build(&schema)?;
    let cases: Value = serde_json::from_str(CASES)?;
    let cases = cases["cases"].as_array().ok_or("missing cases")?;
    assert!(cases.len() >= 25);
    for case in cases {
        assert_eq!(
            validator.is_valid(&case["value"]),
            case["schema_valid"] == true,
            "schema: {}",
            case["name"]
        );
        assert_eq!(
            valid(&case["value"]),
            case["valid"] == true,
            "semantic: {}",
            case["name"]
        );
    }
    Ok(())
}

#[test]
fn all_objects_require_exact_fields() -> Result<(), Box<dyn std::error::Error>> {
    let golden: Value = serde_json::from_str(GOLDEN)?;
    for path in ["", "/provenance", "/players/0", "/synchronization"] {
        let keys: Vec<String> = golden
            .pointer(path)
            .and_then(Value::as_object)
            .ok_or("missing object")?
            .keys()
            .cloned()
            .collect();
        for key in keys {
            let mut value = golden.clone();
            value
                .pointer_mut(path)
                .and_then(Value::as_object_mut)
                .ok_or("missing object")?
                .remove(&key);
            assert!(!valid(&value), "missing {path}/{key}");
        }
        let mut value = golden.clone();
        value
            .pointer_mut(path)
            .and_then(Value::as_object_mut)
            .ok_or("missing object")?
            .insert("unsupported".to_owned(), json!(null));
        assert!(!valid(&value), "extra field at {path}");
    }
    Ok(())
}

#[test]
fn duplicate_keys_and_non_integer_wire_tokens_are_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    let message: CoopSynchronizationMessage = decode_json(GOLDEN)?;
    let canonical = canonical_json(&message)?;
    let roundtrip: CoopSynchronizationMessage = decode_json(&canonical)?;
    assert_eq!(message, roundtrip);
    assert_eq!(canonical_json(&roundtrip)?, canonical);
    for wire in [
        canonical.replacen("\"generation\":4", "\"generation\":4,\"generation\":4", 1),
        canonical.replacen("\"generation\":4", "\"generation\":4.0", 1),
        canonical.replacen(
            "\"role\":\"local\"",
            "\"role\":\"local\",\"role\":\"local\"",
            1,
        ),
        canonical.replacen(
            "\"status\":\"synchronized\"",
            "\"status\":\"synchronized\",\"status\":\"synchronized\"",
            1,
        ),
    ] {
        assert_ne!(wire, canonical);
        assert!(decode_json::<CoopSynchronizationMessage>(&wire).is_err());
    }
    Ok(())
}

#[test]
fn manifest_names_the_two_complete_wire_consumers() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../artifacts/coop-synchronization-v1/manifest.json"
    ))?;
    assert_eq!(
        manifest["consumers"],
        json!(["sts2-gateway", "sts2-mcp-server"])
    );
    assert_eq!(
        manifest["schema_digest"],
        sts2_protocol::COOP_SYNC_SCHEMA_DIGEST
    );
    assert_eq!(
        manifest["protocol_version"],
        sts2_protocol::COOP_SYNC_PROTOCOL_VERSION
    );
    assert_eq!(
        manifest["goldens"]
            .as_array()
            .ok_or("goldens missing")?
            .len(),
        3
    );
    Ok(())
}
