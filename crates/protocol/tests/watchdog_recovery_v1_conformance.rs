// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/watchdog-recovery-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/watchdog-recovery-v1/schema.json");
const MANIFEST: &str = include_str!("../../../artifacts/watchdog-recovery-v1/manifest.json");
const CASES: &str = include_str!("../../../conformance/cases/watchdog-recovery-v1.json");
const ARTIFACT_CASES: &str =
    include_str!("../../../artifacts/watchdog-recovery-v1/conformance.json");
const RCJ_VECTORS: &str = include_str!("../../../artifacts/watchdog-recovery-v1/rcj-vectors.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/watchdog-recovery-v1/SHA256SUMS");

macro_rules! valid_fixture {
    ($kind:literal, $name:literal) => {
        (
            $kind,
            include_str!(concat!(
                "../../../artifacts/watchdog-recovery-v1/fixtures/valid/",
                $name
            )),
        )
    };
}

const VALID: &[(&str, &str)] = &[
    valid_fixture!("bootstrap_request", "bootstrap-request.json"),
    valid_fixture!("bootstrap_response", "bootstrap-response.json"),
    valid_fixture!("host_fence_request", "host-fence-request.json"),
    valid_fixture!("host_fence_response", "host-fence-response.json"),
    valid_fixture!("lease_acquire_request", "lease-acquire-request.json"),
    valid_fixture!("lease_acquire_response", "lease-acquire-response.json"),
    valid_fixture!("lease_renew_request", "lease-renew-request.json"),
    valid_fixture!("lease_renew_response", "lease-renew-response.json"),
    valid_fixture!("lease_revoke_request", "lease-revoke-request.json"),
    valid_fixture!("lease_revoke_response", "lease-revoke-response.json"),
    valid_fixture!("operation_intent_request", "operation-intent-request.json"),
    valid_fixture!(
        "operation_intent_response",
        "operation-intent-response.json"
    ),
    valid_fixture!(
        "operation_dispatch_request",
        "operation-dispatch-request.json"
    ),
    valid_fixture!(
        "operation_dispatch_response",
        "operation-dispatch-response.json"
    ),
    valid_fixture!("operation_lookup_request", "operation-lookup-request.json"),
    valid_fixture!(
        "operation_lookup_response",
        "operation-lookup-response.json"
    ),
    valid_fixture!(
        "operation_reconcile_request",
        "operation-reconcile-request.json"
    ),
    valid_fixture!(
        "operation_reconcile_response",
        "operation-reconcile-response.json"
    ),
];

const INVALID: &[(&str, &str)] = &[
    (
        "oversized-action",
        include_str!(
            "../../../artifacts/watchdog-recovery-v1/fixtures/invalid/oversized-action.json"
        ),
    ),
    (
        "stale-contract",
        include_str!(
            "../../../artifacts/watchdog-recovery-v1/fixtures/invalid/stale-contract.json"
        ),
    ),
    (
        "unknown-field",
        include_str!("../../../artifacts/watchdog-recovery-v1/fixtures/invalid/unknown-field.json"),
    ),
];

fn schema() -> jsonschema::Validator {
    let mut source: Value = serde_json::from_str(SOURCE_SCHEMA).expect("source schema is JSON");
    // The contract intentionally repeats the 65,536-byte bound in its regex and
    // maxLength. Rust's regex compiler rejects that large repetition count;
    // replacing only the redundant regex bound leaves the tested language
    // unchanged while keeping the checked-in contract bytes exact.
    source["$defs"]["v3_action"]["properties"]["canonical_json_b64"]["pattern"] =
        json!("^[A-Za-z0-9+/=_-]+$");
    jsonschema::draft202012::options()
        .build(&source)
        .expect("recovery schema compiles as Draft 2020-12")
}

#[test]
fn source_artifact_manifest_and_case_are_bound() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest is JSON");
    assert_eq!(manifest["contract"], "watchdog-recovery-v1");
    assert_eq!(
        manifest["schema_digest"],
        "fb934d3157485aaf6e13e6ebbb213ec8a14c7fc6f5eeebc06b7a22c1f0009217"
    );
    assert_eq!(
        manifest["consumers"],
        json!([
            "ascension-watchdog",
            "sts2-game-mod",
            "sts2-gateway",
            "sts2-harness",
            "sts2-mcp-server"
        ])
    );
    assert_eq!(CASES, ARTIFACT_CASES);
    let cases: Value = serde_json::from_str(CASES).expect("conformance case is JSON");
    assert_eq!(cases["schema_digest"], manifest["schema_digest"]);
    assert_eq!(
        cases["fixtures"]["valid"].as_array().map(Vec::len),
        Some(18)
    );
    assert_eq!(
        cases["coverage"]["request_kinds"].as_array().map(Vec::len),
        Some(9)
    );
    assert_eq!(
        cases["coverage"]["response_kinds"].as_array().map(Vec::len),
        Some(9)
    );
    assert_eq!(
        cases["assertions"]["rcj_requires_exact_canonical_bytes"],
        true
    );
    assert_eq!(
        cases["assertions"]["rcj_rejects_duplicate_unicode_float_escape_and_malformed_base64_inputs"],
        true
    );
}

