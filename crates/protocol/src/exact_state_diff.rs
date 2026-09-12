// SPDX-License-Identifier: MIT

//! Bounded, path-addressed differences between two canonical exact-state payloads.
//!
//! A privileged diagnostic must show changed paths and value kinds, not megabytes of raw state.
//! Leaves report exact JSON-Pointer paths; containers recurse and arrays compare index-wise so
//! reordering is visible. Output is bounded: exceeding the depth or entry budget is a refusal
//! rather than a silently truncated diff.

use crate::exact_state::CanonicalValue;

/// Maximum nesting examined while diffing.
pub const MAX_DIFF_DEPTH: usize = 64;
/// Maximum differences reported before the diff refuses.
pub const MAX_DIFF_ENTRIES: usize = 1024;

/// Kind of difference observed at one path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DifferenceKind {
    /// The right payload has a member or element the left lacks.
    Added,
    /// The left payload has a member or element the right lacks.
    Removed,
    /// Both sides exist but their value kinds differ.
    TypeChanged,
    /// Both sides exist with the same kind but different scalar values.
    ValueChanged,
}

impl DifferenceKind {
    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::TypeChanged => "type_changed",
            Self::ValueChanged => "value_changed",
        }
    }
}

/// One path-addressed difference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Difference {
    /// JSON Pointer path to the changed node; the empty string is the root.
    pub path: String,
    /// Kind of change at that path.
    pub kind: DifferenceKind,
}

/// Rejection reasons for a bounded diff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffError {
    /// The payloads nest deeper than [`MAX_DIFF_DEPTH`].
    TooDeep,
    /// More than [`MAX_DIFF_ENTRIES`] differences exist.
    TooManyEntries,
}

/// Returns bounded path-addressed differences between two canonical values.
pub fn describe_differences(
    left: &CanonicalValue,
    right: &CanonicalValue,
) -> Result<Vec<Difference>, DiffError> {
    let mut differences = Vec::new();
    walk("", left, right, 0, &mut differences)?;
    Ok(differences)
}

fn walk(
    path: &str,
    left: &CanonicalValue,
    right: &CanonicalValue,
    depth: usize,
    out: &mut Vec<Difference>,
) -> Result<(), DiffError> {
    if depth > MAX_DIFF_DEPTH {
        return Err(DiffError::TooDeep);
    }
    match (left, right) {
        (CanonicalValue::Object(left_entries), CanonicalValue::Object(right_entries)) => {
            for (key, value) in left_entries {
                let child = child_path(path, key);
                match right_entries.get(key) {
                    Some(other) => walk(&child, value, other, depth + 1, out)?,
                    None => push(out, child, DifferenceKind::Removed)?,
                }
            }
            for key in right_entries.keys() {
                if !left_entries.contains_key(key) {
                    push(out, child_path(path, key), DifferenceKind::Added)?;
                }
            }
            Ok(())
        }
        (CanonicalValue::Array(left_items), CanonicalValue::Array(right_items)) => {
            for (index, value) in left_items.iter().enumerate() {
                let child = child_path(path, &index.to_string());
                match right_items.get(index) {
                    Some(other) => walk(&child, value, other, depth + 1, out)?,
                    None => push(out, child, DifferenceKind::Removed)?,
                }
            }
            for index in left_items.len()..right_items.len() {
                push(
                    out,
                    child_path(path, &index.to_string()),
                    DifferenceKind::Added,
                )?;
            }
            Ok(())
        }
        (left_scalar, right_scalar) => {
            if std::mem::discriminant(left_scalar) != std::mem::discriminant(right_scalar) {
                push(out, path.to_owned(), DifferenceKind::TypeChanged)?;
            } else if left_scalar != right_scalar {
                push(out, path.to_owned(), DifferenceKind::ValueChanged)?;
            }
            Ok(())
        }
    }
}

fn push(out: &mut Vec<Difference>, path: String, kind: DifferenceKind) -> Result<(), DiffError> {
    if out.len() >= MAX_DIFF_ENTRIES {
        return Err(DiffError::TooManyEntries);
    }
    out.push(Difference { path, kind });
    Ok(())
}

fn child_path(parent: &str, segment: &str) -> String {
    let escaped = segment.replace('~', "~0").replace('/', "~1");
    format!("{parent}/{escaped}")
}
