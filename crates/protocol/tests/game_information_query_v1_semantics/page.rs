// SPDX-License-Identifier: MIT

use super::super::*;
use super::core::*;

fn page_error(page: &Value, query: &Value, context: &SemanticContext) -> Option<&'static str> {
    let Some(object) = page.as_object() else {
        return Some(MALFORMED);
    };
    let Some(items) = object.get("items").and_then(Value::as_array) else {
        return Some(MALFORMED);
    };
    if !valid_limit_object(&object["limits"]) || !object["accounting"].is_object() {
        return Some(MALFORMED);
    }
    if let Some(error) = limit_error(&object["limits"], &context.capabilities) {
        return Some(error);
    }
    let max_item_bytes = items.iter().map(canonical_bytes).max().unwrap_or(0);
    let payload_bytes = canonical_bytes(&object["items"]);
    let text_bytes = text_bytes(&object["items"]);
    let mut page_without_accounting = object.clone();
    page_without_accounting.remove("accounting");
    let page_bytes = canonical_bytes(&Value::Object(page_without_accounting));
    if items.len() > object["limits"]["page_items"].as_u64().unwrap_or(0) as usize
        || max_item_bytes > object["limits"]["item_bytes"].as_u64().unwrap_or(0) as usize
        || page_bytes > object["limits"]["page_bytes"].as_u64().unwrap_or(0) as usize
        || text_bytes > object["limits"]["text_bytes"].as_u64().unwrap_or(0) as usize
    {
        return Some(RESULT_LIMIT_EXCEEDED);
    }
    if object["limits"] != query["limits"] {
        return Some(MALFORMED);
    }
    if object["accounting"]["item_count"].as_u64() != Some(items.len() as u64)
        || object["accounting"]["item_bytes"].as_u64() != Some(max_item_bytes as u64)
        || object["accounting"]["payload_bytes"].as_u64() != Some(payload_bytes as u64)
        || object["accounting"]["page_bytes"].as_u64() != Some(page_bytes as u64)
        || object["accounting"]["text_bytes"].as_u64() != Some(text_bytes as u64)
    {
        return Some(MALFORMED);
    }
    match (
        object["final_page"].as_bool(),
        object["next_cursor"].is_null(),
    ) {
        (Some(true), true) => {}
        (Some(false), false) if !object["cursor_binding"].is_null() => {}
        _ => return Some(MALFORMED),
    }
    match object["total_count_known"].as_bool() {
        Some(true) if object["total_count"].as_u64().is_some() => {}
        Some(false) if object["total_count"].is_null() => {}
        _ => return Some(MALFORMED),
    }
    if matches!(
        object["coverage"].as_str(),
        Some("unavailable" | "not_observable")
    ) && (items.is_empty()
        && object["next_cursor"].is_null()
        && object["cursor_binding"].is_null()
        && object["final_page"] == true
        && object["total_count_known"] == false
        && object["total_count"].is_null()
        && object["accounting"]["item_count"] == 0
        && object["accounting"]["item_bytes"] == 0
        && object["accounting"]["payload_bytes"] == 0
        && object["accounting"]["text_bytes"] == 0)
    {
        // Empty unavailable pages satisfy the conditional page contract.
    } else if matches!(
        object["coverage"].as_str(),
        Some("unavailable" | "not_observable")
    ) {
        return Some(MALFORMED);
    }
    if let Some(cursor_binding) = object["cursor_binding"].as_object()
        && !values_equal(
            &Value::Object(cursor_binding.clone()),
            &binding_value(query),
        )
    {
        return Some(STALE_CURSOR);
    }
    if let Some(cursor) = object["next_cursor"].as_str() {
        if !context
            .cursor_bindings
            .iter()
            .any(|(known, _)| known == cursor)
        {
            return Some(STALE_CURSOR);
        }
    } else if !object["next_cursor"].is_null() {
        return Some(MALFORMED);
    }
    for item in items {
        if !is_definition_ref(&item["definition_ref"])
            || item["definition_ref"]["content_manifest_id"]
                != query["binding"]["content_manifest_id"]
        {
            return Some(MALFORMED);
        }
        if query["binding"]["mode"] == "static" {
            if !item["instance_ref"].is_null() {
                return Some(MALFORMED);
            }
        } else if !values_equal(&item["instance_ref"], &query["binding"]["instance_ref"]) {
            return Some(MALFORMED);
        }
        let Some(fields) = item["fields"].as_array() else {
            return Some(MALFORMED);
        };
        let mut previous = "";
        for field in fields {
            let Some(name) = field["name"].as_str() else {
                return Some(MALFORMED);
            };
            if name <= previous {
                return Some(MALFORMED);
            }
            previous = name;
            if let Some(error) = field_error(field) {
                return Some(error);
            }
        }
    }
    None
}