#[test]
fn every_request_and_response_fixture_matches_the_closed_schema() {
    let validator = schema();
    let mut kinds = BTreeSet::new();
    for (expected_kind, text) in VALID {
        let value: Value = serde_json::from_str(text).expect("valid fixture is JSON");
        assert!(
            validator.is_valid(&value),
            "fixture {expected_kind} must validate"
        );
        assert_eq!(value["kind"], *expected_kind);
        kinds.insert(expected_kind.to_string());
    }
    assert_eq!(kinds.len(), 18);
    for (name, text) in INVALID {
        let value: Value = serde_json::from_str(text).expect("invalid fixture is JSON");
        assert!(
            !validator.is_valid(&value),
            "fixture {name} must be rejected"
        );
    }
}

#[test]
fn nested_and_top_level_unknown_members_are_rejected() {
    let validator = schema();
    let mut top: Value = serde_json::from_str(VALID[0].1).expect("fixture is JSON");
    top["unexpected"] = json!(true);
    assert!(!validator.is_valid(&top));

    let mut nested: Value = serde_json::from_str(VALID[0].1).expect("fixture is JSON");
    nested["payload"]["release"]["unapproved_digest"] = json!("x");
    assert!(!validator.is_valid(&nested));
}

#[test]
fn checksum_inventory_covers_every_release_input() {
    for path in [
        "../../conformance/cases/watchdog-recovery-v1.json",
        "../../schemas/watchdog-recovery-v1.schema.json",
        "manifest.json",
        "schema.json",
        "conformance.json",
        "rcj-vectors.json",
        "README.md",
        "fixtures/README.md",
        "fixtures/invalid/oversized-action.json",
        "fixtures/invalid/stale-contract.json",
        "fixtures/invalid/unknown-field.json",
        "fixtures/valid/bootstrap-request.json",
        "fixtures/valid/bootstrap-response.json",
        "fixtures/valid/host-fence-request.json",
        "fixtures/valid/host-fence-response.json",
        "fixtures/valid/lease-acquire-request.json",
        "fixtures/valid/lease-acquire-response.json",
        "fixtures/valid/lease-renew-request.json",
        "fixtures/valid/lease-renew-response.json",
        "fixtures/valid/lease-revoke-request.json",
        "fixtures/valid/lease-revoke-response.json",
        "fixtures/valid/operation-dispatch-request.json",
        "fixtures/valid/operation-dispatch-response.json",
        "fixtures/valid/operation-intent-request.json",
        "fixtures/valid/operation-intent-response.json",
        "fixtures/valid/operation-lookup-request.json",
        "fixtures/valid/operation-lookup-response.json",
        "fixtures/valid/operation-reconcile-request.json",
        "fixtures/valid/operation-reconcile-response.json",
    ] {
        assert_eq!(checksum_for(CHECKSUMS, path).len(), 64, "{path}");
    }
    assert_eq!(
        checksum_for(CHECKSUMS, "schema.json"),
        "fb934d3157485aaf6e13e6ebbb213ec8a14c7fc6f5eeebc06b7a22c1f0009217"
    );
    assert_eq!(
        checksum_for(CHECKSUMS, "../../schemas/watchdog-recovery-v1.schema.json"),
        checksum_for(CHECKSUMS, "schema.json")
    );
}

