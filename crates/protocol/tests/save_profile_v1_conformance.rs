// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use sts2_protocol::save_profile_v1::{Codec, MAX_MESSAGE_BYTES, SCHEMA, intent_digest};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn golden(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(root().join(format!("artifacts/save-profile-v1/golden/{name}.json"))).unwrap(),
    )
    .unwrap()
}
fn codec() -> Codec {
    Codec::new().unwrap()
}

#[test]
fn standalone_schema_all_goldens_and_complete_inventory() {
    let base = root().join("artifacts/save-profile-v1");
    assert_eq!(
        SCHEMA.as_bytes(),
        fs::read(base.join("schema.json")).unwrap()
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(base.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["owner"], "sts2-protocol");
    assert_eq!(manifest["status"], "candidate");
    assert_eq!(manifest["consumers"], json!([]));
    assert!(manifest["prospective_consumers"].as_array().unwrap().len() >= 2);
    let codec = codec();
    let inventory = fs::read_to_string(base.join("SHA256SUMS")).unwrap();
    let mut listed = std::collections::BTreeSet::new();
    for line in inventory.lines() {
        let (digest, name) = line.split_once("  ").unwrap();
        assert!(listed.insert(name.to_owned()));
        assert_eq!(
            digest,
            Sha256::digest(fs::read(base.join(name)).unwrap())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    for item in fs::read_dir(&base).unwrap() {
        let item = item.unwrap();
        let name = item.file_name().into_string().unwrap();
        if name != "golden" && name != "SHA256SUMS" {
            assert!(listed.contains(&name));
        }
    }
    let names = manifest["golden_files"].as_array().unwrap();
    assert_eq!(
        names.len(),
        fs::read_dir(base.join("golden")).unwrap().count()
    );
    for name in names {
        let name = name.as_str().unwrap();
        assert!(listed.contains(name));
        let bytes = fs::read(base.join(name)).unwrap();
        assert_eq!(codec.encode(&codec.decode(&bytes).unwrap()).unwrap(), bytes);
    }
    assert!(listed.contains("../../schemas/save-profile-v1.schema.json"));
    assert!(listed.contains("../../conformance/cases/save-profile-v1.json"));
}

#[test]
fn recovery_retains_scope_operation_intent_and_original_lease() {
    let codec = codec();
    for action in ["select", "create_disposable"] {
        let original = golden(&format!("{action}-request"));
        let lookup = golden(&format!("{action}-lookup"));
        codec.bind_operation(&original, &lookup).unwrap();
        for outcome in ["not_found", "request_error", "uncertain"] {
            codec
                .bind_lookup(&original, &lookup, &golden(&format!("{action}-{outcome}")))
                .unwrap();
        }
        for status in [
            "accepted",
            "settled",
            "rejected",
            "cancelled",
            "blocked",
            "unknown",
        ] {
            codec
                .bind_operation(&original, &golden(&format!("{action}-{status}")))
                .unwrap();
        }
        for field in ["operation_id", "original_lease_id"] {
            let mut changed = lookup.clone();
            changed["intent"][field] = json!("different");
            changed["intent_digest"] = json!(intent_digest(&changed["intent"]).unwrap());
            codec.validate(&changed).unwrap();
            assert!(codec.bind_operation(&original, &changed).is_err());
        }
        let mut reply = golden(&format!("{action}-not_found"));
        reply["current_lease_id"] = json!("wrong-lease");
        assert!(codec.bind_lookup(&original, &lookup, &reply).is_err());
    }
}

#[test]
fn exact_owner_newer_observation_selected_slot_and_witness_are_required() {
    let codec = codec();
    for action in ["select", "create_disposable"] {
        let settled = golden(&format!("{action}-settled"));
        for field in [
            "instance_id",
            "process_namespace",
            "allocation_id",
            "mod_incarnation",
        ] {
            let mut changed = settled.clone();
            changed["observation"]["owner"][field] = json!("different");
            assert!(codec.validate(&changed).is_err(), "{field}");
        }
        for field in [
            "approved_process_profile",
            "process_generation",
            "mod_generation",
        ] {
            let mut changed = settled.clone();
            changed["observation"]["owner"][field] = json!(17);
            assert!(codec.validate(&changed).is_err(), "{field}");
        }
        for (pointer, replacement) in [
            ("/observation/generation", json!(4)),
            ("/observation/selected_slot_id", json!("unrelated")),
            ("/observation/native_ready", json!(false)),
            ("/observation/active_run", json!(true)),
            ("/observation/baseline", Value::Null),
            ("/witness", Value::Null),
            ("/witness/operation_id", json!("unrelated")),
            ("/witness/intent_digest", json!("c".repeat(64))),
            ("/witness/slot_id", json!("unrelated")),
            ("/witness/allocation_id", json!("unrelated")),
            ("/witness/generation", json!(6)),
            ("/allocation_stage", json!("directory_created")),
        ] {
            let mut changed = settled.clone();
            *changed.pointer_mut(pointer).unwrap() = replacement;
            assert!(codec.validate(&changed).is_err(), "{action}:{pointer}");
        }
    }
}

#[test]
fn discovery_needs_no_baseline_and_creation_cannot_adopt_old_allocation() {
    let codec = codec();
    codec
        .bind_discovery(&golden("discover-request"), &golden("discover-empty"))
        .unwrap();
    let mut create = golden("create_disposable-request");
    create["intent"]["prior_allocation_id"] = create["intent"]["owner"]["allocation_id"].clone();
    create["intent_digest"] = json!(intent_digest(&create["intent"]).unwrap());
    assert!(codec.validate(&create).is_err());
    let mut select = golden("select-request");
    select["intent"]["baseline"] = Value::Null;
    select["intent_digest"] = json!(intent_digest(&select["intent"]).unwrap());
    assert!(codec.validate(&select).is_err());
    for status in ["accepted", "rejected", "cancelled", "blocked", "unknown"] {
        let mut value = golden(&format!("create_disposable-{status}"));
        value["allocation_stage"] = json!("native_ready");
        assert!(codec.validate(&value).is_err());
    }
}

#[test]
fn raw_duplicate_unknown_unsafe_numeric_and_size_faults() {
    let codec = codec();
    let request = golden("select-request");
    let raw = serde_json::to_string(&request).unwrap();
    for old_new in [
        ("\"kind\":", "\"kind\":\"operation_request\",\"kind\":"),
        ("\"slot_id\":", "\"slot_id\":\"duplicate\",\"slot_id\":"),
        (
            "\"process_generation\":3",
            "\"process_generation\":9007199254740992",
        ),
        (
            "\"process_generation\":3",
            "\"process_generation\":3.0000000000000001",
        ),
        ("\"process_generation\":3", "\"process_generation\":1e999"),
        ("\"process_generation\":3", "\"process_generation\":-1"),
        ("\"action\":\"select\"", "\"action\":\"delete\""),
        ("\"kind\":", "\"path\":\"forbidden\",\"kind\":"),
        ("save-profile-v1", "save-profile-v9"),
    ] {
        assert!(raw.contains(old_new.0));
        assert!(
            codec
                .decode(raw.replace(old_new.0, old_new.1).as_bytes())
                .is_err()
        );
    }
    let exact = raw.replace("\"process_generation\":3", "\"process_generation\":3e0");
    assert_eq!(codec.decode(exact.as_bytes()).unwrap(), request);
    assert!(codec.decode(&vec![b' '; MAX_MESSAGE_BYTES + 1]).is_err());
    assert!(codec.decode(&[255]).is_err());
    let mut changed = request.clone();
    changed["schema_digest"] = json!("f".repeat(64));
    assert!(codec.validate(&changed).is_err());
    changed = request;
    changed["intent"]["owner"]["extra"] = json!(true);
    assert!(codec.validate(&changed).is_err());
}

#[test]
fn independent_fixed_intent_hashes() {
    for (action, expected) in [
        (
            "select",
            "3ddec2a15f478c9d8039817e083353b1e7155adde07615024b57261e9147ad28",
        ),
        (
            "create_disposable",
            "b70af704f717e83f6bb06ef8fbdba072308bdb65dc8885daa9dcf89089d6c57a",
        ),
    ] {
        assert_eq!(
            intent_digest(&golden(&format!("{action}-request"))["intent"]).unwrap(),
            expected
        );
    }
}

#[test]
fn recovered_receipts_bind_current_lookup_lease_without_changing_intent() {
    let codec = codec();
    let original = golden("select-request");
    let lookup = golden("select-lookup");
    for status in [
        "accepted",
        "settled",
        "rejected",
        "cancelled",
        "blocked",
        "unknown",
    ] {
        let mut receipt = golden(&format!("select-{status}"));
        assert!(codec.bind_lookup(&original, &lookup, &receipt).is_err());
        receipt["current_lease_id"] = lookup["current_lease_id"].clone();
        codec.bind_lookup(&original, &lookup, &receipt).unwrap();
    }
    let mut value = original;
    value["intent"]["owner"]["process_generation"] = json!(3.0);
    assert!(codec.encode(&value).is_err());
}

#[test]
fn scope_baseline_owner_types_and_nested_closure_faults() {
    let codec = codec();
    for (name, pointer, replacement) in [
        (
            "select-request",
            "/intent/scope",
            json!("create_disposable_profile"),
        ),
        (
            "create_disposable-request",
            "/intent/scope",
            json!("select_profile"),
        ),
        (
            "select-request",
            "/intent/prior_allocation_id",
            json!("allocation-old"),
        ),
        (
            "create_disposable-request",
            "/intent/baseline",
            json!("a".repeat(64)),
        ),
        (
            "create_disposable-request",
            "/intent/prior_allocation_id",
            Value::Null,
        ),
        (
            "select-request",
            "/intent/owner/approved_process_profile",
            json!("automation-disposable-v1"),
        ),
    ] {
        let mut value = golden(name);
        *value.pointer_mut(pointer).unwrap() = replacement;
        value["intent_digest"] = json!(intent_digest(&value["intent"]).unwrap());
        assert!(codec.validate(&value).is_err(), "{pointer}");
    }
    for object in ["observation", "witness"] {
        let mut value = golden("select-settled");
        value[object]["unknown"] = json!(true);
        assert!(codec.validate(&value).is_err());
    }
    let mut older = golden("select-settled");
    older["observation"]["generation"] = json!(3);
    older["witness"]["generation"] = json!(3);
    assert!(codec.validate(&older).is_err());
}
