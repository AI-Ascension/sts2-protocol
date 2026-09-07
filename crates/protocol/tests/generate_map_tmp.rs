use sts2_protocol::*;

fn snapshot() -> RuntimeMapV1Snapshot {
    RuntimeMapV1Snapshot {
        state_id: "map-state-42".into(),
        generation: 42,
        projection_version: "runtime-map-v1".into(),
        game_build: "0.103.2".into(),
        mod_version: "map-mod-1".into(),
        map_instance_id: Some("map-instance-1".into()),
        act_id: Some(1),
        scope_id: Some("campaign-1".into()),
        availability: RuntimeMapV1Availability::Available,
        completeness: RuntimeMapV1Completeness::Complete,
        freshness: RuntimeMapV1Freshness::Current,
        reason: None,
        nodes: vec![
            RuntimeMapV1Node { id: "map:1:0:0".into(), row: 0, column: 0, category: RuntimeMapV1RoomCategory::Start, visited: false },
            RuntimeMapV1Node { id: "map:1:1:0".into(), row: 1, column: 0, category: RuntimeMapV1RoomCategory::Monster, visited: true },
            RuntimeMapV1Node { id: "map:1:1:1".into(), row: 1, column: 1, category: RuntimeMapV1RoomCategory::Event, visited: false },
            RuntimeMapV1Node { id: "map:1:2:0".into(), row: 2, column: 0, category: RuntimeMapV1RoomCategory::Boss, visited: false },
        ],
        edges: vec![
            RuntimeMapV1Edge { from: "map:1:0:0".into(), to: "map:1:1:0".into() },
            RuntimeMapV1Edge { from: "map:1:0:0".into(), to: "map:1:1:1".into() },
            RuntimeMapV1Edge { from: "map:1:1:0".into(), to: "map:1:2:0".into() },
            RuntimeMapV1Edge { from: "map:1:1:1".into(), to: "map:1:2:0".into() },
        ],
        position: RuntimeMapV1Position::Current { node_id: "map:1:1:0".into() },
        history: vec!["map:1:0:0".into()],
        terminal_node_ids: vec!["map:1:2:0".into()],
        bindings: vec![
            RuntimeMapV1ActionBinding { graph_node_id: "map:1:1:1".into(), host_action_id: "select-map-node:42:map:1:1:1".into(), action: RuntimeMapV1NavigationAction::SelectMapNode { node_id: "map:1:1:1".into() } },
            RuntimeMapV1ActionBinding { graph_node_id: "map:1:1:0".into(), host_action_id: "select-map-node:42:map:1:1:0".into(), action: RuntimeMapV1NavigationAction::SelectMapNode { node_id: "map:1:1:0".into() } },
        ],
    }
}

#[test]
fn print_map_fixture() {
    let snapshot = snapshot();
    let context = RuntimeMapV1Context::new("corr-42", "instance-1", "session-1", "lease-1", 7);
    let request = RuntimeMapV1Message::snapshot_request(context.clone(), 42);
    let response = RuntimeMapV1Message::snapshot_response(context, snapshot, Some(RuntimeMapV1Timeout { timeout_millis: 1000, elapsed_millis: 12 }));
    println!("REQUEST={}", canonical_map_message_json(&request).unwrap());
    println!("RESPONSE={}", canonical_map_message_json(&response).unwrap());
}
