//! CLI integration tests using assert_cmd.
//!
//! These tests verify that the CLI interface works correctly.

use assert_cmd::cargo_bin_cmd;

#[test]
fn test_cli_runs() {
    // Test that the binary can be executed with the doctor command
    let mut cmd = cargo_bin_cmd!("shadow-secret");
    cmd.arg("doctor")
        .assert()
        .success()
        .stdout(predicates::str::contains("Shadow Secret Doctor"));
}

#[test]
#[ignore]
fn test_cli_version_flag() {
    // Test --version flag
    let mut cmd = cargo_bin_cmd!("shadow-secret");
    cmd.arg("--version").assert().success();
}

#[test]
#[ignore]
fn test_cli_help_flag() {
    // Test --help flag
    let mut cmd = cargo_bin_cmd!("shadow-secret");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Shadow Secret"));
}

#[test]
#[ignore]
fn test_cli_invalid_command() {
    // Test that invalid commands return an error
    let mut cmd = cargo_bin_cmd!("shadow-secret");
    cmd.arg("nonexistent-command").assert().failure();
}
