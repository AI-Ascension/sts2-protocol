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
        "d23ca838a7be875f32242123955b4a27782bac04"
    );
    assert_eq!(
        evidence["source"]["tree"],
        "23336ca834b5870d15ee6369c101d5c67ff34caf"
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
            "8940fba823a0893b31d1a96301831c182d37ed32",
            "99db77111acec45f87ab3d47bb2d48c064614e49",
        ),
        (
            "sts2-mcp-server",
            "f3b6eaa8bcf2241b8d6c47587c958388a8fe1031",
            "ef60b30ca2b2ea3d6c51b1c79c7bd887e82d4d32",
        ),
        (
            "sts2-harness",
            "9e85c29049e942140b97d8fbab2e52ba95475964",
            "0419eba01298b426bb3ae557e4adcaa581efb63e",
        ),
    ]) {
        assert_eq!(consumer["name"], name);
        assert_eq!(consumer["result"], "pass");
        assert_eq!(consumer["live_status"], "unverified");
        assert_eq!(consumer["commit"], commit);
        assert_eq!(consumer["tree"], tree);
    }
    assert_eq!(
        consumers[2]["verification_scope"],
        "current-main native co-op wire, coordinator, artifact-verification, and replay component tests"
    );
    assert_eq!(evidence["cross_boundary"]["catalog_action_count"], 2);
    assert_eq!(evidence["cross_boundary"]["catalog_vote_count"], 1);
    assert_eq!(evidence["cross_boundary"]["source_to_consumer"], "pass");
    assert_eq!(evidence["admission"]["artifact_admission"], "component");
    assert_eq!(
        evidence["admission"]["registered_consumers"],
        serde_json::json!(["sts2-gateway", "sts2-mcp-server", "sts2-harness"])
    );
}
