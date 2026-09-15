// SPDX-License-Identifier: MIT

use super::*;

impl Validator {
    pub(super) fn query(&self, query: &Value, context: &ValidationContext) -> Result {
        let caps = &context.capabilities;
        for (key, plural, error) in [
            ("query_kind", "query_kinds", Rejection::MissingCapability),
            ("entity_kind", "entity_kinds", Rejection::MissingCapability),
            (
                "projection",
                "projections",
                Rejection::UnsupportedProjection,
            ),
            (
                "detail_level",
                "detail_levels",
                Rejection::UnsupportedProjection,
            ),
        ] {
            require(array(&caps[plural])?.contains(&query[key]), error)?;
        }
        for field in array(&query["fields"])? {
            require(
                array(&caps["fields"])?.contains(field),
                Rejection::UnsupportedField,
            )?;
        }
        limits(&query["limits"], &caps["limits"])?;
        if !query["cursor"].is_null() {
            self.cursor(&query["cursor"], query, context)?;
        }
        let binding = &query["binding"];
        if binding["mode"] == "live" {
            require(
                fence(&binding["instance_ref"], &context.authority),
                Rejection::DeniedScope,
            )?;
            require(
                binding["instance_ref"] == binding["snapshot_ref"]["instance_ref"]
                    && query["parent_observation"]["instance_ref"] == binding["instance_ref"]
                    && query["parent_observation"]["snapshot_ref"] == binding["snapshot_ref"]
                    && query["parent_observation"]["state_generation"]
                        == binding["snapshot_ref"]["state_generation"],
                Rejection::StaleSnapshot,
            )?;
        } else {
            require(
                query["target"]["instance_ref"].is_null() && query["parent_observation"].is_null(),
                Rejection::InvalidIdentity,
            )?;
        }
        if query["entity_kind"] == "rest_option" {
            self.rest_query(query, context)
        } else {
            require(
                binding["mode"] != "live"
                    || query["target"]["instance_ref"] == binding["instance_ref"],
                Rejection::InvalidIdentity,
            )
        }
    }

    fn rest_query(&self, query: &Value, context: &ValidationContext) -> Result {
        let capture = context.rest.as_ref().ok_or(Rejection::MissingCapability)?;
        let binding = &query["binding"];
        let live = binding["mode"] == "live";
        require(
            binding["content_manifest_id"] == capture.content_manifest_id
                && binding["locale"] == capture.locale,
            Rejection::StaleSnapshot,
        )?;
        if live {
            require(
                context.capabilities["rest_context"]["availability"] == "available",
                Rejection::MissingCapability,
            )?;
            require(
                *binding == capture.context["binding"]
                    && query["parent_observation"] == capture.context["parent_observation"],
                Rejection::StaleSnapshot,
            )?;
            if query["query_kind"] == "list" {
                require(
                    query["target"]["instance_ref"] == binding["instance_ref"],
                    Rejection::InvalidIdentity,
                )?;
            }
        } else if query["query_kind"] == "search" && !query["filters"]["display_name"].is_null() {
            require(capture.text_index_available, Rejection::UnsupportedFilter)?;
        }
        if !query["target"]["definition_ref"].is_null() {
            require(
                capture.entries.iter().any(|entry| {
                    entry.definition_ref == query["target"]["definition_ref"]
                        && (!live || entry.instance_ref == query["target"]["instance_ref"])
                }),
                Rejection::InvalidIdentity,
            )?;
        }
        if !live {
            require(
                array(&query["filters"]["instance_ids"])?.is_empty(),
                Rejection::UnsupportedFilter,
            )?;
        }
        Ok(())
    }

    pub(super) fn cursor(
        &self,
        cursor: &Value,
        query: &Value,
        context: &ValidationContext,
    ) -> Result {
        let text = cursor.as_str().ok_or(Rejection::StaleCursor)?;
        require(
            text.len() <= number(&context.capabilities["max_cursor_bytes"])?,
            Rejection::ResultLimitExceeded,
        )?;
        let known = context
            .cursors
            .iter()
            .find(|entry| entry.cursor == text)
            .ok_or(Rejection::StaleCursor)?;
        let revision = if query["entity_kind"] == "rest_option" {
            context
                .rest
                .as_ref()
                .ok_or(Rejection::MissingCapability)?
                .revision
                .as_str()
        } else {
            ""
        };
        require(
            known.profile == PROFILE
                && known.schema_digest == SCHEMA_DIGEST
                && known.producer_lease == context.authority.producer_lease
                && known.occurrence_revision == revision
                && known.normalized_query == normalized_query(query)?,
            Rejection::StaleCursor,
        )
    }
}

pub(super) fn limits(limits: &Value, capabilities: &Value) -> Result {
    for (name, ceiling) in [
        ("page_items", 32),
        ("item_bytes", 4096),
        ("page_bytes", 65536),
        ("text_bytes", 4096),
    ] {
        let amount = number(&limits[name])?;
        require(
            amount > 0 && amount <= ceiling && amount <= number(&capabilities[name])?,
            Rejection::ResultLimitExceeded,
        )?;
    }
    Ok(())
}