fn response_error(value: &Value, context: &SemanticContext) -> Option<&'static str> {
    if let Some(error) = query_error(&value["query"], context) {
        return Some(error);
    }
    if !value["result"].is_object() {
        return Some(MALFORMED);
    }
    if value["result"]["read_only"] == false {
        return Some(READ_ONLY_VIOLATION);
    }
    if value["result"]["read_only"] != true {
        return Some(MALFORMED);
    }
    if value["query"]["binding"]["mode"] == "live" {
        if !safe_integer(&value["result"]["result_generation"])
            || !value["result"]["parent_observation"].is_object()
        {
            return Some(MALFORMED);
        }
        if value["result"]["result_generation"]
            != value["query"]["binding"]["snapshot_ref"]["state_generation"]
        {
            return Some(MIXED_GENERATION);
        }
        if value["result"]["parent_observation"] != value["query"]["parent_observation"] {
            return Some(MIXED_GENERATION);
        }
    } else if !value["result"]["result_generation"].is_null()
        || !value["result"]["parent_observation"].is_null()
    {
        return Some(MALFORMED);
    }
    page_error(&value["result"]["page"], &value["query"], context)
}

fn known_error_code(code: &str) -> Option<&'static str> {
    Some(match code {
        "unknown_kind" => "unknown_kind",
        "unknown_id" => "unknown_id",
        "ambiguous_id" => "ambiguous_id",
        "unsupported_filter" => "unsupported_filter",
        "unsupported_projection" => "unsupported_projection",
        "unsupported_version" => "unsupported_version",
        "denied_scope" => "denied_scope",
        "stale_snapshot" => "stale_snapshot",
        "stale_cursor" => "stale_cursor",
        "result_limit_exceeded" => "result_limit_exceeded",
        "missing_capability" => "missing_capability",
        "unsupported_field" => "unsupported_field",
        "invalid_identity" => "invalid_identity",
        "invalid_bounds" => "invalid_bounds",
        "mixed_generation" => "mixed_generation",
        "live_fence_mismatch" => "live_fence_mismatch",
        "availability_mismatch" => "availability_mismatch",
        "field_value_mismatch" => "field_value_mismatch",
        "page_state_mismatch" => "page_state_mismatch",
        "invalid_accounting" => "invalid_accounting",
        "cross_epoch_cursor" => "cross_epoch_cursor",
        "duplicate_key" => "duplicate_key",
        "malformed" => "malformed",
        "read_only_violation" => "read_only_violation",
        _ => return None,
    })
}

pub(crate) fn semantic_rejection(value: &Value, context_name: &str) -> Option<&'static str> {
    let context = context_for(context_name);
    if !value.is_object() {
        return Some(MALFORMED);
    }
    if value["protocol_version"] != PROFILE {
        return Some(UNSUPPORTED_VERSION);
    }
    if value["schema_digest"] != SCHEMA_DIGEST {
        return Some(MALFORMED);
    }
    if canonical_bytes(value)
        > context.capabilities["max_message_bytes"]
            .as_u64()
            .unwrap_or(0) as usize
    {
        return Some(RESULT_LIMIT_EXCEEDED);
    }
    match value["kind"].as_str() {
        Some("capabilities_response") => {
            if value["capabilities"]["profile"] == PROFILE {
                None
            } else {
                Some(MALFORMED)
            }
        }
        Some("error_response") => {
            let Some(code) = value["error"]["code"].as_str() else {
                return Some(MALFORMED);
            };
            let derived = if value["query"].is_null() {
                None
            } else {
                query_error(&value["query"], &context)
            };
            if matches!(derived, Some(MISSING_CAPABILITY | UNSUPPORTED_FIELD)) {
                let expected = derived.expect("capability or field error");
                return if code == expected {
                    Some(expected)
                } else {
                    Some(MALFORMED)
                };
            }
            if matches!(code, MISSING_CAPABILITY | UNSUPPORTED_FIELD) {
                return Some(MALFORMED);
            }
            known_error_code(code).or(Some(MALFORMED))
        }
        Some("query_request") => query_error(&value["query"], &context),
        Some("query_response") => response_error(value, &context),
        _ => Some(UNKNOWN_KIND),
    }
}

fn context_for(name: &str) -> SemanticContext {
    let capabilities = value("capabilities-response")["capabilities"].clone();
    let static_response = value("static-page-1-response");
    let live_request = value("live-detail-request");
    let mut context = SemanticContext {
        capabilities,
        cursor_bindings: vec![
            (
                static_response["result"]["page"]["next_cursor"]
                    .as_str()
                    .expect("static cursor")
                    .to_owned(),
                binding_value(&static_response["query"]),
            ),
            (
                "cursor:live:1".to_owned(),
                binding_value(&live_request["query"]),
            ),
        ],
    };
    match name {
        "no-live" => {
            context.capabilities["snapshot_policy"]["supports_live"] = Value::Bool(false);
        }
        "no-tags" => {
            context.capabilities["fields"] = json!([
                "amount",
                "cost",
                "description",
                "display_name",
                "flags",
                "owner",
                "position",
                "rarity",
                "source_id"
            ]);
        }
        "summary-only" => {
            context.capabilities["projections"] = json!(["summary"]);
        }
        "small-message" => {
            context.capabilities["max_message_bytes"] = json!(2560);
        }
        _ => {}
    }
    context
}
