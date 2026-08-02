//! CLI smoke tests - these test command-line parsing and behavior
//! Note: Tests that would open windows are excluded to avoid interactive issues

use assert_cmd::Command;
use predicates::str::contains;
use std::path::PathBuf;

#[test]
fn help_flag_exits_success() {
    let mut cmd = Command::cargo_bin("fehrust").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("fehrust"))
        .stdout(contains("image viewer"));
}

#[test]
fn version_flag_exits_success() {
    let mut cmd = Command::cargo_bin("fehrust").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn list_mode_prints_info() {
    let img_path = PathBuf::from("tests/fixtures/test_1x1.png");
    let expected_size = std::fs::metadata(&img_path).unwrap().len().to_string();

    let mut cmd = Command::cargo_bin("fehrust").unwrap();
    cmd.arg("-L")
        .arg(&img_path)
        .assert()
        .success()
        .stdout(contains("test_1x1.png"))
        .stdout(contains(expected_size));
}

#[test]
fn customlist_format_filename_only() {
    let img_path = PathBuf::from("tests/fixtures/test_1x1.png");

    let mut cmd = Command::cargo_bin("fehrust").unwrap();
    cmd.arg("--customlist")
        .arg("%n")
        .arg(&img_path)
        .assert()
        .success()
        .stdout(contains("test_1x1.png"));
}
