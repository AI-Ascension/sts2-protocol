// SPDX-License-Identifier: MIT

use super::*;
use std::collections::BTreeSet;

impl Validator {
    pub(super) fn validate_context(&self, context: &ValidationContext) -> Result {
        self.fragment("capabilities", &context.capabilities)?;
        let authority = &context.authority;
        for id in [
            &authority.producer_lease,
            &authority.instance_id,
            &authority.run_id,
        ] {
            self.fragment("identity", &Value::String(id.clone()))?;
        }
        require(
            authority.epoch <= 9_007_199_254_740_991,
            Rejection::InvalidIdentity,
        )?;
        let caps = &context.capabilities;
        query::limits(&caps["limits"], &caps["limits"])?;
        require(
            caps["snapshot_policy"]["lifetime_generations"] == 1
                && caps["snapshot_policy"]["max_retained_snapshots"] == 1,
            Rejection::Malformed,
        )?;
        require(context.cursors.len() <= 64, Rejection::ResultLimitExceeded)?;
        let mut cursors = BTreeSet::new();
        for cursor in &context.cursors {
            self.fragment("cursor", &Value::String(cursor.cursor.clone()))?;
            require(cursors.insert(&cursor.cursor), Rejection::StaleCursor)?;
        }
        let available = caps["rest_context"]["availability"] == "available";
        if available {
            let read = &caps["rest_context"]["value"];
            self.read_context(read, authority)?;
            require(
                context
                    .rest
                    .as_ref()
                    .is_some_and(|capture| capture.context == *read),
                Rejection::StaleSnapshot,
            )?;
        }
        if let Some(capture) = &context.rest {
            self.fragment("identity", &Value::String(capture.revision.clone()))?;
            self.fragment(
                "identity",
                &Value::String(capture.content_manifest_id.clone()),
            )?;
            self.fragment("locale", &Value::String(capture.locale.clone()))?;
            if !capture.context.is_null() {
                self.read_context(&capture.context, authority)?;
                require(
                    capture.context["binding"]["content_manifest_id"]
                        == capture.content_manifest_id
                        && capture.context["binding"]["locale"] == capture.locale,
                    Rejection::StaleSnapshot,
                )?;
            } else {
                require(capture.legal_actions.is_empty(), Rejection::InvalidIdentity)?;
            }
            self.entries(capture)?;
        }
        Ok(())
    }

    fn read_context(&self, read: &Value, authority: &Authority) -> Result {
        self.fragment("rest_read_context", read)?;
        let binding = &read["binding"];
        require(
            fence(&binding["instance_ref"], authority),
            Rejection::DeniedScope,
        )?;
        require(
            binding["instance_ref"] == binding["snapshot_ref"]["instance_ref"]
                && read["parent_observation"]["instance_ref"] == binding["instance_ref"]
                && read["parent_observation"]["snapshot_ref"] == binding["snapshot_ref"]
                && read["parent_observation"]["state_generation"]
                    == binding["snapshot_ref"]["state_generation"],
            Rejection::MixedGeneration,
        )
    }

