// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    COOP_GAMEPLAY_ARTIFACT, COOP_GAMEPLAY_GENERATOR, COOP_GAMEPLAY_PROTOCOL_VERSION,
    COOP_GAMEPLAY_SCHEMA_DIGEST, COOP_GAMEPLAY_SCHEMA_SOURCE, CoopGameplayMessage,
    CoopGameplayMessageKind, CoopLocalAction, CoopLocalActionKind, CoopPeerRole, CoopPlayer,
    CoopProvenance, CoopSyncStatus, CoopSynchronization,
};

const SCHEMA: &str = include_str!("../../../schemas/coop-gameplay-v1.schema.json");

fn base(kind: CoopGameplayMessageKind) -> CoopGameplayMessage {
    CoopGameplayMessage {
        protocol_version: COOP_GAMEPLAY_PROTOCOL_VERSION.to_owned(),
        schema_digest: COOP_GAMEPLAY_SCHEMA_DIGEST.to_owned(),
        provenance: CoopProvenance {
            artifact: COOP_GAMEPLAY_ARTIFACT.to_owned(),
            source: COOP_GAMEPLAY_SCHEMA_SOURCE.to_owned(),
            generator: COOP_GAMEPLAY_GENERATOR.to_owned(),
        },
        correlation_id: "corr-1".to_owned(),
        instance_id: "instance-1".to_owned(),
        session_id: "session-1".to_owned(),
        lease_id: "lease-1".to_owned(),
        lease_epoch: 1,
        generation: 4,
        kind,
        players: vec![
            CoopPlayer {
                peer_id: "local-1".to_owned(),
                role: CoopPeerRole::Local,
            },
            CoopPlayer {
                peer_id: "ally-1".to_owned(),
                role: CoopPeerRole::Ally,
            },
        ],
        local_action: None,
        shared_vote: None,
        shared_effect: None,
        ally_target: None,
        synchronization: CoopSynchronization {
            status: CoopSyncStatus::Synchronized,
            generation: 4,
            peer_count: 2,
            missing_peers: Vec::new(),
        },
    }
}

#[test]
fn co_op_contract_separates_local_action_and_peer_synchronization() {
    let mut message = base(CoopGameplayMessageKind::LocalActionRequest);
    message.local_action = Some(CoopLocalAction {
        action_id: "combat.play-card".to_owned(),
        kind: CoopLocalActionKind::PlayCard,
    });
    assert!(message.validate().is_ok());
    assert!(message.is_synchronized_action_request());
    message.synchronization.status = CoopSyncStatus::Disagreement;
    assert!(message.validate().is_err());
    assert!(!message.is_synchronized_action_request());
}

#[test]
fn co_op_missing_peers_are_unique_known_members_and_invalidate_synchronized_requests() {
    let mut message = base(CoopGameplayMessageKind::SynchronizationResponse);
    message.synchronization.missing_peers = vec!["ally-1".to_owned(), "ally-1".to_owned()];
    assert!(message.validate().is_err());

    let mut message = base(CoopGameplayMessageKind::SynchronizationResponse);
    message.synchronization.missing_peers = vec!["unknown-1".to_owned()];
    assert!(message.validate().is_err());

    let mut message = base(CoopGameplayMessageKind::LocalActionRequest);
    message.local_action = Some(CoopLocalAction {
        action_id: "combat.end-turn".to_owned(),
        kind: CoopLocalActionKind::EndTurn,
    });
    message.synchronization.missing_peers = vec!["ally-1".to_owned()];
    assert!(message.validate().is_err());
    assert!(!message.is_synchronized_action_request());
}

#[test]
fn co_op_schema_rejects_unknown_fields_and_accepts_synchronization_response() {
    let mut message = base(CoopGameplayMessageKind::SynchronizationResponse);
    let value: Value = serde_json::to_value(&message).expect("message encodes");
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    assert!(validator.is_valid(&value));
    message.synchronization.status = CoopSyncStatus::Disconnected;
    let value = serde_json::to_value(message).expect("message encodes");
    assert!(validator.is_valid(&value));
    let mut unknown = value;
    unknown["raw_memory"] = json!("blocked");
    assert!(!validator.is_valid(&unknown));
}

