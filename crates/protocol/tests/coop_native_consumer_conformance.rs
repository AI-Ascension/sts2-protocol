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
            "d5dedd264115472799b780b49fd9a545cb6a1507",
            "7868858f187b6b121981b3e36cf54e85665cefad",
        ),
        (
            "sts2-mcp-server",
            "f376105ab779ea692855557a5ad6fdab32f9891d",
            "679e8eae14f0269ab1dfa0244919ef419684560a",
        ),
        (
            "sts2-harness",
            "4e738133822a48b99bea9a710aa49cf635e7cd2d",
            "f9200703423d98ee2121a79864877342bb2ac5cd",
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
