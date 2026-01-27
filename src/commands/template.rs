use crate::error::Result;
use clap::Args;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Generate project templates
#[derive(Args)]
pub struct TemplateCommand {
    /// Template name
    #[arg(value_parser = ["token2022", "privacy-cash", "arcium", "light"])]
    pub name: String,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: String,
}

impl TemplateCommand {
    /// Generate a new template project
    pub fn run(&self) -> Result<()> {
        match self.name.as_str() {
            "token2022" => self.generate_token2022_template(),
            "privacy-cash" => self.generate_privacy_cash_template(),
            "arcium" => self.generate_arcium_template(),
            "light" => self.generate_light_template(),
            "noir" => self.generate_noir_template(),
            _ => unreachable!(),
        }
    }

    fn generate_token2022_template(&self) -> Result<()> {
        let project_name = "token2022-confidential";
        let base_path = Path::new(&self.output).join(project_name);

        println!(
            "  {}:",
            "Token-2022 Confidential Transfer Template".bright_white()
        );
        println!("  ├─ Creating project structure...");

        // Create directories
        fs::create_dir_all(base_path.join("scripts"))?;
        fs::create_dir_all(base_path.join("keys"))?;
        fs::create_dir_all(base_path.join("src"))?;

        // Generate Cargo.toml
        let cargo_toml = r#"[package]
name = "token2022-confidential"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "1.18"
solana-client = "1.18"
spl-token-2022 = { version = "3.0", features = ["no-entrypoint"] }
spl-associated-token-account = { version = "2.3", features = ["no-entrypoint"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
"#;
        fs::write(base_path.join("Cargo.toml"), cargo_toml)?;
        println!("  ├─ Created Cargo.toml");

        // Generate main.rs
        let main_rs = r#"//! Token-2022 Confidential Transfer Example
//! 
//! This example demonstrates how to:
//! 1. Create a mint with confidential transfer extension
//! 2. Configure auditor keys
//! 3. Perform encrypted transfers

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    pubkey::Pubkey,
};
use spl_token_2022::{
    extension::ExtensionType,
    instruction::{initialize_mint, initialize_account},
    state::Mint,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Token-2022 Confidential Transfer Demo");
    println!("=====================================\n");
    
    // 1. Setup Connection
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    println!("Connected to devnet");

    // 2. Setup Payer
    let payer = Keypair::new();
    println!("Payer: {}", payer.pubkey());
    
    // Request airdrop
    println!("Requesting airdrop...");
    match client.request_airdrop(&payer.pubkey(), 1_000_000_000) {
        Ok(sig) => {
            println!("Airdrop requested: {}", sig);
            // In a real app, await confirmation
        },
        Err(e) => println!("Airdrop failed (might be rate limited): {}", e),
    }

    // 3. Create Mint with Confidential Extension
    let mint = Keypair::new();
    let decimals = 9;
    
    println!("Creating Mint: {}", mint.pubkey());
    
    // Calculate rent
    let mint_len = ExtensionType::try_calculate_account_len::<Mint>(&[
        ExtensionType::ConfidentialTransferMint,
    ])?;
    let rent = client.get_minimum_balance_for_rent_exemption(mint_len)?;

    // Build instruction
    // Note: detailed instruction building omitted for brevity in this template,
    // but this gives you the scaffold to start.
    // You would use:
    // - system_instruction::create_account
    // - spl_token_2022::extension::confidential_transfer::instruction::initialize_mint
    // - spl_token_2022::instruction::initialize_mint

    println!("\nTo complete setup:");
    println!("1. Ensure you have enough SOL");
    println!("2. Run the full creation logic (see solprivacy source for details)");
    
    Ok(())
}
"#;
        fs::write(base_path.join("src/main.rs"), main_rs)?;
        println!("  ├─ Created src/main.rs");

        // Generate README
        let readme = r#"# Token-2022 Confidential Transfer Project

This project demonstrates Token-2022 Confidential Transfers on Solana.

## Features

- Confidential token mint creation
- ElGamal encryption for transfer amounts
- Auditor key configuration

## Quick Start

```bash
# Generate auditor keypair
solprivacy keygen auditor -o keys/auditor.json

# Build and run
cargo build
cargo run
```

## Resources

- [Token-2022 Documentation](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [SolPrivacy CLI](https://github.com/user/solprivacy-cli)
"#;
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");

        // Generate .gitignore
        let gitignore = "/target\n/keys/*.json\n";
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");

        println!();
        println!(
            "{} Template created at: {}",
            "✓".bright_green(),
            base_path.display()
        );
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", project_name);
        println!("    2. solprivacy keygen auditor -o keys/auditor.json");
        println!("    3. cargo build && cargo run");

        Ok(())
    }

    fn generate_arcium_template(&self) -> Result<()> {
        let project_name = "blind-auction";
        let base_path = Path::new(&self.output).join(project_name);

        println!(
            "  {}:",
            "Arcium MXE (Multiparty Execution) Template".bright_white()
        );
        println!("  ├─ Creating project structure...");

        // Create standard Anchor-like structure
        fs::create_dir_all(base_path.join("programs").join("blind-auction").join("src"))?;
        fs::create_dir_all(base_path.join("tests"))?;
        fs::create_dir_all(base_path.join("app"))?;

        // 1. Generate Anchor.toml
        let anchor_toml = r#"[features]
seeds = false
skip-lint = false
[programs.localnet]
blind_auction = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
"#;
        fs::write(base_path.join("Anchor.toml"), anchor_toml)?;
        println!("  ├─ Created Anchor.toml");

        // 2. Generate Cargo.toml (Workspace)
        let workspace_cargo = r#"[workspace]
members = [
    "programs/*"
]

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
"#;
        fs::write(base_path.join("Cargo.toml"), workspace_cargo)?;

        // 3. Generate Program Cargo.toml
        let program_cargo = r#"[package]
name = "blind-auction"
version = "0.1.0"
description = "Blind Auction with Arcium MXE"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "blind_auction"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []

[dependencies]
anchor-lang = "0.29.0"
arcium-anchor = "0.1.0" 
"#;
        fs::write(
            base_path.join("programs/blind-auction/Cargo.toml"),
            program_cargo,
        )?;
        println!("  ├─ Created programs/blind-auction/Cargo.toml");

        // 4. Generate lib.rs with MXE structure (Blind Auction)
            let lib_rs = r#"use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[arcium_program]
pub mod blind_auction {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing Blind Auction...");
        Ok(())
    }

    #[arcium_computation]
    pub fn place_bid(
        ctx: Context<Bid>,
        bid_amount: Encrypted<u64>,
        bidder: Pubkey
    ) -> Result<()> {
        // MXE stores the encrypted bid securely
        // In a real implementation, this would update the secret state
        msg!("Bid received from {}", bidder);
        Ok(())
    }

    #[arcium_computation]
    pub fn resolve_auction(
        ctx: Context<Resolve>,
        bids: Vec<Encrypted<u64>>
    ) -> Result<Encrypted<u64>> {
        // Securely compute the maximum bid without revealing individual bids
        let max_bid = bids.iter().fold(Encrypted::new(0), |max, bid| {
            // max(a, b) logic inside MPC
            arcium::ops::max(&max, bid)
        });
        Ok(max_bid)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Bid {}

#[derive(Accounts)]
pub struct Resolve {}
"#;
        fs::write(base_path.join("programs/blind-auction/src/lib.rs"), lib_rs)?;
        println!("  ├─ Created programs/blind-auction/src/lib.rs");

        // 5. Generate README
        let readme = r#"# Arcium MXE Project

Secure Multiparty Execution Environment on Solana.

## Features

- Confidential computation using MPC/TEE
- Anchor-compatible structure
- Secure input/output handling

## Prerequisites

- [Arcium CLI](https://docs.arcium.com)
- Anchor Framework

## Quick Start

```bash
# Build the program
anchor build

# Deploy to Arcium network
arcium deploy
```

## Resources

- [Arcium Documentation](https://docs.arcium.com)
"#;
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");

        let gitignore = "/target\n/node_modules\n.anchor\n*.tsbuildinfo\n";
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");

        println!();
        println!(
            "{} Template created at: {}",
            "✓".bright_green(),
            base_path.display()
        );
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", project_name);
        println!("    2. Install dependencies: npm install");
        println!("    3. Build: anchor build");
        println!("    4. Deploy: arcium deploy");

        Ok(())
    }

    fn generate_noir_template(&self) -> Result<()> {
        let project_name = "noir-zk-circuit";
        let base_path = Path::new(&self.output).join(project_name);

        println!(
            "  {}:",
            "Noir Zero-Knowledge Circuit Template".bright_white()
        );
        println!("  ├─ Creating project structure...");

        fs::create_dir_all(base_path.join("src"))?;

        // 1. Generate Nargo.toml
        let nargo_toml = r#"[package]
name = "noir_zk_circuit"
type = "bin"
authors = ["SolPrivacy User"]
compiler_version = "0.22.0"

[dependencies]
"#;
        fs::write(base_path.join("Nargo.toml"), nargo_toml)?;
        println!("  ├─ Created Nargo.toml");

        // 2. Generate src/main.nr
        let main_nr = r#"fn main(x: Field, y: pub Field) {
    // Simple constraint: x must not equal y
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);    
    // main(1, 1); // This would fail
}
"#;
        fs::write(base_path.join("src/main.nr"), main_nr)?;
        println!("  ├─ Created src/main.nr");

        // 3. Generate Prover.toml (Example inputs)
        let prover_toml = r#"x = "1"
y = "2"
"#;
        fs::write(base_path.join("Prover.toml"), prover_toml)?;
        println!("  ├─ Created Prover.toml");

        // 4. Generate README
        let readme = r#"# Noir ZK Circuit Project

Zero-Knowledge circuit written in Noir.

## Structure

- `src/main.nr`: Circuit logic
- `Nargo.toml`: Project configuration
- `Prover.toml`: Example inputs

## Quick Start

```bash
# Check circuit
nargo check

# Run tests
nargo test

# Generate proof
nargo prove

# Verify proof
nargo verify
```

## Resources

- [Noir Documentation](https://noir-lang.org)
- [SolPrivacy CLI](https://github.com/user/solprivacy-cli)
"#;
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");

        let gitignore = "/target\n/proofs\n";
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");

        println!();
        println!(
            "{} Template created at: {}",
            "✓".bright_green(),
            base_path.display()
        );
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", project_name);
        println!("    2. Ensure Nargo is installed (check with 'nargo --version')");
        println!("    3. Run: nargo check");

        Ok(())
    }

    fn generate_privacy_cash_template(&self) -> Result<()> {
        let project_name = "privacy-cash-project";
        let base_path = Path::new(&self.output).join(project_name);

        println!("  {}:", "Privacy Cash Project Template".bright_white());
        println!("  ├─ Creating project structure...");

        // Create directories
        fs::create_dir_all(base_path.join("src"))?;
        fs::create_dir_all(base_path.join("circuits"))?;
        fs::create_dir_all(base_path.join("scripts"))?;

        // Generate Cargo.toml
        let cargo_toml = r#"[package]
name = "privacy-cash-project"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "1.18"
solana-client = "1.18"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
# privacy-cash = "0.2" # Add when available
"#;
        fs::write(base_path.join("Cargo.toml"), cargo_toml)?;
        println!("  ├─ Created Cargo.toml");

        // Generate main.rs
        let main_rs = r#"//! Privacy Cash Example
//! 
//! Demonstrates private transfers using ZK proofs.

use anyhow::Result;
use solana_sdk::signature::{Keypair, Signer};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Privacy Cash Demo");
    println!("=================\n");
    
    // 1. Setup
    let user = Keypair::new();
    println!("User: {}", user.pubkey());
    
    // 2. Deposit (Simulated)
    println!("Depositing 1 SOL into privacy pool...");
    // privacy_cash::deposit(&user, 1_000_000_000).await?;
    
    // 3. Generate Proof
    println!("Generating zero-knowledge proof...");
    // let proof = privacy_cash::prove_ownership(...);
    
    // 4. Withdraw to new address
    let recipient = Keypair::new();
    println!("Withdrawing to: {}", recipient.pubkey());
    // privacy_cash::withdraw(&recipient, proof).await?;
    
    Ok(())
}
"#;
        fs::write(base_path.join("src/main.rs"), main_rs)?;
        println!("  ├─ Created src/main.rs");

        // Generate README
        let readme = r#"# Privacy Cash Project

Private SOL transfers using Zero-Knowledge Proofs.

## Features

- Breaks on-chain link between sender and recipient
- Uses Groth16 proofs (compatible with Solana syscalls)
- Relayer network support

## Quick Start

```bash
cargo run
```

## Resources

- [Privacy Cash Docs](https://docs.privacy.cash)
"#;
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");

        let gitignore = "/target\n/keys\n";
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");

        println!();
        println!(
            "{} Template created at: {}",
            "✓".bright_green(),
            base_path.display()
        );

        Ok(())
    }
    fn generate_light_template(&self) -> Result<()> {
        let project_name = "light-protocol-project";
        let base_path = Path::new(&self.output).join(project_name);

        println!(
            "  {}:",
            "Light Protocol ZK Compression Template".bright_white()
        );
        println!("  ├─ Creating project structure...");

        // Create directories
        fs::create_dir_all(base_path.join("src"))?;
        fs::create_dir_all(base_path.join("scripts"))?;

        // Generate Cargo.toml
        let cargo_toml = r#"[package]
name = "light-protocol-project"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "1.18"
solana-client = "1.18"
solana-pubkey = "1.18"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
light-client = "0.18"
"#;
        fs::write(base_path.join("Cargo.toml"), cargo_toml)?;
        println!("  ├─ Created Cargo.toml");

        // Generate main.rs
        let main_rs = r#"//! Light Protocol ZK Compression Example
//! 
//! This example demonstrates:
//! 1. Connecting to Photon RPC
//! 2. Querying compressed accounts
//! 3. Creating a compressed mint (via CLI wrapper)

use anyhow::Result;
use light_client::rpc::{LightClient, LightClientConfig};
use solana_pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Light Protocol ZK Compression Demo");
    println!("===================================\n");
    
    // 1. Setup Connection
    // Note: For devnet, you need a Helius API key or similar provider supporting Photon
    let rpc_url = "https://devnet.helius-rpc.com/?api-key=YOUR_KEY".to_string();
    println!("Connecting to: {}", rpc_url);

    // 2. Initialize Light Client
    let config = LightClientConfig::new(rpc_url.clone(), Some(rpc_url), None);
    
    // Note: This will fail without a valid API key
    match LightClient::new(config).await {
        Ok(client) => {
            println!("Connected to Light Protocol!");
            
            // 3. Query Compressed Balance (Example)
            // Replace with your pubkey
            let owner = Pubkey::from_str("11111111111111111111111111111111").unwrap();
            
            println!("Querying compressed balance for: {}", owner);
            match client.get_compressed_balance_by_owner(&owner, None).await {
                Ok(balance) => println!("Compressed SOL Balance: {}", balance.value),
                Err(e) => println!("Failed to query balance: {}", e),
            }
        },
        Err(e) => {
            println!("Failed to connect (expected if no API key): {}", e);
            println!("\nTo fix:");
            println!("1. Get a Helius API key from https://helius.dev");
            println!("2. Update the rpc_url in src/main.rs");
        }
    }
    
    println!("\nNext Steps:");
    println!("1. Install Light CLI: npm i -g @lightprotocol/zk-compression-cli");
    println!("2. Run: light test-validator (for local dev)");
    
    Ok(())
}
"#;
        fs::write(base_path.join("src/main.rs"), main_rs)?;
        println!("  ├─ Created src/main.rs");

        // Generate README
        let readme = r#"# Light Protocol ZK Compression Project

State compression for 1000x cost reduction on Solana.

## Features

- Native Rust integration with `light-client`
- Query compressed accounts and balances
- Template for ZK compression applications

## Prerequisites

- [Light Protocol CLI](https://github.com/Lightprotocol/light-protocol)
- Helius API Key (for Devnet) or Local Validator

## Quick Start

```bash
# Install dependencies
npm i -g @lightprotocol/zk-compression-cli

# Build
cargo build

# Run
cargo run
```

## Resources

- [Light Protocol Docs](https://docs.lightprotocol.com)
- [SolPrivacy CLI](https://github.com/user/solprivacy-cli)
"#;
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");

        let gitignore = "/target\n";
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");

        println!();
        println!(
            "{} Template created at: {}",
            "✓".bright_green(),
            base_path.display()
        );
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", project_name);
        println!("    2. Edit src/main.rs to add your Helius API Key");
        println!("    3. cargo run");

        Ok(())
    }
}