#[test]
fn request_shape_predicate_rejects_invalid_metadata_shape_and_peer_identity() {
    let mut message = base(CoopGameplayMessageKind::LocalActionRequest);
    assert!(
        !message.is_synchronized_action_request(),
        "missing action is not a synchronized action request"
    );
    message.local_action = Some(CoopLocalAction {
        action_id: "combat.end-turn".to_owned(),
        kind: CoopLocalActionKind::EndTurn,
    });
    assert!(message.is_synchronized_action_request());
    let mut invalid = message.clone();
    invalid.schema_digest = "0".repeat(64);
    assert!(!invalid.is_synchronized_action_request());
    let mut invalid = message.clone();
    invalid.players[1].peer_id = invalid.players[0].peer_id.clone();
    assert!(!invalid.is_synchronized_action_request());
    let mut invalid = message.clone();
    invalid.synchronization.generation += 1;
    assert!(!invalid.is_synchronized_action_request());
    let mut invalid = message;
    invalid.ally_target = Some("unknown-peer".to_owned());
    assert!(!invalid.is_synchronized_action_request());
}

#[test]
fn nullable_fields_are_required_on_wire() {
    let message = base(CoopGameplayMessageKind::Observation);
    let value = serde_json::to_value(message).expect("message encodes");
    assert!(serde_json::from_value::<CoopGameplayMessage>(value.clone()).is_ok());
    for field in [
        "local_action",
        "shared_vote",
        "shared_effect",
        "ally_target",
    ] {
        let mut missing = value.clone();
        missing.as_object_mut().expect("object").remove(field);
        assert!(
            serde_json::from_value::<CoopGameplayMessage>(missing).is_err(),
            "{field}"
        );
    }
}

#[test]
fn schema_enforces_kind_payloads_and_peer_structure() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    let value = serde_json::to_value(base(CoopGameplayMessageKind::Observation)).unwrap();
    let mut invalid = Vec::new();
    for kind in [
        "local_action_request",
        "shared_vote_request",
        "shared_effect_response",
        "recovery_required",
    ] {
        let mut candidate = value.clone();
        candidate["kind"] = json!(kind);
        invalid.push(candidate);
    }
    let mut candidate = value.clone();
    candidate["players"] = json!([]);
    invalid.push(candidate);
    let mut candidate = value.clone();
    candidate["players"][1] = candidate["players"][0].clone();
    invalid.push(candidate);
    let mut candidate = value.clone();
    candidate["players"][0]["role"] = json!("ally");
    invalid.push(candidate);
    let mut candidate = value.clone();
    candidate["synchronization"]["missing_peers"] = json!(["ally-1"]);
    invalid.push(candidate);
    let mut candidate = value.clone();
    candidate["synchronization"]["status"] = json!("disconnected");
    candidate["synchronization"]["missing_peers"] = json!(["ally-1", "ally-1"]);
    invalid.push(candidate);
    let mut candidate = value;
    candidate["local_action"] = json!({"action_id": "action-1", "kind": "end_turn"});
    invalid.push(candidate);
    for candidate in invalid {
        assert!(!validator.is_valid(&candidate), "{candidate}");
        assert!(
            serde_json::from_value::<CoopGameplayMessage>(candidate)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}

#[test]
fn all_six_message_kinds_have_schema_and_semantic_positive_cases() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles");
    for kind in [
        "observation",
        "synchronization_response",
        "local_action_request",
        "shared_vote_request",
        "shared_effect_response",
        "recovery_required",
    ] {
        let mut value = serde_json::to_value(base(CoopGameplayMessageKind::Observation)).unwrap();
        value["kind"] = json!(kind);
        match kind {
            "local_action_request" => {
                value["local_action"] = json!({"action_id": "action-1", "kind": "end_turn"});
                value["ally_target"] = json!("ally-1");
            }
            "shared_vote_request" => {
                value["shared_vote"] = json!({
                    "vote_id": "vote-1", "proposal_id": "proposal-1", "voter_id": "ally-1", "choice": "approve"
                })
            }
            "shared_effect_response" => {
                value["shared_effect"] = json!({
                    "effect_id": "effect-1", "kind": "transition", "from_generation": 3, "to_generation": 4
                })
            }
            "recovery_required" => {
                value["synchronization"]["status"] = json!("disconnected");
                value["synchronization"]["missing_peers"] = json!(["ally-1"]);
            }
            _ => {}
        }
        assert!(validator.is_valid(&value), "{kind}");
        let decoded: CoopGameplayMessage = serde_json::from_value(value).unwrap();
        assert!(decoded.validate().is_ok(), "{kind}");
    }
}
