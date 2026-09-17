// SPDX-License-Identifier: MIT

use serde_json::{Value, json};

fn ref_key(reference: &Value) -> Value {
    json!([
        reference["instance_id"],
        reference["run_id"],
        reference["epoch"],
        reference["entity_kind"],
        reference["entity_id"]
    ])
}

fn limits_within(response: &Value, request: &Value) -> bool {
    [
        "max_visible_entities",
        "max_item_bytes",
        "max_message_bytes",
    ]
    .into_iter()
    .all(|field| {
        response["limits"][field].as_u64().unwrap() <= request["limits"][field].as_u64().unwrap()
    })
}

pub(super) fn response_request_error(response: &Value, request: &Value) -> Option<&'static str> {
    for field in ["correlation_id", "scope", "selector"] {
        if response[field] != request[field] {
            return Some("invalid_binding");
        }
    }
    if response["kind"] == "error_response" {
        return None;
    }
    if !limits_within(response, request) {
        return Some("invalid_bounds");
    }
    semantic_error(response)
}

pub(super) fn semantic_error(document: &Value) -> Option<&'static str> {
    if document["kind"] == "error_response" {
        return None;
    }
    if document["kind"] != "bootstrap_response" {
        return None;
    }
    let parent = &document["parent_observation"];
    if parent["instance_ref"] != parent["snapshot_ref"]["instance_ref"]
        || parent["state_generation"] != parent["snapshot_ref"]["state_generation"]
    {
        return Some("invalid_binding");
    }
    let visible = document["visible_entities"].as_array()?;
    let limit = document["limits"]["max_visible_entities"].as_u64()?;
    if visible.len() as u64 > limit {
        return Some("invalid_bounds");
    }
    let parent_ref = &parent["instance_ref"];
    let scope = &document["scope"];
    if scope["instance_id"] != parent_ref["instance_id"]
        || scope["run_id"] != parent_ref["run_id"]
        || scope["content_manifest_id"]
            != document["selector"]["definition_ref"]["content_manifest_id"]
    {
        return Some("invalid_binding");
    }
    if document["owner_provenance"]
        != json!({
            "authority_epoch_owner": "sts2-harness",
            "content_manifest_owner": "sts2-game-mod",
            "instance_fence_owner": "sts2-gateway",
            "instance_ref_epoch_owner": "sts2-game-mod",
            "native_snapshot_owner": "sts2-game-mod",
            "transport_lease_epoch_role": "fence_only"
        })
    {
        return Some("invalid_binding");
    }
    let definition = &document["selector"]["definition_ref"];
    let selector_ref = &document["selector"]["instance_ref"];
    let mut keys = Vec::with_capacity(visible.len());
    for entity in visible {
        let reference = &entity["instance_ref"];
        let entity_snapshot = &entity["snapshot_ref"];
        for field in ["instance_id", "run_id", "epoch"] {
            if reference[field] != parent_ref[field] {
                return Some("invalid_binding");
            }
        }
        if entity_snapshot["instance_ref"] != *reference
            || entity_snapshot["snapshot_id"] != parent["snapshot_ref"]["snapshot_id"]
        {
            return Some("invalid_binding");
        }
        if entity_snapshot["state_generation"] != parent["state_generation"] {
            return Some("stale_snapshot");
        }
        if entity["definition_ref"].is_object() && entity["definition_ref"] != *definition {
            return Some("invalid_binding");
        }
        let key = ref_key(reference);
        if keys.iter().any(|existing| existing == &key) {
            return Some("invalid_binding");
        }
        keys.push(key);
    }
    if !visible
        .iter()
        .any(|entity| entity["instance_ref"] == *parent_ref)
    {
        return Some("invalid_binding");
    }
    if !selector_ref.is_null()
        && (selector_ref != parent_ref
            || visible
                .iter()
                .filter(|entity| entity["instance_ref"] == *selector_ref)
                .count()
                != 1)
    {
        return Some("invalid_binding");
    }
    let max_item_bytes = document["limits"]["max_item_bytes"].as_u64()?;
    for entity in visible {
        if serde_json::to_vec(entity).ok()?.len() as u64 > max_item_bytes {
            return Some("invalid_bounds");
        }
    }
    if serde_json::to_vec(document).ok()?.len() as u64
        > document["limits"]["max_message_bytes"].as_u64()?
    {
        return Some("invalid_bounds");
    }
    None
}