#[test]
fn rcj_vectors_cover_frozen_action_variants_and_reject_malformed_inputs() {
    let vectors: Value = serde_json::from_str(RCJ_VECTORS).expect("RCJ vectors are JSON");
    let valid = vectors["valid_actions"].as_array().expect("valid actions");
    assert_eq!(valid.len(), 8);
    let mut names = BTreeSet::new();
    for vector in valid {
        let wire = vector["canonical_json"].as_str().expect("canonical action");
        let encoded = vector["canonical_json_b64"]
            .as_str()
            .expect("base64 action");
        let decoded = decode_base64(encoded).expect("base64 decodes");
        assert_eq!(decoded, wire.as_bytes());
        assert!(rcj_action_valid(wire), "RCJ action must validate: {wire}");
        let payload_digest = vector["payload_digest"].as_str().expect("payload digest");
        assert_eq!(
            sha256_hex(wire.as_bytes()),
            payload_digest,
            "payload digest must hash the exact canonical action bytes"
        );
        names.insert(vector["name"].as_str().expect("action name").to_owned());
    }
    assert_eq!(
        names,
        [
            "choose_reward",
            "end_turn",
            "event_choice",
            "play_card",
            "select_map_node",
            "shop_purchase",
            "shop_remove",
            "start_run"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );

    for malformed in vectors["malformed_actions"]
        .as_array()
        .expect("malformed actions")
    {
        let wire = malformed["wire"].as_str().expect("malformed wire");
        assert!(!rcj_action_valid(wire), "malformed RCJ accepted: {wire}");
    }

    for malformed in vectors["malformed_base64"]
        .as_array()
        .expect("malformed base64 vectors")
    {
        let value = malformed["value"].as_str().expect("malformed base64 value");
        assert!(
            decode_base64(value).is_none(),
            "malformed base64 accepted: {value}"
        );
    }
}

fn rcj_action_valid(wire: &str) -> bool {
    if wire.is_empty()
        || wire.bytes().any(|byte| {
            !byte.is_ascii()
                || byte.is_ascii_whitespace()
                || byte == b'\\'
                || byte.is_ascii_control()
        })
    {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(wire) else {
        return false;
    };
    let Ok(canonical) = serde_json::to_string(&value) else {
        return false;
    };
    if canonical != wire {
        return false;
    }
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.len() != 2 || !object.contains_key("action") || !object.contains_key("action_id") {
        return false;
    }
    let Some(action_id) = object["action_id"].as_str() else {
        return false;
    };
    if !valid_identity(action_id) {
        return false;
    }
    let Some(action) = object["action"].as_object() else {
        return false;
    };
    let Some(kind) = action.get("kind").and_then(Value::as_str) else {
        return false;
    };
    let required = match kind {
        "end_turn" | "skip_reward" | "rest" | "confirm_victory" | "save_quit" | "proceed"
        | "confirm_selection" | "cancel_selection" => &[][..],
        "start_run" => &["character_id"][..],
        "select_map_node" => &["node_id"][..],
        "play_card" => &["card_id", "target_id"][..],
        "choose_reward" => &["reward_id"][..],
        "shop_purchase" => &["item_id"][..],
        "shop_remove" | "smith" | "select_card" => &["card_id"][..],
        "event_choice" => &["choice_id"][..],
        _ => return false,
    };
    if action.len() != required.len() + 1 {
        return false;
    }
    required.iter().all(|field| {
        action.get(*field).is_some_and(|value| {
            value.is_null() && *field == "target_id" || value.as_str().is_some_and(valid_identity)
        })
    })
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.:/-".contains(&byte))
}

fn decode_base64(value: &str) -> Option<Vec<u8>> {
    if value.is_empty() || !value.len().is_multiple_of(4) {
        return None;
    }
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    for (index, chunk) in bytes.chunks(4).enumerate() {
        let is_last = index + 1 == bytes.len() / 4;
        let a = base64_value(chunk[0])?;
        let b = base64_value(chunk[1])?;
        let c_padding = chunk[2] == b'=';
        let d_padding = chunk[3] == b'=';
        if (!is_last && (c_padding || d_padding)) || (c_padding && !d_padding) {
            return None;
        }
        if c_padding {
            if b & 0x0f != 0 {
                return None;
            }
            output.push((a << 2) | (b >> 4));
            continue;
        }
        let c = base64_value(chunk[2])?;
        if d_padding {
            if c & 0x03 != 0 {
                return None;
            }
            output.push((a << 2) | (b >> 4));
            output.push((b << 4) | (c >> 2));
            continue;
        }
        let d = base64_value(chunk[3])?;
        output.push((a << 2) | (b >> 4));
        output.push((b << 4) | (c >> 2));
        output.push((c << 6) | d);
    }
    Some(output)
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn checksum_for<'a>(inventory: &'a str, path: &str) -> &'a str {
    inventory
        .lines()
        .find_map(|line| {
            let (digest, listed_path) = line.split_once("  ")?;
            (listed_path == path).then_some(digest)
        })
        .expect("checksum inventory contains the requested path")
}
