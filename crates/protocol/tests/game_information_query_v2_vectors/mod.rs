// SPDX-License-Identifier: MIT

use serde_json::{Value, json};

pub fn canonical(value: &Value) -> String {
    serde_json::to_string(value).expect("JSON value")
}

pub fn apply(value: &mut Value, mutations: &Value) {
    for mutation in mutations.as_array().expect("mutations") {
        let path = mutation["path"].as_str().expect("JSON pointer");
        let (parent, leaf) = path.rsplit_once('/').expect("pointer member");
        let parent = value
            .pointer_mut(parent)
            .unwrap_or_else(|| panic!("missing parent {parent}"));
        if let Some(object) = parent.as_object_mut() {
            if mutation["remove"] == true {
                assert!(object.remove(leaf).is_some(), "existing member {path}");
            } else {
                object.insert(leaf.to_owned(), mutation["value"].clone());
            }
        } else if let Some(array) = parent.as_array_mut() {
            let index: usize = leaf.parse().expect("array index");
            if mutation["remove"] == true {
                array.remove(index);
            } else {
                array[index] = mutation["value"].clone();
            }
        } else {
            panic!("non-container parent {path}");
        }
    }
}

pub fn recount(value: &mut Value) {
    if value["result"].is_null() {
        return;
    }
    let page = &value["result"]["page"];
    let items = page["items"].as_array().expect("items");
    let count = items.len();
    let item_bytes = items
        .iter()
        .map(|item| canonical(item).len())
        .max()
        .unwrap_or(0);
    let payload = canonical(&page["items"]).len();
    let text_bytes = items
        .iter()
        .flat_map(|item| item["fields"].as_array().expect("fields"))
        .filter(|field| field["availability"] == "available")
        .map(|field| {
            if field["kind"] == "text" {
                field["value"].as_str().map_or(0, str::len)
            } else if field["kind"] == "text_list" {
                field["value"].as_array().map_or(0, |values| {
                    values.iter().filter_map(Value::as_str).map(str::len).sum()
                })
            } else {
                0
            }
        })
        .sum::<usize>();
    let mut bare = page.as_object().expect("page").clone();
    bare.remove("accounting");
    let page_bytes = canonical(&Value::Object(bare)).len();
    value["result"]["page"]["accounting"] = json!({
        "item_count": count, "item_bytes": item_bytes, "payload_bytes": payload,
        "page_bytes": page_bytes, "text_bytes": text_bytes,
    });
}
