// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn accounting_is_reproducible_and_bounded() {
    for name in [
        "static-page-1-response",
        "static-page-2-response",
        "live-detail-response",
    ] {
        let response = value(name);
        let page = &response["result"]["page"];
        let items = page["items"].as_array().expect("items");
        let max_item = items.iter().map(canonical_bytes).max().unwrap_or(0);
        assert_eq!(page["accounting"]["item_bytes"], json!(max_item));
        assert_eq!(
            page["accounting"]["payload_bytes"],
            json!(canonical_bytes(&page["items"]))
        );
        assert_eq!(
            page["accounting"]["text_bytes"],
            json!(text_bytes(&page["items"]))
        );
        assert!(
            page["accounting"]["item_bytes"].as_u64().unwrap()
                <= page["limits"]["item_bytes"].as_u64().unwrap()
        );
        assert!(
            page["accounting"]["payload_bytes"].as_u64().unwrap()
                <= page["limits"]["page_bytes"].as_u64().unwrap()
        );
        assert!(
            page["accounting"]["text_bytes"].as_u64().unwrap()
                <= page["limits"]["text_bytes"].as_u64().unwrap()
        );
    }
}

#[test]
fn field_and_wire_vectors_are_present_and_inventory_is_exact() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    for vector in case["valid_vectors"].as_array().expect("valid vectors") {
        if let Some(path) = vector["fixture"].as_str() {
            let fixture: Value = serde_json::from_str(
                &fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(path),
                )
                .expect("field fixture"),
            )
            .expect("field fixture JSON");
            assert_eq!(fixture["id"], vector["id"]);
        }
    }
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(manifest["protocol_version"], PROFILE);
    assert_eq!(manifest["checksums"], "SHA256SUMS");
    assert_eq!(
        manifest["goldens"].as_array().map(Vec::len),
        Some(GOLDENS.len())
    );
    let root = artifact_root();
    let mut checked = 0;
    for line in CHECKSUMS.lines().filter(|line| !line.is_empty()) {
        let (expected, path) = line.split_once("  ").expect("checksum columns");
        let bytes = fs::read(root.join(path)).expect("inventory path");
        assert_eq!(digest(&bytes), expected, "inventory digest for {path}");
        checked += 1;
    }
    assert!(checked >= 12);
}
