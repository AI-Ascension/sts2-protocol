// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn action_matches_transition(value: &Value, transition: &Value) -> Option<bool> {
    let action_payload = object(value.get("action")?, "action")?;
    let action_kind = string(action_payload, "kind")?;
    let transition_kind = string(transition, "kind")?;
    let option = string(transition, "rest_option_id")?;
    if option_witness_kind(option).is_none()
        || string(action_payload, "rest_option_id") != Some(option)
    {
        return Some(false);
    }
    Some(match transition_kind {
        "rest_option_selection_requested" => {
            action_kind == "rest_option" && selector_kind_for_option(option).is_some()
        }
        "rest_option_selection_progressed" => {
            action_kind
                == match selector_kind_for_option(option) {
                    Some("card") => "select_card",
                    Some("player") => "select_player",
                    _ => return Some(false),
                }
                && string(action_payload, "selection_id") == string(transition, "selection_id")
        }
        "rest_option_selection_completed" => {
            action_kind
                == match string(transition, "selection_kind") {
                    Some("card") => "confirm_selection",
                    Some("player") if action_kind == "select_player" => "select_player",
                    Some("player") => "confirm_selection",
                    _ => return Some(false),
                }
                && string(action_payload, "selection_id") == string(transition, "selection_id")
                && selector_kind_for_option(option) == string(transition, "selection_kind")
        }
        "rest_option_completed" => {
            action_kind == "rest_option" && selector_kind_for_option(option).is_none()
        }
        _ => false,
    })
}

fn selector_is_consistent(
    value: &Value,
    transition: &Value,
    selector: &Value,
    admissions: &BTreeMap<String, SelectorAdmission>,
) -> Option<bool> {
    let option_id = string(transition, "rest_option_id")?;
    let selection_id = string(selector, "selection_id")?;
    let selection_kind = string(selector, "selection_kind")?;
    if selector_kind_for_option(option_id) != Some(selection_kind) {
        return Some(false);
    }
    let required = u64_value(selector, "required_count")?;
    let selected = ids(selector, "selected_choice_ids")?;
    let selected_set: BTreeSet<_> = selected.iter().cloned().collect();
    let remaining = u64_value(selector, "remaining_count")?;
    if selected.len() as u64 != selected_set.len() as u64
        || selected.len() as u64 > required
        || remaining != required.saturating_sub(selected.len() as u64)
    {
        return Some(false);
    }
    let admission = admissions.get(selection_id)?;
    if admission.option_id != option_id
        || admission.selection_kind != selection_kind
        || admission.required_count != required
        || selected.iter().any(|id| !admission.choice_ids.contains(id))
    {
        return Some(false);
    }
    let visible = visible_choice_ids(value)?;
    let legal_actions = selector.get("legal_actions")?.as_array()?;
    if legal_actions.is_empty() {
        return Some(false);
    }
    let mut action_ids = BTreeSet::new();
    let mut confirms = 0;
    let mut cancels = 0;
    let mut has_choice = false;
    for legal in legal_actions {
        let action_id = string(legal, "action_id")?;
        if !action_ids.insert(action_id) {
            return Some(false);
        }
        let payload = object(legal, "action")?;
        let kind = string(payload, "kind")?;
        if string(payload, "selection_id") != Some(selection_id)
            || string(payload, "rest_option_id") != Some(option_id)
        {
            return Some(false);
        }
        match kind {
            "confirm_selection" => confirms += 1,
            "cancel_selection" => cancels += 1,
            "select_card" => {
                let id = string(payload, "card_id")?;
                if selection_kind != "card"
                    || !visible.contains(id)
                    || selected_set.contains(id)
                    || !admission.choice_ids.contains(id)
                {
                    return Some(false);
                }
                has_choice = true;
            }
            "select_player" => {
                let id = string(payload, "player_id")?;
                if selection_kind != "player"
                    || !visible.contains(id)
                    || selected_set.contains(id)
                    || !admission.choice_ids.contains(id)
                {
                    return Some(false);
                }
                has_choice = true;
            }
            _ => return Some(false),
        }
    }
    if cancels != 1
        || (remaining == 0 && confirms != 1)
        || (remaining > 0 && (confirms != 0 || !has_choice))
    {
        return Some(false);
    }
    if string(transition, "kind") == Some("rest_option_selection_progressed") {
        for field in [
            "selection_id",
            "selection_kind",
            "required_count",
            "selected_choice_ids",
            "remaining_count",
        ] {
            if transition.get(field) != selector.get(field) {
                return Some(false);
            }
        }
        let action = object(value.get("action")?, "action")?;
        let choice = match string(action, "kind") {
            Some("select_card") => string(action, "card_id"),
            Some("select_player") => string(action, "player_id"),
            _ => None,
        }?;
        if !selected_set.contains(choice) || !visible.contains(choice) {
            return Some(false);
        }
    }
    Some(true)
}

fn nonempty_array(value: &Value, field: &str) -> Option<bool> {
    Some(!value.get(field)?.as_array()?.is_empty())
}

fn hp_evidence_is_effectful(evidence: &Value) -> Option<bool> {
    let before = u64_value(evidence, "hp_before")?;
    let after = u64_value(evidence, "hp_after")?;
    let max_before = u64_value(evidence, "max_hp_before")?;
    let max_after = u64_value(evidence, "max_hp_after")?;
    Some(
        string(evidence, "kind") == Some("hp_change")
            && before <= max_before
            && after <= max_after
            && after > before,
    )
}

fn native_evidence_matches(value: &Value, evidence: &Value) -> Option<bool> {
    Some(
        string(evidence, "kind") == Some("native_completion")
            && string(evidence, "completion_id").is_some()
            && string(evidence, "native_state_id")
                == value
                    .get("observation")
                    .and_then(|observation| string(observation, "state_id")),
    )
}

