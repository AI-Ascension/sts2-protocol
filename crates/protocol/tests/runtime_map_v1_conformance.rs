// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_MAP_V1_MAX_MESSAGE_BYTES, RUNTIME_MAP_V1_SCHEMA_DIGEST, RuntimeMapV1ActionBinding,
    RuntimeMapV1Availability, RuntimeMapV1Completeness, RuntimeMapV1DecodeError,
    RuntimeMapV1Message, RuntimeMapV1NavigationAction, RuntimeMapV1Position, RuntimeMapV1Snapshot,
    RuntimeMapV1ValidationError, canonical_map_message_json, decode_map_snapshot,
    decode_runtime_map_message, map_content_digest, map_navigation_digest, map_snapshot_digest,
    map_topology_digest,
};

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-map-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-map-v1/schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-map-v1.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-map-v1/SHA256SUMS");
const REQUEST: &str =
    include_str!("../../../artifacts/runtime-map-v1/golden/snapshot-request.json");
const RESPONSE: &str =
    include_str!("../../../artifacts/runtime-map-v1/golden/snapshot-response.json");
const SNAPSHOT: &str = include_str!("../../../artifacts/runtime-map-v1/golden/visible-map.json");

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
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

fn response_message() -> RuntimeMapV1Message {
    decode_runtime_map_message(RESPONSE.as_bytes()).expect("response fixture validates")
}

fn response_snapshot() -> RuntimeMapV1Snapshot {
    response_message()
        .snapshot
        .expect("response has a snapshot")
}

#[test]
fn runtime_map_v1_goldens_validate_and_round_trip() {
    for (name, text) in [("request", REQUEST), ("response", RESPONSE)] {
        let message = decode_runtime_map_message(text.as_bytes())
            .unwrap_or_else(|error| panic!("{name} must validate: {error}"));
        let canonical = canonical_map_message_json(&message).expect("canonical encoding succeeds");
        assert_eq!(canonical, payload(text), "{name} canonical bytes");
    }
    let snapshot = decode_map_snapshot(SNAPSHOT.as_bytes()).expect("standalone snapshot validates");
    assert_eq!(
        snapshot.canonical_json().expect("snapshot canonical bytes"),
        payload(SNAPSHOT)
    );
}

#[test]
fn runtime_map_v1_schema_and_case_are_bound_to_artifact() {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let source: Value = serde_json::from_str(SOURCE_SCHEMA).expect("source schema is JSON");
    let case: Value = serde_json::from_str(CASE).expect("conformance case is JSON");
    assert_eq!(source["$id"], "sts2-runtime-map-v1");
    assert_eq!(case["contract"], "sts2.protocol/runtime-map-v1");
    assert_eq!(case["checksums"], "artifacts/runtime-map-v1/SHA256SUMS");
    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("schema compiles as Draft 2020-12");
    for text in [REQUEST, RESPONSE, SNAPSHOT] {
        let value: Value = serde_json::from_str(text).expect("fixture is JSON");
        assert!(validator.is_valid(&value));
    }
    let mut unknown: Value = serde_json::from_str(RESPONSE).expect("response is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));
    let mut nested: Value = serde_json::from_str(RESPONSE).expect("response is JSON");
    nested["snapshot"]["nodes"][0]["future_node"] = json!(true);
    assert!(!validator.is_valid(&nested));
}

#[test]
fn runtime_map_v1_checksum_inventory_covers_contract_inputs() {
    for path in [
        "../../conformance/cases/runtime-map-v1.json",
        "../../schemas/runtime-map-v1.schema.json",
        "manifest.json",
        "schema.json",
        "golden/snapshot-request.json",
        "golden/snapshot-response.json",
        "golden/visible-map.json",
    ] {
        assert_eq!(checksum_for(CHECKSUMS, path).len(), 64, "{path}");
    }
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../artifacts/runtime-map-v1/manifest.json"
    ))
    .expect("manifest is JSON");
    assert_eq!(manifest["schema_digest"], RUNTIME_MAP_V1_SCHEMA_DIGEST);
}

#[test]
fn runtime_map_v1_rejects_adversarial_graph_and_binding_shapes() {
    let mut duplicate_node = response_snapshot();
    duplicate_node.nodes.push(duplicate_node.nodes[0].clone());
    assert_eq!(
        duplicate_node.validate(),
        Err(RuntimeMapV1ValidationError::DuplicateNode)
    );

    let mut duplicate_edge = response_snapshot();
    duplicate_edge.edges.push(duplicate_edge.edges[0].clone());
    assert_eq!(
        duplicate_edge.validate(),
        Err(RuntimeMapV1ValidationError::DuplicateEdge)
    );

    let mut cyclic = response_snapshot();
    cyclic.edges.push(sts2_protocol::RuntimeMapV1Edge {
        from: "map:1:2:0".into(),
        to: "map:1:0:0".into(),
    });
    assert_eq!(
        cyclic.validate(),
        Err(RuntimeMapV1ValidationError::CyclicGraph)
    );

    let mut mismatched = response_snapshot();
    mismatched.bindings[0].action = RuntimeMapV1NavigationAction::SelectMapNode {
        node_id: "map:1:2:0".into(),
    };
    assert_eq!(
        mismatched.validate(),
        Err(RuntimeMapV1ValidationError::BindingPayloadMismatch)
    );

    let mut stale = response_message();
    stale.generation += 1;
    assert_eq!(
        stale.validate(),
        Err(RuntimeMapV1ValidationError::ResponseShape)
    );
}

