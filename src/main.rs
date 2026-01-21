use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod config;
mod error;
mod validation;

use commands::{ConfigCommand, DebugCommand, InitCommand, KeygenCommand, TemplateCommand, FundCommand, MintCommand, ZkCommand, AccountCommand, TransferCommand, WalletCommand, SetupCommand, DevCommand, LightCommand, DocsCommand, ExamplesCommand, ConfidentialCommand, ComplianceCommand, HeliusCommand, DoctorCommand, QuickstartCommand};
use error::Result;

/// SolPrivacy-CLI: Privacy Orchestration Layer for Solana
/// 
/// An all-in-one CLI tool for building privacy-preserving applications on Solana.
/// Integrates Privacy Cash, Token-2022 Confidential Transfers, Light Protocol, and more.
#[derive(Parser)]
#[command(name = "solprivacy")]
#[command(author = "SolPrivacy Team")]
#[command(version)]
#[command(about = "Privacy Orchestration Layer for Solana", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new privacy-enabled Solana project
    Init(InitCommand),
    
    /// Configure RPC providers and environment
    Config(ConfigCommand),
    
    /// Debug and decrypt confidential transactions
    Debug(DebugCommand),
    
    /// Generate cryptographic keys (ElGamal, ZK)
    Keygen(KeygenCommand),
    
    /// Generate project templates
    Template(TemplateCommand),
    
    /// Fund a wallet (with optional private transfer via Privacy Cash)
    Fund(FundCommand),
    
    /// Create and manage Token-2022 confidential tokens
    Mint(MintCommand),
    
    /// Zero-Knowledge proof operations using Noir circuits
    Zk(ZkCommand),
    
    /// Manage token accounts
    Account(AccountCommand),
    
    /// Transfer tokens (supports confidential mode)
    Transfer(TransferCommand),
    
    /// Wallet management (create, import, balance, airdrop)
    Wallet(WalletCommand),
    
    /// Setup and validate privacy development toolchains
    Setup(SetupCommand),
    
    /// Local development environment (start/stop validator)
    Dev(DevCommand),
    
    /// Light Protocol ZK Compression commands
    Light(LightCommand),
    
    /// Quick access to privacy documentation
    Docs(DocsCommand),
    
    /// Clone example projects
    Examples(ExamplesCommand),
    
    /// Confidential transfer operations (configure, deposit, apply, transfer, withdraw)
    Confidential(ConfidentialCommand),
    
    /// Compliance and risk assessment using Range Protocol
    Compliance(ComplianceCommand),
    
    /// Helius API integration (DAS, Priority Fees, Webhooks)
    Helius(HeliusCommand),
    
    /// Diagnostic and troubleshooting tools
    Doctor(DoctorCommand),
    
    /// Interactive tutorials and getting started guides
    Quickstart(QuickstartCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Print banner
    print_banner();
    
    match cli.command {
        Commands::Init(cmd) => cmd.run().await,
        Commands::Config(cmd) => cmd.run().await,
        Commands::Debug(cmd) => cmd.run().await,
        Commands::Keygen(cmd) => cmd.run().await,
        Commands::Template(cmd) => cmd.run().await,
        Commands::Fund(cmd) => cmd.run().await,
        Commands::Mint(cmd) => cmd.run().await,
        Commands::Zk(cmd) => cmd.run().await,
        Commands::Account(cmd) => cmd.run().await,
        Commands::Transfer(cmd) => cmd.run().await,
        Commands::Wallet(cmd) => cmd.run().await,
        Commands::Setup(cmd) => cmd.run().await,
        Commands::Dev(cmd) => cmd.run().await,
        Commands::Light(cmd) => cmd.run().await,
        Commands::Docs(cmd) => cmd.run().await,
        Commands::Examples(cmd) => cmd.run().await,
        Commands::Confidential(cmd) => cmd.run().await,
        Commands::Compliance(cmd) => cmd.run().await,
        Commands::Helius(cmd) => cmd.run().await,
        Commands::Doctor(cmd) => cmd.run().await,
        Commands::Quickstart(cmd) => cmd.run().await,
    }
}

fn print_banner() {
    let banner = r#"
  ____        _ ____       _                       
 / ___|  ___ | |  _ \ _ __(_)_   ____ _  ___ _   _ 
 \___ \ / _ \| | |_) | '__| \ \ / / _` |/ __| | | |
  ___) | (_) | |  __/| |  | |\ V / (_| | (__| |_| |
 |____/ \___/|_|_|   |_|  |_| \_/ \__,_|\___|\__, |
                                             |___/ 
    "#;
    println!("{}", banner.bright_cyan());
    println!("{}", "  Privacy Orchestration Layer for Solana".bright_white());
    println!("{}", "  ─────────────────────────────────────────".bright_black());
    println!();
}
