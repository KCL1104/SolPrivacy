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

// ============================================================================
// Config Command Tests
// ============================================================================

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
fn config_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("config").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("show"))
        .stdout(contains("network"))
        .stdout(contains("provider"));
}

// ============================================================================
// Keygen Command Tests
// ============================================================================

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
fn keygen_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("keygen").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("auditor"))
        .stdout(contains("ElGamal"));
}

// ============================================================================
// Debug Command Tests
// ============================================================================

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
fn debug_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("debug").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--decrypt"))
        .stdout(contains("--tx"));
}

// ============================================================================
// Fund Command Tests
// ============================================================================

#[test]
fn fund_private_dry_run_outputs_alternatives() {
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
        .stdout(contains("Privacy Cash Integration Unavailable"))
        .stdout(contains("Alternatives"))
        .stdout(contains("Token-2022"))
        .stdout(contains("Light Protocol"));
}

#[test]
fn fund_private_shows_limitation_notice() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("fund")
        .arg("--private")
        .arg("--to")
        .arg("5wHk9eBGheZ1VL1e5Bvj3tVo7p9XLh2k7W8xYZ")
        .arg("--amount")
        .arg("0.1")
        .arg("--from")
        .arg("3xPmKqN8vRtY2uWsD4fA6gH7jL9kM1nO5pQxYZ");
    cmd.assert()
        .success()
        .stdout(contains("Privacy Cash Integration Unavailable"));
}

// ============================================================================
// Wallet Command Tests
// ============================================================================

#[test]
fn wallet_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("wallet").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("new"))
        .stdout(contains("import"))
        .stdout(contains("balance"))
        .stdout(contains("airdrop"))
        .stdout(contains("list"))
        .stdout(contains("address"));
}

#[test]
fn wallet_new_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("wallet").arg("new").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--name"))
        .stdout(contains("--force"));
}

#[test]
fn wallet_list_runs_successfully() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("wallet").arg("list");
    cmd.assert()
        .success()
        .stdout(contains("Configured Wallets"));
}

#[test]
fn wallet_address_nonexistent_shows_error() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("wallet").arg("address").arg("nonexistent_wallet_12345");
    cmd.assert()
        .success()
        .stdout(contains("not found"));
}

#[test]
fn wallet_import_requires_source() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("wallet").arg("import").arg("--name").arg("test");
    cmd.assert()
        .success()
        .stdout(contains("Provide --private-key or --keypair"));
}

// ============================================================================
// Setup Command Tests
// ============================================================================

#[test]
fn setup_runs_and_shows_toolchain_status() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("setup");
    cmd.assert()
        .success()
        .stdout(contains("Privacy Development Toolchain Setup"))
        .stdout(contains("Toolchain Status"))
        .stdout(contains("Rust"));
}

#[test]
fn setup_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("setup").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--install"))
        .stdout(contains("--check"));
}

#[test]
fn setup_check_specific_tool() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("setup").arg("--check").arg("rust");
    cmd.assert()
        .success()
        .stdout(contains("Rust"));
}

#[test]
fn setup_check_unknown_tool_shows_error() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("setup").arg("--check").arg("nonexistent_tool");
    cmd.assert()
        .success()
        .stdout(contains("Unknown tool"))
        .stdout(contains("Available"));
}

// ============================================================================
// Dev Command Tests
// ============================================================================

#[test]
fn dev_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("dev").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("start"))
        .stdout(contains("stop"))
        .stdout(contains("status"))
        .stdout(contains("logs"))
        .stdout(contains("reset"));
}

#[test]
fn dev_start_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("dev").arg("start").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--reset"))
        .stdout(contains("--verbose"));
}

#[test]
fn dev_status_runs_successfully() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("dev").arg("status");
    cmd.assert()
        .success()
        .stdout(contains("Development Environment Status"))
        .stdout(contains("Local Validator"));
}

#[test]
fn dev_logs_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("dev").arg("logs").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--lines"))
        .stdout(contains("--follow"));
}

// ============================================================================
// Light Command Tests
// ============================================================================

#[test]
fn light_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("setup"))
        .stdout(contains("create-mint"))
        .stdout(contains("mint"))
        .stdout(contains("transfer"))
        .stdout(contains("compress-sol"))
        .stdout(contains("decompress-sol"))
        .stdout(contains("info"));
}

#[test]
fn light_info_shows_zk_compression_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("info");
    cmd.assert()
        .success()
        .stdout(contains("Light Protocol"))
        .stdout(contains("ZK Compression"))
        .stdout(contains("Cost Comparison"));
}

#[test]
fn light_setup_shows_installation_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("setup");
    cmd.assert()
        .success()
        .stdout(contains("Light Protocol Setup"));
}

