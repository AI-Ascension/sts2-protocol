// SPDX-License-Identifier: MIT

use super::*;
use std::collections::BTreeSet;

impl Validator {
    pub(super) fn response(&self, value: &Value, context: &ValidationContext) -> Result {
        let query = &value["query"];
        let result = &value["result"];
        if query["binding"]["mode"] == "live" {
            require(
                result["result_generation"] == query["binding"]["snapshot_ref"]["state_generation"]
                    && result["parent_observation"] == query["parent_observation"],
                Rejection::MixedGeneration,
            )?;
        }
        let page = &result["page"];
        let items = array(&page["items"])?;
        require(page["limits"] == query["limits"], Rejection::Malformed)?;
        super::accounting::validate(page)?;
        if matches!(
            page["coverage"].as_str(),
            Some("unavailable" | "not_observable")
        ) {
            require(
                items.is_empty()
                    && page["final_page"] == true
                    && page["total_count_known"] == false
                    && page["next_cursor"].is_null()
                    && page["cursor_binding"].is_null(),
                Rejection::Malformed,
            )?;
        }
        if page["total_count_known"] == true {
            require(
                number(&page["total_count"])? >= items.len(),
                Rejection::Malformed,
            )?;
        }
        if !page["next_cursor"].is_null() {
            require(
                page["cursor_binding"] == normalized_query(query)?,
                Rejection::StaleCursor,
            )?;
            self.cursor(&page["next_cursor"], query, context)?;
        }
        if query["entity_kind"] == "rest_option" {
            self.rest_page(page, query, context)?;
        } else {
            for item in items {
                common_item(item, query)?;
                require(
                    query["binding"]["mode"] != "live"
                        || item["instance_ref"] == query["binding"]["instance_ref"],
                    Rejection::InvalidIdentity,
                )?;
            }
        }
        Ok(())
    }

    fn rest_page(&self, page: &Value, query: &Value, context: &ValidationContext) -> Result {
        require(
            page["ordering"]["key"] == "definition_ref"
                && page["ordering"]["direction"] == "ascending"
                && page["ordering"]["algorithm"] == "identity_bytes",
            Rejection::Malformed,
        )?;
        let items = array(&page["items"])?;
        let capture = context.rest.as_ref().ok_or(Rejection::MissingCapability)?;
        let live = query["binding"]["mode"] == "live";
        let fields = requested_fields(query)?;
        let mut previous = None;
        let mut occurrences = BTreeSet::new();
        for item in items {
            common_item(item, query)?;
            let definition = item["definition_ref"].to_string();
            require(
                previous.as_ref().is_none_or(|old| *old < definition),
                Rejection::AmbiguousId,
            )?;
            previous = Some(definition);
            let entry = capture
                .entries
                .iter()
                .find(|entry| entry.definition_ref == item["definition_ref"])
                .ok_or(Rejection::InvalidIdentity)?;
            if live {
                require(
                    item["instance_ref"] == entry.instance_ref
                        && occurrences.insert(item["instance_ref"].to_string()),
                    Rejection::InvalidIdentity,
                )?;
            }
            let projected: Vec<_> = entry
                .fields
                .iter()
                .filter(|field| fields.contains(&field["name"].as_str().unwrap_or("")))
                .cloned()
                .collect();
            require(
                item["fields"] == Value::Array(projected),
                Rejection::UnsupportedField,
            )?;
        }
        if page["coverage"] == "complete" {
            require(
                page["final_page"] == true
                    && capture.fully_classified
                    && items.iter().all(|item| {
                        item["fields"].as_array().is_some_and(|fields| {
                            fields
                                .iter()
                                .all(|field| field["availability"] == "available")
                        })
                    }),
                Rejection::Malformed,
            )?;
        }
        if matches!(
            query["query_kind"].as_str(),
            Some("get" | "detail" | "availability")
        ) {
            require(items.len() <= 1, Rejection::Malformed)?;
        }
        if !matches!(
            page["coverage"].as_str(),
            Some("unavailable" | "not_observable")
        ) {
            self.traversal(page, query, context)?;
        }
        Ok(())
    }

