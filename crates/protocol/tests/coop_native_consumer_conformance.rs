// SPDX-License-Identifier: MIT

use serde_json::Value;

const MANIFEST: &str = include_str!("../../../artifacts/coop-native-v1/manifest.json");
const CASES: &str = include_str!("../../../artifacts/coop-native-v1/conformance.json");
const EVIDENCE: &str = include_str!("../../../artifacts/coop-native-v1/consumer-conformance.json");

#[test]
fn serialized_consumer_evidence_registers_three_component_consumers() {
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest is JSON");
    let cases: Value = serde_json::from_str(CASES).expect("conformance case is JSON");
    let evidence: Value = serde_json::from_str(EVIDENCE).expect("consumer evidence is JSON");

    assert_eq!(
        manifest["consumer_conformance"],
        "consumer-conformance.json"
    );
    assert_eq!(
        manifest["consumer_conformance_status"],
        "component_serialized_conformance"
    );
    assert_eq!(
        cases["serialized_conformance"]["evidence"],
        "consumer-conformance.json"
    );
    assert_eq!(
        cases["serialized_conformance"]["status"],
        "component_serialized_conformance"
    );
    assert_eq!(cases["serialized_conformance"]["live_status"], "unverified");
    assert_eq!(cases["serialized_conformance"]["admission"], "component");

    assert_eq!(evidence["contract"], "sts2.protocol/coop-native-v1");
    assert_eq!(evidence["profile"], "coop-native-v1");
    assert_eq!(
        evidence["schema_digest"],
        "2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629"
    );
    assert_eq!(evidence["status"], "component_serialized_conformance");
    assert_eq!(evidence["source"]["name"], "sts2-game-mod");
    assert_eq!(evidence["source"]["role"], "producer");
    assert_eq!(
        evidence["source"]["commit"],
        "ab702dbbc79bc5854bd0840b44a729834ae50e68"
    );
    assert_eq!(
        evidence["source"]["tree"],
        "e3f0aa9d30fe25585fbb3c1fe8e3c1fcdfa43223"
    );
    assert_eq!(evidence["source"]["result"], "pass");
    assert_eq!(evidence["source"]["live_status"], "unverified");

    let consumers = evidence["consumers"]
        .as_array()
        .expect("serialized consumer evidence is present");
    assert_eq!(consumers.len(), 3);
    assert_eq!(
        consumers
            .iter()
            .map(|consumer| consumer["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["sts2-gateway", "sts2-mcp-server", "sts2-harness"]
    );
    for (consumer, (name, commit, tree)) in consumers.iter().zip([
        (
            "sts2-gateway",
            "de1fe72345ea972d56c05d30837da5327e5f1655",
            "cbf10caa6775ca06adf7fe1e9d6cc98be6453a66",
        ),
        (
            "sts2-mcp-server",
            "47d63f6ce41c9efb3431dea6dc31b39fddc5d79a",
            "a6eb009912a4f624c49398f6cfc25022da65b563",
        ),
        (
            "sts2-harness",
            "a2cb481cfc57475f147d5c8c3783afc6d2b2d63b",
            "083e1dc3ca88c35eedc10a6187a63c91a9864c46",
        ),
    ]) {
        assert_eq!(consumer["name"], name);
        assert_eq!(consumer["result"], "pass");
        assert_eq!(consumer["live_status"], "unverified");
        assert_eq!(consumer["commit"], commit);
        assert_eq!(consumer["tree"], tree);
    }
    assert_eq!(evidence["cross_boundary"]["catalog_action_count"], 2);
    assert_eq!(evidence["cross_boundary"]["catalog_vote_count"], 1);
    assert_eq!(evidence["cross_boundary"]["source_to_consumer"], "pass");
    assert_eq!(evidence["admission"]["artifact_admission"], "component");
    assert_eq!(
        evidence["admission"]["registered_consumers"],
        serde_json::json!(["sts2-gateway", "sts2-mcp-server", "sts2-harness"])
    );
}