#[test]
fn light_create_mint_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("create-mint").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--decimals"));
}

#[test]
fn light_accounts_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("accounts").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--owner"))
        .stdout(contains("--rpc-url"));
}

#[test]
fn light_balance_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("balance").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--owner"))
        .stdout(contains("--mint"))
        .stdout(contains("--rpc-url"));
}

#[test]
fn light_health_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("health").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--rpc-url"));
}

#[test]
fn light_compare_shows_cost_analysis() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("compare").arg("--count").arg("100").arg("--size").arg("100");
    cmd.assert()
        .success()
        .stdout(contains("Cost Comparison"))
        .stdout(contains("Regular Accounts"))
        .stdout(contains("Compressed Accounts"))
        .stdout(contains("Savings"));
}

#[test]
fn light_workflow_shows_complete_guide() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("workflow");
    cmd.assert()
        .success()
        .stdout(contains("ZK Compression Complete Workflow"))
        .stdout(contains("Step 1"))
        .stdout(contains("Step 2"))
        .stdout(contains("Compress SOL"))
        .stdout(contains("Native SDK Queries"));
}

#[test]
fn light_config_show_runs() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("config").arg("--show");
    cmd.assert()
        .success()
        .stdout(contains("Light Protocol Configuration"));
}

#[test]
fn light_info_shows_native_sdk_status() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("light").arg("info");
    cmd.assert()
        .success()
        .stdout(contains("Native SDK Integration"))
        .stdout(contains("light-client"));
}

// ============================================================================
// Docs Command Tests
// ============================================================================

#[test]
fn docs_without_topic_shows_all() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("docs");
    cmd.assert()
        .success()
        .stdout(contains("Privacy Development Documentation"))
        .stdout(contains("Available Topics"))
        .stdout(contains("token2022"))
        .stdout(contains("light"))
        .stdout(contains("arcium"))
        .stdout(contains("noir"))
        .stdout(contains("hackathon"));
}

#[test]
fn docs_help_shows_topics() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("docs").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("token2022"))
        .stdout(contains("light"))
        .stdout(contains("arcium"))
        .stdout(contains("noir"))
        .stdout(contains("privacy-cash"))
        .stdout(contains("helius"))
        .stdout(contains("quicknode"))
        .stdout(contains("hackathon"));
}

#[test]
fn docs_token2022_shows_correct_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("docs").arg("token2022");
    cmd.assert()
        .success()
        .stdout(contains("Token-2022 Confidential Transfers"))
        .stdout(contains("solana.com"));
}

#[test]
fn docs_hackathon_shows_correct_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("docs").arg("hackathon");
    cmd.assert()
        .success()
        .stdout(contains("Solana Privacy Hackathon"))
        .stdout(contains("privacyhack"));
}

// ============================================================================
// Examples Command Tests
// ============================================================================

#[test]
fn examples_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("examples").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("clone"));
}

#[test]
fn examples_list_shows_available_examples() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("examples").arg("list");
    cmd.assert()
        .success()
        .stdout(contains("Available Example Projects"))
        .stdout(contains("noir-solana"))
        .stdout(contains("token2022-confidential"))
        .stdout(contains("light-compressed-token"))
        .stdout(contains("anchor-privacy"));
}

#[test]
fn examples_clone_unknown_shows_error() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("examples").arg("clone").arg("nonexistent_example_12345");
    cmd.assert()
        .success()
        .stdout(contains("Unknown example"))
        .stdout(contains("Available examples"));
}

#[test]
fn examples_clone_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("examples").arg("clone").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("--output"))
        .stdout(contains("<NAME>"));
}

// ============================================================================
// ZK Command Tests
// ============================================================================

#[test]
fn zk_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("init"))
        .stdout(contains("compile"))
        .stdout(contains("prove"))
        .stdout(contains("verify"))
        .stdout(contains("test"))
        .stdout(contains("solana"))
        .stdout(contains("templates"))
        .stdout(contains("workflow"));
}

#[test]
fn zk_templates_shows_all_templates() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("templates");
    cmd.assert()
        .success()
        .stdout(contains("Available Circuit Templates"))
        .stdout(contains("ownership"))
        .stdout(contains("merkle"))
        .stdout(contains("signature"))
        .stdout(contains("range"))
        .stdout(contains("balance"))
        .stdout(contains("nullifier"))
        .stdout(contains("privacy-transfer"));
}

#[test]
fn zk_workflow_shows_complete_guide() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("workflow");
    cmd.assert()
        .success()
        .stdout(contains("ZK Development Complete Workflow"))
        .stdout(contains("Phase 1"))
        .stdout(contains("Phase 2"))
        .stdout(contains("Phase 3"))
        .stdout(contains("Solana Deployment"));
}