#[test]
fn runtime_map_v1_accepts_truthful_unavailable_and_complete_empty_states() {
    let mut unavailable = response_snapshot();
    unavailable.availability = RuntimeMapV1Availability::Unavailable;
    unavailable.completeness = RuntimeMapV1Completeness::Unknown;
    unavailable.map_instance_id = None;
    unavailable.act_id = None;
    unavailable.scope_id = None;
    unavailable.reason = Some("map screen is not observable".into());
    unavailable.nodes.clear();
    unavailable.edges.clear();
    unavailable.position = RuntimeMapV1Position::Unavailable {};
    unavailable.history.clear();
    unavailable.terminal_node_ids.clear();
    unavailable.bindings.clear();
    unavailable
        .validate()
        .expect("unavailable projection is explicit");

    let mut empty = unavailable;
    empty.availability = RuntimeMapV1Availability::Available;
    empty.completeness = RuntimeMapV1Completeness::Complete;
    empty.map_instance_id = Some("map-instance-empty".into());
    empty.act_id = Some(1);
    empty.scope_id = Some("campaign-empty".into());
    empty.reason = None;
    empty.position = RuntimeMapV1Position::PreStart {};
    empty
        .validate()
        .expect("complete empty pre-start projection is valid");
}

#[test]
fn runtime_map_v1_preserves_visible_overlaps_and_disconnected_components() {
    let mut projection = response_snapshot();
    projection.nodes[1].row = projection.nodes[0].row;
    projection.nodes[1].column = projection.nodes[0].column;
    projection.nodes.push(sts2_protocol::RuntimeMapV1Node {
        id: "map:1:9:9".into(),
        row: 9,
        column: 9,
        category: sts2_protocol::RuntimeMapV1RoomCategory::Unknown,
        visited: false,
    });
    projection.validate().expect(
        "coordinates and visible disconnected components are projection facts, not inferred topology errors",
    );
}

#[test]
fn runtime_map_v1_canonical_and_independent_digests_are_stable() {
    let message = response_message();
    let snapshot = message.snapshot.as_ref().expect("snapshot");
    let canonical = canonical_map_message_json(&message).expect("canonical message");
    let mut reordered = message.clone();
    let reordered_snapshot = reordered.snapshot.as_mut().expect("snapshot");
    reordered_snapshot.nodes.reverse();
    reordered_snapshot.edges.reverse();
    reordered_snapshot.bindings.reverse();
    reordered_snapshot.terminal_node_ids.reverse();
    assert_eq!(
        canonical,
        canonical_map_message_json(&reordered).expect("same canonical bytes")
    );

    let full = map_snapshot_digest(snapshot).expect("full digest");
    let content = map_content_digest(snapshot).expect("content digest");
    let topology = map_topology_digest(snapshot).expect("topology digest");
    let navigation = map_navigation_digest(snapshot).expect("navigation digest");
    assert!(
        [&full, &content, &topology, &navigation]
            .iter()
            .all(|digest| digest.len() == 64)
    );

    let mut content_changed = snapshot.clone();
    content_changed.nodes[2].visited = !content_changed.nodes[2].visited;
    assert_ne!(
        content,
        map_content_digest(&content_changed).expect("changed content digest")
    );
    assert_eq!(
        topology,
        map_topology_digest(&content_changed).expect("unchanged topology digest")
    );

    let mut navigation_changed = snapshot.clone();
    navigation_changed.bindings[0]
        .host_action_id
        .push_str(":changed");
    assert_ne!(
        navigation,
        map_navigation_digest(&navigation_changed).expect("changed navigation digest")
    );
}

#[test]
fn runtime_map_v1_decoder_checks_bytes_before_allocation_and_duplicate_keys() {
    let oversized = vec![b' '; RUNTIME_MAP_V1_MAX_MESSAGE_BYTES + 1];
    assert!(matches!(
        decode_runtime_map_message(&oversized),
        Err(RuntimeMapV1DecodeError::TooLarge { .. })
    ));
    let duplicate = payload(REQUEST).replacen(
        "\"generation\":42",
        "\"generation\":42,\"generation\":42",
        1,
    );
    assert!(matches!(
        decode_runtime_map_message(duplicate.as_bytes()),
        Err(RuntimeMapV1DecodeError::DuplicateKey)
    ));
}

#[test]
fn runtime_map_v1_does_not_expose_harness_or_hidden_state_fields() {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("schema");
    let snapshot = &schema["$defs"]["snapshot"];
    for forbidden in [
        "harness_id",
        "provider",
        "rng",
        "future_nodes",
        "raw_host_object",
    ] {
        assert!(
            snapshot["properties"].get(forbidden).is_none(),
            "{forbidden}"
        );
    }
    let binding: RuntimeMapV1ActionBinding = RuntimeMapV1ActionBinding {
        graph_node_id: "map:1:1:0".into(),
        host_action_id: "select-map-node:42:map:1:1:0".into(),
        action: RuntimeMapV1NavigationAction::SelectMapNode {
            node_id: "map:1:1:0".into(),
        },
    };
    assert_eq!(binding.graph_node_id, "map:1:1:0");
}