fn stat_evidence_is_effectful(evidence: &Value) -> Option<bool> {
    Some(
        string(evidence, "kind") == Some("stat_change")
            && string(evidence, "stat_id").is_some()
            && evidence.get("before")?.as_i64()? != evidence.get("after")?.as_i64()?,
    )
}

fn effect_matches_option(value: &Value, transition: &Value) -> Option<bool> {
    let option = string(transition, "rest_option_id")?;
    let expected_kind = option_witness_kind(option)?;
    let root = object(value, "effect_witness")?;
    let nested = object(transition, "effect_witness")?;
    if root != nested
        || string(root, "version") != Some(RUNTIME_V4_EXPERT_REST_ACTION_EFFECT_WITNESS_VERSION)
        || string(root, "kind") != Some(expected_kind)
        || string(root, "operation_id") != string(value, "operation_id")
        || string(root, "rest_option_id") != Some(option)
        || u64_value(root, "generation") != u64_value(value, "generation")
        || (option == "mend") != root.get("target_player_id").is_some()
    {
        return Some(false);
    }
    let evidence = object(root, "evidence")?;
    Some(match option {
        "clone" => {
            string(evidence, "kind") == Some("card_change")
                && nonempty_array(evidence, "added_card_ids")?
        }
        "cook" => {
            string(evidence, "kind") == Some("card_change")
                && nonempty_array(evidence, "removed_card_ids")?
        }
        "dig" | "hatch" => {
            string(evidence, "kind") == Some("relic_change")
                && nonempty_array(evidence, "added_relic_ids")?
        }
        "heal" => hp_evidence_is_effectful(evidence)?,
        "kindle" => native_evidence_matches(value, evidence)?,
        "lift" => {
            stat_evidence_is_effectful(evidence)? || native_evidence_matches(value, evidence)?
        }
        "smith" => {
            string(evidence, "kind") == Some("card_change")
                && nonempty_array(evidence, "upgraded_card_ids")?
                && evidence.get("upgraded_card_ids") == transition.get("selected_choice_ids")
        }
        "mend" => {
            let selected = transition.get("selected_choice_ids")?.as_array()?;
            root.get("target_player_id") == selected.first()
                && selected.len() == 1
                && (hp_evidence_is_effectful(evidence)?
                    || native_evidence_matches(value, evidence)?)
        }
        _ => false,
    })
}

fn completion_matches_admission(
    transition: &Value,
    admissions: &BTreeMap<String, SelectorAdmission>,
) -> Option<bool> {
    let selection_id = string(transition, "selection_id")?;
    let admission = match admissions.get(selection_id) {
        Some(admission) => admission,
        None => return Some(false),
    };
    let selected = ids(transition, "selected_choice_ids")?;
    Some(
        string(transition, "rest_option_id") == Some(admission.option_id.as_str())
            && string(transition, "selection_kind") == Some(admission.selection_kind.as_str())
            && u64_value(transition, "required_count") == Some(admission.required_count)
            && selected.len() as u64 == admission.required_count
            && selected.iter().all(|id| admission.choice_ids.contains(id)),
    )
}

pub(crate) fn strict_semantics(
    value: &Value,
    expected_actions: &BTreeMap<String, Value>,
    admissions: &BTreeMap<String, SelectorAdmission>,
) -> Option<bool> {
    let operation_id = string(value, "operation_id")?;
    if string(value, "protocol_version") != Some(RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION)
        || string(value, "profile") != Some(RUNTIME_V4_EXPERT_REST_ACTION_PROFILE)
        || string(value, "schema_digest") != Some(RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST)
        || value.get("provenance")
            != Some(&serde_json::json!({
                "artifact": RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT,
                "source": RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_SOURCE,
                "generator": RUNTIME_V4_EXPERT_REST_ACTION_GENERATOR
            }))
    {
        return Some(false);
    }
    if let Some(expected) = expected_actions.get(operation_id)
        && value.get("action") != Some(expected)
    {
        return Some(false);
    }
    if string(value, "kind") != Some("action_response")
        || string(value, "status") != Some("settled")
    {
        return Some(true);
    }
    let observation = value.get("observation")?;
    if observation.get("state_id") != value.get("state_id")
        || observation.get("generation") != value.get("generation")
        || string(observation, "schema_digest") != Some(RUNTIME_V4_EXPERT_SCHEMA_DIGEST)
    {
        return Some(false);
    }
    let transition = value.get("transition")?;
    let before = u64_value(transition, "before_generation")?;
    let after = u64_value(transition, "after_generation")?;
    if before >= after
        || Some(after) != u64_value(value, "generation")
        || !action_matches_transition(value, transition).is_some_and(|matches| matches)
    {
        return Some(false);
    }
    Some(match string(transition, "kind")? {
        "rest_option_selection_requested" | "rest_option_selection_progressed" => {
            value.get("effect_witness") == Some(&Value::Null)
                && transition.get("effect_witness") == Some(&Value::Null)
                && selector_is_consistent(
                    value,
                    transition,
                    transition.get("selector")?,
                    admissions,
                )?
        }
        "rest_option_selection_completed" => {
            let selected = ids(transition, "selected_choice_ids")?;
            let required = u64_value(transition, "required_count")?;
            let unique = selected.iter().collect::<BTreeSet<_>>().len() == selected.len();
            u64_value(transition, "remaining_count")? == 0
                && selected.len() as u64 == required
                && unique
                && completion_matches_admission(transition, admissions)?
                && effect_matches_option(value, transition)?
        }
        "rest_option_completed" => effect_matches_option(value, transition)?,
        _ => false,
    })
}
