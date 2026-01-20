use assert_cmd::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_tmp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("solprivacy_test_{}", nanos))
}

#[test]
fn config_show_runs() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("config").arg("show");
    cmd.assert()
        .success()
        .stdout(contains("Current Configuration:"))
        .stdout(contains("Active RPC URL"));
}

#[test]
fn keygen_auditor_generates_file() {
    let base = unique_tmp_dir();
    fs::create_dir_all(&base).unwrap();
    let out = base.join("auditor.json");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("keygen")
        .arg("auditor")
        .arg("-o")
        .arg(&out)
        .arg("--force");
    cmd.assert().success().stdout(contains("keypair generated"));

    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("\"key_type\": \"auditor\""));

    fs::remove_file(&out).unwrap();
    fs::remove_dir(&base).unwrap();
}

#[test]
fn debug_decrypt_without_tx_shows_usage() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("debug").arg("--decrypt");
    cmd.assert()
        .success()
        .stdout(contains("Transaction signature required!"))
        .stdout(contains("Usage"));
}

#[test]
fn fund_private_dry_run_outputs_plan() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("fund")
        .arg("--private")
        .arg("--to")
        .arg("5wHk9eBGheZ1VL1e5Bvj3tVo7p9XLh2k7W8xYZ")
        .arg("--amount")
        .arg("0.1")
        .arg("--from")
        .arg("3xPmKqN8vRtY2uWsD4fA6gH7jL9kM1nO5pQxYZ")
        .arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(contains("Dry run - no transaction sent"))
        .stdout(contains("Deposit"))
        .stdout(contains("Generate ZK proof"))
        .stdout(contains("Withdraw"));
}
