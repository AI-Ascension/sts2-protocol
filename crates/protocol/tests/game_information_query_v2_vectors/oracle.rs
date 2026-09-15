// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
pub struct Reference {
    pub valid: bool,
    pub canonical: Option<String>,
}

/// One bounded child for the whole corpus; every result is computed from the actual raw variant.
pub fn exact_reference(root: &Path, raw: &[String]) -> Vec<Reference> {
    let mut child = Command::new("node")
        .arg(root.join("tools/game-information-v2/numeric-reference.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("existing pinned Node reference runtime");
    let mut input = child.stdin.take().expect("oracle input");
    input
        .write_all(&serde_json::to_vec(raw).expect("raw case batch"))
        .expect("oracle input write");
    drop(input);
    let output = child.wait_with_output().expect("oracle terminal output");
    assert!(
        output.status.success(),
        "exact numeric oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let references: Vec<Reference> =
        serde_json::from_slice(&output.stdout).expect("oracle results");
    assert_eq!(references.len(), raw.len());
    references
}
