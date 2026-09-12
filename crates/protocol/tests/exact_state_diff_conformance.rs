// SPDX-License-Identifier: MIT

use sts2_protocol::{
    CanonicalValue, DiffError, Difference, DifferenceKind, MAX_DIFF_DEPTH, describe_differences,
};

fn parse(text: &str) -> CanonicalValue {
    CanonicalValue::parse_str(text).expect("payload is inside the profile")
}

fn diff(left: &str, right: &str) -> Vec<Difference> {
    describe_differences(&parse(left), &parse(right)).expect("diff fits its budget")
}

#[test]
fn identical_values_have_no_differences() {
    let payload = r#"{"gameplay":{"hp":42,"deck":["a","b"]},"execution":{},"rng":[]}"#;
    assert!(diff(payload, payload).is_empty());
}

#[test]
fn nested_scalar_changes_report_exact_paths() {
    let left = r#"{"gameplay":{"player":{"hp":42,"gold":10}}}"#;
    let right = r#"{"gameplay":{"player":{"hp":41,"gold":10}}}"#;
    assert_eq!(
        diff(left, right),
        vec![Difference {
            path: "/gameplay/player/hp".to_owned(),
            kind: DifferenceKind::ValueChanged,
        }]
    );
}

#[test]
fn added_and_removed_members_are_reported() {
    let left = r#"{"gameplay":{"a":1,"b":2}}"#;
    let right = r#"{"gameplay":{"a":1,"c":3}}"#;
    let differences = diff(left, right);
    assert!(differences.contains(&Difference {
        path: "/gameplay/b".to_owned(),
        kind: DifferenceKind::Removed,
    }));
    assert!(differences.contains(&Difference {
        path: "/gameplay/c".to_owned(),
        kind: DifferenceKind::Added,
    }));
    assert_eq!(differences.len(), 2);
}

#[test]
fn type_changes_are_distinguished_from_value_changes() {
    let left = r#"{"gameplay":{"floor":3}}"#;
    let right = r#"{"gameplay":{"floor":"3"}}"#;
    assert_eq!(
        diff(left, right),
        vec![Difference {
            path: "/gameplay/floor".to_owned(),
            kind: DifferenceKind::TypeChanged,
        }]
    );
    let truthy = diff(
        r#"{"gameplay":{"flag":1}}"#,
        r#"{"gameplay":{"flag":true}}"#,
    );
    assert_eq!(truthy[0].kind, DifferenceKind::TypeChanged);
}

#[test]
fn array_reordering_and_length_changes_are_visible() {
    let reordered = diff(r#"{"deck":[1,2]}"#, r#"{"deck":[2,1]}"#);
    assert_eq!(
        reordered,
        vec![
            Difference {
                path: "/deck/0".to_owned(),
                kind: DifferenceKind::ValueChanged,
            },
            Difference {
                path: "/deck/1".to_owned(),
                kind: DifferenceKind::ValueChanged,
            },
        ]
    );
    let removed = diff(r#"{"deck":[1,2,3]}"#, r#"{"deck":[1,2]}"#);
    assert_eq!(
        removed,
        vec![Difference {
            path: "/deck/2".to_owned(),
            kind: DifferenceKind::Removed,
        }]
    );
    let added = diff(r#"{"deck":[1]}"#, r#"{"deck":[1,2]}"#);
    assert_eq!(added[0].kind, DifferenceKind::Added);
    assert_eq!(added[0].path, "/deck/1");
}

#[test]
fn depth_and_entry_budgets_refuse_instead_of_truncating() {
    let mut deep = CanonicalValue::Int(0);
    for _ in 0..=MAX_DIFF_DEPTH {
        deep = CanonicalValue::Array(vec![deep]);
    }
    assert_eq!(
        describe_differences(&deep, &deep).expect_err("depth refused"),
        DiffError::TooDeep
    );

    let mut left = String::from("{\"gameplay\":{");
    let mut right = String::from("{\"gameplay\":{");
    for index in 0..1100 {
        if index > 0 {
            left.push(',');
            right.push(',');
        }
        left.push_str(&format!("\"k{index}\":{index}"));
        right.push_str(&format!("\"k{index}\":{}", index + 1));
    }
    left.push_str("}}");
    right.push_str("}}");
    assert_eq!(
        describe_differences(&parse(&left), &parse(&right)).expect_err("entries refused"),
        DiffError::TooManyEntries
    );
}

#[test]
fn differences_carry_paths_and_kinds_only() {
    let left = r#"{"gameplay":{"blob":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#;
    let right = r#"{"gameplay":{"blob":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}"#;
    let differences = diff(left, right);
    assert_eq!(differences.len(), 1);
    assert_eq!(differences[0].path, "/gameplay/blob");
    assert_eq!(differences[0].kind.as_str(), "value_changed");
    let rendered = format!("{:?}", differences[0]);
    assert!(!rendered.contains("aaaa"));
    assert!(!rendered.contains("bbbb"));
}