#[test]
fn zk_setup_shows_installation() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("setup");
    cmd.assert()
        .success()
        .stdout(contains("Noir ZK Development Setup"))
        .stdout(contains("nargo"))
        .stdout(contains("noirup"));
}

#[test]
fn zk_solana_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("solana").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("setup"))
        .stdout(contains("keygen"))
        .stdout(contains("verifier"))
        .stdout(contains("install"));
}

#[test]
fn zk_solana_install_shows_instructions() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("solana").arg("install");
    cmd.assert()
        .success()
        .stdout(contains("Sunspot Installation"))
        .stdout(contains("Go 1.24"))
        .stdout(contains("reilabs/sunspot"));
}

#[test]
fn zk_init_help_shows_templates() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("zk").arg("init").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("ownership"))
        .stdout(contains("merkle"))
        .stdout(contains("range"));
}

// ============================================================================
// Main CLI Tests
// ============================================================================

#[test]
fn main_help_shows_all_commands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Privacy Orchestration Layer for Solana"))
        .stdout(contains("wallet"))
        .stdout(contains("setup"))
        .stdout(contains("dev"))
        .stdout(contains("light"))
        .stdout(contains("docs"))
        .stdout(contains("examples"))
        .stdout(contains("config"))
        .stdout(contains("keygen"))
        .stdout(contains("mint"))
        .stdout(contains("transfer"))
        .stdout(contains("fund"))
        .stdout(contains("debug"))
        .stdout(contains("zk"))
        .stdout(contains("confidential"));
}

#[test]
fn main_version_shows_version() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(contains("solprivacy"));
}

// ============================================================================
// Confidential Command Tests
// ============================================================================

#[test]
fn confidential_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("confidential").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("configure"))
        .stdout(contains("deposit"))
        .stdout(contains("apply"))
        .stdout(contains("transfer"))
        .stdout(contains("withdraw"))
        .stdout(contains("balance"))
        .stdout(contains("workflow"));
}

#[test]
fn confidential_workflow_shows_steps() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("confidential").arg("workflow");
    cmd.assert()
        .success()
        .stdout(contains("SETUP PHASE"))
        .stdout(contains("CREATE CONFIDENTIAL TOKEN"))
        .stdout(contains("CONFIGURE ACCOUNT"))
        .stdout(contains("DEPOSIT TO CONFIDENTIAL BALANCE"))
        .stdout(contains("APPLY PENDING BALANCE"))
        .stdout(contains("CONFIDENTIAL TRANSFER"));
}

#[test]
fn confidential_configure_dry_run() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("confidential")
        .arg("configure")
        .arg("--account")
        .arg("11111111111111111111111111111111")
        .arg("--mint")
        .arg("11111111111111111111111111111111")
        .arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(contains("Configure Confidential Transfer Account"))
        .stdout(contains("Configuration Requirements"));
}

#[test]
fn confidential_deposit_dry_run() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("confidential")
        .arg("deposit")
        .arg("--account")
        .arg("11111111111111111111111111111111")
        .arg("--mint")
        .arg("11111111111111111111111111111111")
        .arg("--amount")
        .arg("100")
        .arg("--decimals")
        .arg("9")
        .arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(contains("Deposit to Confidential Balance"))
        .stdout(contains("What Deposit Does"));
}

#[test]
fn confidential_transfer_dry_run() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("confidential")
        .arg("transfer")
        .arg("--mint")
        .arg("11111111111111111111111111111111")
        .arg("--to")
        .arg("11111111111111111111111111111111")
        .arg("--amount")
        .arg("50")
        .arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(contains("Confidential Transfer"))
        .stdout(contains("ZK Proofs Required"));
}

// ============================================================================
// Compliance Command Tests (Range Protocol)
// ============================================================================

#[test]
fn compliance_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("check"))
        .stdout(contains("batch"))
        .stdout(contains("transaction"))
        .stdout(contains("integrate"))
        .stdout(contains("info"));
}

#[test]
fn compliance_info_shows_range_protocol() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("info");
    cmd.assert()
        .success()
        .stdout(contains("Range Protocol"))
        .stdout(contains("Risk"));
}

#[test]
fn compliance_integrate_shows_guide() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("integrate");
    cmd.assert()
        .success()
        .stdout(contains("Integration"));
}

#[test]
fn compliance_check_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("check").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("address"))
        .stdout(contains("format"));
}

#[test]
fn compliance_batch_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("batch").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("addresses"));
}

