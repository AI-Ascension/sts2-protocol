// SPDX-License-Identifier: MIT

use super::*;

pub(super) fn validate(page: &Value) -> Result {
    let items = array(&page["items"])?;
    let item_bytes = items
        .iter()
        .map(size)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(0);
    let payload = size(&page["items"])?;
    let text = items
        .iter()
        .map(text_bytes)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    let mut bare = page.as_object().ok_or(Rejection::Malformed)?.clone();
    bare.remove("accounting");
    let page_bytes = size(&Value::Object(bare))?;
    for (name, actual) in [
        ("item_count", items.len()),
        ("item_bytes", item_bytes),
        ("payload_bytes", payload),
        ("page_bytes", page_bytes),
        ("text_bytes", text),
    ] {
        require(
            number(&page["accounting"][name])? == actual,
            Rejection::Malformed,
        )?;
    }
    for (name, actual) in [
        ("page_items", items.len()),
        ("item_bytes", item_bytes),
        ("page_bytes", page_bytes),
        ("text_bytes", text),
    ] {
        require(
            actual <= number(&page["limits"][name])?,
            Rejection::ResultLimitExceeded,
        )?;
    }
    Ok(())
}

fn text_bytes(item: &Value) -> Result<usize> {
    let mut count = 0;
    for field in array(&item["fields"])? {
        if field["availability"] != "available" {
            continue;
        }
        match field["kind"].as_str() {
            Some("text") => count += field["value"].as_str().ok_or(Rejection::Malformed)?.len(),
            Some("text_list") => {
                for text in array(&field["value"])? {
                    count += text.as_str().ok_or(Rejection::Malformed)?.len();
                }
            }
            _ => {}
        }
    }
    Ok(count)
}
