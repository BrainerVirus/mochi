//! Smoke test for the diagnostics CLI subcommand. Verifies the binary
//! starts, parses the `diagnostics` command, and exits cleanly.

use std::process::Command;

#[test]
fn diagnostics_cli_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_mochi"))
        .arg("diagnostics")
        .output()
        .expect("mochi binary should be invokable");

    assert!(
        output.status.success(),
        "diagnostics command failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("mochi") || stdout.contains("Mochi") || stdout.contains("version"),
        "diagnostics output should mention mochi or version, got: {}",
        stdout
    );
}

#[test]
fn help_lists_diagnostics_subcommand() {
    let output = Command::new(env!("CARGO_BIN_EXE_mochi"))
        .arg("--help")
        .output()
        .expect("mochi binary should be invokable");

    assert!(output.status.success(), "--help failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("diagnostics"),
        "--help should list diagnostics subcommand"
    );
}
