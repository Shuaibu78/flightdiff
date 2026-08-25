#![allow(clippy::unwrap_used, missing_docs)]

//! Snapshots of what the binary actually prints, run against the committed
//! fixtures in `testdata/`.

use std::{path::PathBuf, process::Command};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(args: &[&str]) -> String {
    let paths: Vec<String> = args
        .iter()
        .skip(1)
        .map(|name| format!("testdata/{name}"))
        .collect();
    let output = Command::new(env!("CARGO_BIN_EXE_flightdiff"))
        .current_dir(repo_root())
        .arg(args[0])
        .args(&paths)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{} exited with {}",
        args[0],
        output.status
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn info_reports_a_complete_log() {
    insta::assert_snapshot!(run(&["info", "crash.ulg"]));
}

#[test]
fn info_flags_a_truncated_log() {
    insta::assert_snapshot!(run(&["info", "truncated.ulg"]));
}

#[test]
fn diff_reports_what_changed_between_two_flights() {
    insta::assert_snapshot!(run(&["diff", "good.ulg", "crash.ulg"]));
}

#[test]
fn diff_against_a_truncated_log() {
    insta::assert_snapshot!(run(&["diff", "good.ulg", "truncated.ulg"]));
}

#[test]
fn diff_reports_truncation_alongside_real_differences() {
    insta::assert_snapshot!(run(&["diff", "crash.ulg", "truncated.ulg"]));
}