#[test]
fn compliance_transaction_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("compliance").arg("transaction").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("signature"));
}

// ============================================================================
// Helius Command Tests (DAS, Priority Fees, Webhooks)
// ============================================================================

#[test]
fn helius_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("assets"))
        .stdout(contains("fees"))
        .stdout(contains("webhooks"))
        .stdout(contains("send"))
        .stdout(contains("info"));
}

#[test]
fn helius_info_shows_api_info() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("info");
    cmd.assert()
        .success()
        .stdout(contains("Helius"))
        .stdout(contains("API"));
}

#[test]
fn helius_assets_help_shows_queries() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("assets").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("get"))
        .stdout(contains("by-owner"))
        .stdout(contains("search"));
}

#[test]
fn helius_fees_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("fees").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("transaction"))
        .stdout(contains("accounts"))
        .stdout(contains("level"));
}

#[test]
fn helius_webhooks_help_shows_actions() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("webhooks").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("create"))
        .stdout(contains("delete"))
        .stdout(contains("guide"));
}

#[test]
fn helius_webhooks_guide_shows_documentation() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("webhooks").arg("guide");
    cmd.assert()
        .success()
        .stdout(contains("Webhook"));
}

#[test]
fn helius_send_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("send").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("transaction"));
}

#[test]
fn helius_config_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("helius").arg("config").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("api-key"))
        .stdout(contains("show"));
}

// ============================================================================
// Doctor Command Tests (Diagnostics)
// ============================================================================

#[test]
fn doctor_help_shows_subcommands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("check"))
        .stdout(contains("env"))
        .stdout(contains("toolchain"))
        .stdout(contains("network"))
        .stdout(contains("config"))
        .stdout(contains("troubleshoot"))
        .stdout(contains("report"));
}

#[test]
fn doctor_env_shows_environment() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("env");
    cmd.assert()
        .success()
        .stdout(contains("Environment"))
        .stdout(contains("System"));
}

#[test]
fn doctor_toolchain_shows_tools() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("toolchain");
    cmd.assert()
        .success()
        .stdout(contains("Toolchain"))
        .stdout(contains("Rust"));
}

#[test]
fn doctor_config_shows_configuration() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("config");
    cmd.assert()
        .success()
        .stdout(contains("Configuration"));
}

#[test]
fn doctor_troubleshoot_shows_guide() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("troubleshoot");
    cmd.assert()
        .success()
        .stdout(contains("Troubleshooting"))
        .stdout(contains("rpc"))
        .stdout(contains("keys"))
        .stdout(contains("builds"));
}

#[test]
fn doctor_troubleshoot_rpc_shows_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("troubleshoot").arg("--issue").arg("rpc");
    cmd.assert()
        .success()
        .stdout(contains("RPC"));
}

#[test]
fn doctor_check_help_shows_options() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("doctor").arg("check").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("verbose"))
        .stdout(contains("export"));
}

// ============================================================================
// Quickstart Command Tests (Tutorials)
// ============================================================================

#[test]
fn quickstart_help_shows_tutorials() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("begin"))
        .stdout(contains("wallet"))
        .stdout(contains("confidential"))
        .stdout(contains("compression"))
        .stdout(contains("circuits"));
}

#[test]
fn quickstart_list_shows_all_tutorials() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("list");
    cmd.assert()
        .success()
        .stdout(contains("Tutorial"))
        .stdout(contains("Getting Started"))
        .stdout(contains("Wallet"));
}

#[test]
fn quickstart_begin_shows_intro() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("begin");
    cmd.assert()
        .success()
        .stdout(contains("Getting Started"))
        .stdout(contains("SolPrivacy"));
}

#[test]
fn quickstart_wallet_shows_tutorial() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("wallet");
    cmd.assert()
        .success()
        .stdout(contains("Wallet"))
        .stdout(contains("Creating"));
}

#[test]
fn quickstart_confidential_shows_tutorial() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("confidential");
    cmd.assert()
        .success()
        .stdout(contains("Confidential"))
        .stdout(contains("Token-2022"));
}

#[test]
fn quickstart_compression_shows_tutorial() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("compression");
    cmd.assert()
        .success()
        .stdout(contains("Compression"))
        .stdout(contains("Light Protocol"));
}

#[test]
fn quickstart_circuits_shows_tutorial() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("circuits");
    cmd.assert()
        .success()
        .stdout(contains("Noir"))
        .stdout(contains("Circuit"));
}

#[test]
fn quickstart_path_shows_learning_path() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("solprivacy"));
    cmd.arg("quickstart").arg("path");
    cmd.assert()
        .success()
        .stdout(contains("Learning Path"))
        .stdout(contains("essential"));
}