    fn traversal(&self, page: &Value, query: &Value, context: &ValidationContext) -> Result {
        let capture = context.rest.as_ref().ok_or(Rejection::MissingCapability)?;
        let mut expected: Vec<_> = capture
            .entries
            .iter()
            .filter(|entry| selected(entry, query))
            .collect();
        expected.sort_by_key(|entry| entry.definition_ref.to_string());
        let offset = if let Some(cursor) = query["cursor"].as_str() {
            context
                .cursors
                .iter()
                .find(|known| known.cursor == cursor)
                .ok_or(Rejection::StaleCursor)?
                .offset
        } else {
            0
        };
        let items = array(&page["items"])?;
        require(
            offset <= expected.len() && items.len() <= expected.len() - offset,
            Rejection::InvalidIdentity,
        )?;
        for (item, entry) in items.iter().zip(&expected[offset..]) {
            require(
                item["definition_ref"] == entry.definition_ref,
                Rejection::InvalidIdentity,
            )?;
        }
        let end = offset + items.len();
        require(
            page["final_page"] == (end == expected.len()),
            Rejection::Malformed,
        )?;
        require(
            page["final_page"] == true || !items.is_empty(),
            Rejection::Malformed,
        )?;
        if page["total_count_known"] == true {
            require(
                number(&page["total_count"])? == expected.len(),
                Rejection::Malformed,
            )?;
        }
        if let Some(cursor) = page["next_cursor"].as_str() {
            require(
                context
                    .cursors
                    .iter()
                    .any(|known| known.cursor == cursor && known.offset == end),
                Rejection::StaleCursor,
            )?;
        }
        Ok(())
    }
}

fn selected(entry: &RestEntry, query: &Value) -> bool {
    let definition = &entry.definition_ref;
    let filters = &query["filters"];
    let target = &query["target"]["definition_ref"];
    let ids = filters["namespaced_ids"].as_array();
    let refs = filters["definition_refs"].as_array();
    let display = &filters["display_name"];
    (target.is_null() || target == definition)
        && ids.is_some_and(|values| {
            values.is_empty() || values.contains(&definition["namespaced_id"])
        })
        && refs.is_some_and(|values| values.is_empty() || values.contains(definition))
        && (display.is_null()
            || entry.fields.iter().any(|field| {
                field["name"] == "display_name"
                    && field["availability"] == "available"
                    && field["value"] == *display
            }))
}

fn common_item(item: &Value, query: &Value) -> Result {
    require(
        item["definition_ref"]["content_manifest_id"] == query["binding"]["content_manifest_id"]
            && item["definition_ref"]["entity_kind"] == query["entity_kind"],
        Rejection::InvalidIdentity,
    )?;
    if query["binding"]["mode"] == "static" {
        require(item["instance_ref"].is_null(), Rejection::InvalidIdentity)?;
    }
    if !query["target"]["definition_ref"].is_null() {
        require(
            item["definition_ref"] == query["target"]["definition_ref"],
            Rejection::InvalidIdentity,
        )?;
    }
    if query["binding"]["mode"] == "live"
        && matches!(
            query["query_kind"].as_str(),
            Some("detail" | "availability" | "get")
        )
    {
        require(
            item["instance_ref"] == query["target"]["instance_ref"],
            Rejection::InvalidIdentity,
        )?;
    }
    let names: Vec<_> = array(&item["fields"])?
        .iter()
        .map(|field| field["name"].as_str().ok_or(Rejection::Malformed))
        .collect::<Result<_>>()?;
    require(
        names.windows(2).all(|pair| pair[0] < pair[1]),
        Rejection::Malformed,
    )?;
    let requested = array(&query["fields"])?;
    if !requested.is_empty() {
        require(
            names.len() == requested.len()
                && requested
                    .iter()
                    .all(|field| field.as_str().is_some_and(|name| names.contains(&name))),
            Rejection::UnsupportedField,
        )?;
    }
    let filter = &query["filters"];
    require(
        array(&filter["namespaced_ids"])?.is_empty()
            || array(&filter["namespaced_ids"])?.contains(&item["definition_ref"]["namespaced_id"]),
        Rejection::InvalidIdentity,
    )?;
    require(
        array(&filter["definition_refs"])?.is_empty()
            || array(&filter["definition_refs"])?.contains(&item["definition_ref"]),
        Rejection::InvalidIdentity,
    )?;
    Ok(())
}

fn requested_fields(query: &Value) -> Result<Vec<&str>> {
    let fields = array(&query["fields"])?;
    if fields.is_empty() {
        Ok(if query["binding"]["mode"] == "live" {
            LIVE_FIELDS
        } else {
            STATIC_FIELDS
        }
        .to_vec())
    } else {
        fields
            .iter()
            .map(|field| field.as_str().ok_or(Rejection::UnsupportedField))
            .collect()
    }
}
