// SPDX-License-Identifier: MIT

use super::*;

/// Request diagnostics only: retain every inherited envelope/query constraint.
/// This schema is never an acceptance oracle and never comes from caller input.
pub(super) fn compile(owned_schema: &Value) -> Result<jsonschema::Validator> {
    let mut diagnostic = owned_schema.clone();
    let conditions = diagnostic["$defs"]["query"]["allOf"]
        .as_array_mut()
        .ok_or(Rejection::Malformed)?;
    let matching: Vec<_> = conditions
        .iter()
        .enumerate()
        .filter_map(|(index, condition)| {
            (condition["if"]["properties"]["entity_kind"]["const"] == "rest_option")
                .then_some(index)
        })
        .collect();
    require(matching.len() == 1, Rejection::Malformed)?;
    conditions.remove(matching[0]);
    diagnostic["oneOf"] = serde_json::json!([{"$ref": "#/$defs/query_request"}]);
    jsonschema::validator_for(&diagnostic).map_err(|_| Rejection::Malformed)
}

impl Validator {
    pub(super) fn unsupported_request(&self, value: &Value) -> Rejection {
        if value["kind"] != "query_request" || value["query"]["entity_kind"] != "rest_option" {
            return Rejection::Malformed;
        }
        if size(value).is_ok_and(|bytes| bytes > MAX_REQUEST_BYTES) {
            return Rejection::ResultLimitExceeded;
        }
        if !self.request_shape.is_valid(value) {
            return Rejection::Malformed;
        }
        let unsupported = Unsupported::for_query(&value["query"]);
        let Some(rejection) = unsupported.rejection() else {
            return Rejection::Malformed;
        };
        let mut diagnostic = value.clone();
        unsupported.repair(&mut diagnostic["query"]);
        // A different malformed rest constraint must not be masked by a known unsupported selector.
        if self.envelope.is_valid(&diagnostic) {
            rejection
        } else {
            Rejection::Malformed
        }
    }
}

struct Unsupported {
    projection: bool,
    mode: bool,
    fields: bool,
    filters: bool,
}

impl Unsupported {
    fn for_query(query: &Value) -> Self {
        let live = query["binding"]["mode"] == "live";
        let allowed = if live { LIVE_FIELDS } else { STATIC_FIELDS };
        let mode = if live {
            matches!(query["query_kind"].as_str(), Some("get" | "search"))
        } else {
            query["query_kind"] == "detail"
        };
        Self {
            projection: query["projection"] != "standard" || query["detail_level"] != "standard",
            mode,
            fields: query["fields"].as_array().is_some_and(|fields| {
                fields
                    .iter()
                    .any(|field| field.as_str().is_some_and(|name| !allowed.contains(&name)))
            }),
            filters: live
                && (query["filters"]["display_name"] != Value::Null
                    || ["namespaced_ids", "definition_refs", "instance_ids"]
                        .iter()
                        .any(|key| {
                            query["filters"][key]
                                .as_array()
                                .is_some_and(|values| !values.is_empty())
                        })),
        }
    }

    fn rejection(&self) -> Option<Rejection> {
        if self.projection || self.mode {
            Some(Rejection::UnsupportedProjection)
        } else if self.fields {
            Some(Rejection::UnsupportedField)
        } else if self.filters {
            Some(Rejection::UnsupportedFilter)
        } else {
            None
        }
    }

    fn repair(&self, query: &mut Value) {
        let live = query["binding"]["mode"] == "live";
        if self.projection {
            query["projection"] = Value::String("standard".to_owned());
            query["detail_level"] = Value::String("standard".to_owned());
        }
        if self.mode {
            let kind = if !live {
                "get"
            } else if query["target"]["definition_ref"].is_null() {
                "list"
            } else {
                "detail"
            };
            query["query_kind"] = Value::String(kind.to_owned());
        }
        if self.fields
            && let Some(fields) = query["fields"].as_array_mut()
        {
            let allowed = if live { LIVE_FIELDS } else { STATIC_FIELDS };
            fields.retain(|field| field.as_str().is_some_and(|name| allowed.contains(&name)));
        }
        if self.filters {
            query["filters"]["display_name"] = Value::Null;
            for key in ["namespaced_ids", "definition_refs", "instance_ids"] {
                query["filters"][key] = Value::Array(Vec::new());
            }
        }
    }
}
