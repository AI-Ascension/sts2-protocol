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
    assert!(message.mutation_allowed());
    message.synchronization.status = CoopSyncStatus::Disagreement;
    assert!(message.validate().is_err());
    assert!(!message.mutation_allowed());
}

#[test]
fn co_op_missing_peers_are_unique_known_members_and_suspend_synchronized_mutation() {
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
    assert!(!message.mutation_allowed());
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