    fn entries(&self, capture: &RestCapture) -> Result {
        require(capture.entries.len() <= 9, Rejection::ResultLimitExceeded)?;
        require(
            capture.legal_actions.len() <= 9 && capture.sources.len() <= 16,
            Rejection::ResultLimitExceeded,
        )?;
        let mut action_ids = BTreeSet::new();
        for action in &capture.legal_actions {
            self.fragment("rest_action_ref", action)?;
            require(
                action_ids.insert(action["action_id"].to_string()),
                Rejection::AmbiguousId,
            )?;
        }
        for source in &capture.sources {
            self.fragment("source", source)?;
        }
        let mut definitions = BTreeSet::new();
        let mut occurrences = BTreeSet::new();
        let mut options = BTreeSet::new();
        for entry in &capture.entries {
            let mode = option_mode(&entry.option_id)?;
            let item = serde_json::json!({
                "definition_ref": entry.definition_ref, "instance_ref": entry.instance_ref,
                "fields": entry.fields,
            });
            self.fragment("item", &item)?;
            require(
                entry.definition_ref["entity_kind"] == "rest_option",
                Rejection::InvalidIdentity,
            )?;
            require(
                definitions.insert(entry.definition_ref.to_string())
                    && options.insert(&entry.option_id),
                Rejection::AmbiguousId,
            )?;
            require(
                entry.definition_ref["content_manifest_id"] == capture.content_manifest_id,
                Rejection::InvalidIdentity,
            )?;
            if capture.context.is_null() {
                require(
                    entry.instance_ref.is_null()
                        && entry.fields.iter().all(|field| {
                            !matches!(
                                field["name"].as_str(),
                                Some("rest_action" | "rest_eligible" | "rest_selector")
                            ) || field["availability"] != "available"
                        }),
                    Rejection::InvalidIdentity,
                )?;
            } else {
                let room = &capture.context["binding"]["instance_ref"];
                require(
                    entry.instance_ref["entity_kind"] == "rest_option"
                        && occurrences.insert(entry.instance_ref.to_string())
                        && ["instance_id", "run_id", "epoch"]
                            .iter()
                            .all(|key| entry.instance_ref[key] == room[key])
                        && entry.instance_ref["entity_id"] != room["entity_id"],
                    Rejection::InvalidIdentity,
                )?;
            }
            require(
                entry.fields.len() == LIVE_FIELDS.len(),
                Rejection::UnsupportedField,
            )?;
            for (field, expected) in entry.fields.iter().zip(LIVE_FIELDS) {
                require(field["name"] == *expected, Rejection::UnsupportedField)?;
                require(
                    capture.sources.contains(&field["source"]),
                    Rejection::InvalidIdentity,
                )?;
                if field["name"] == "rest_action" && field["availability"] == "available" {
                    require(
                        capture.legal_actions.contains(&field["value"]),
                        Rejection::InvalidIdentity,
                    )?;
                }
            }
            self.entry_fields(entry, mode)?;
        }
        Ok(())
    }

    fn entry_fields(&self, entry: &RestEntry, mode: &str) -> Result {
        let fields: BTreeMap<_, _> = entry
            .fields
            .iter()
            .filter_map(|field| field["name"].as_str().map(|name| (name, field)))
            .collect();
        require(
            fields["rest_option_id"]["value"] == entry.option_id
                && fields["rest_mode"]["value"] == mode,
            Rejection::UnsupportedField,
        )?;
        let action = fields["rest_action"];
        if action["availability"] == "available" {
            require(
                action["value"]["rest_option_id"] == entry.option_id
                    && action["value"]["action_id"] != entry.instance_ref["entity_id"],
                Rejection::InvalidIdentity,
            )?;
        }
        let eligible = fields["rest_eligible"];
        require(
            if eligible["availability"] == "available" {
                eligible["value"].as_bool() == entry.eligibility && entry.eligibility.is_some()
            } else {
                entry.eligibility.is_none()
            },
            Rejection::UnsupportedField,
        )?;
        let selector = fields["rest_selector"];
        if selector["availability"] == "available" {
            let value = &selector["value"];
            require(
                (entry.option_id == "smith" && value["selection_kind"] == "card")
                    || (entry.option_id == "mend" && value["selection_kind"] == "player"),
                Rejection::Malformed,
            )?;
            require(
                number(&value["selected_count"])? + number(&value["remaining_count"])?
                    == number(&value["required_count"])?
                    && number(&value["selected_count"])? == entry.selected_choices.len(),
                Rejection::Malformed,
            )?;
            let mut choices = BTreeSet::new();
            for choice in &entry.selected_choices {
                self.fragment("identity", &Value::String(choice.clone()))?;
                require(choices.insert(choice), Rejection::Malformed)?;
            }
        } else {
            require(entry.selected_choices.is_empty(), Rejection::Malformed)?;
        }
        Ok(())
    }
}

fn option_mode(option: &str) -> Result<&'static str> {
    match option {
        "clone" | "cook" | "dig" | "hatch" | "heal" | "kindle" | "lift" => Ok("immediate"),
        "smith" => Ok("card_selection"),
        "mend" => Ok("player_selection"),
        _ => Err(Rejection::UnsupportedField),
    }
}
