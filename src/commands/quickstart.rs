use clap::{Args, Subcommand};
use colored::Colorize;
use crate::error::Result;

/// Interactive tutorials and getting started guides
#[derive(Args)]
pub struct QuickstartCommand {
    #[command(subcommand)]
    pub action: QuickstartAction,
}

#[derive(Subcommand)]
pub enum QuickstartAction {
    /// Show all available tutorials
    List,
    
    /// Getting started with SolPrivacy (recommended first)
    Begin,
    
    /// Tutorial: Create your first wallet
    Wallet,
    
    /// Tutorial: Token-2022 Confidential Transfers
    Confidential,
    
    /// Tutorial: Light Protocol ZK Compression
    Compression,
    
    /// Tutorial: Noir ZK Circuits
    Circuits,
    
    /// Tutorial: Building a privacy-preserving dApp
    Dapp,
    
    /// Tutorial: Compliance integration
    Compliance,
    
    /// Show recommended learning path
    Path,
}

impl QuickstartCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            QuickstartAction::List => self.show_list().await,
            QuickstartAction::Begin => self.tutorial_begin().await,
            QuickstartAction::Wallet => self.tutorial_wallet().await,
            QuickstartAction::Confidential => self.tutorial_confidential().await,
            QuickstartAction::Compression => self.tutorial_compression().await,
            QuickstartAction::Circuits => self.tutorial_circuits().await,
            QuickstartAction::Dapp => self.tutorial_dapp().await,
            QuickstartAction::Compliance => self.tutorial_compliance().await,
            QuickstartAction::Path => self.show_learning_path().await,
        }
    }

    async fn show_list(&self) -> Result<()> {
        println!("{} Available Tutorials", "📚".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        let tutorials = vec![
            ("begin", "Getting Started", "First steps with SolPrivacy", "5 min"),
            ("wallet", "Wallet Setup", "Create and manage wallets", "3 min"),
            ("confidential", "Confidential Transfers", "Token-2022 private transactions", "10 min"),
            ("compression", "ZK Compression", "Light Protocol state compression", "8 min"),
            ("circuits", "Noir Circuits", "Build custom ZK proofs", "15 min"),
            ("dapp", "Privacy dApp", "Build a complete privacy app", "20 min"),
            ("compliance", "Compliance", "Integrate risk assessment", "5 min"),
        ];

        println!("  {}:", "Tutorials".bright_white());
        println!();

        for (cmd, name, desc, time) in tutorials {
            println!("    {} {:.<30} {} ({})", 
                "→".bright_cyan(),
                format!("{} ", name),
                desc.bright_black(),
                time.bright_green()
            );
            println!("      {}", format!("solprivacy quickstart {}", cmd).bright_black());
            println!();
        }

        println!("  {}:", "Recommended Order".bright_white());
        println!("    {} begin → wallet → confidential → compression → circuits", "1.".bright_cyan());
        println!();
        println!("  Run {} for a guided learning path", "solprivacy quickstart path".bright_cyan());

        Ok(())
    }

    async fn tutorial_begin(&self) -> Result<()> {
        println!("{} Getting Started with SolPrivacy", "🚀".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Welcome to SolPrivacy - the Privacy Orchestration Layer for Solana!");
        println!();
        println!("  This CLI helps you build privacy-preserving applications using:");
        println!("    {} Token-2022 Confidential Transfers (encrypted balances)", "•".bright_cyan());
        println!("    {} Light Protocol ZK Compression (400x cheaper accounts)", "•".bright_cyan());
        println!("    {} Noir Circuits (custom zero-knowledge proofs)", "•".bright_cyan());
        println!("    {} Range Protocol (compliance and risk assessment)", "•".bright_cyan());
        println!();

        self.print_section("Step 1: Verify Your Environment");
        println!("    Run the diagnostic tool to check your setup:");
        println!();
        self.print_command("solprivacy doctor check");
        println!();
        println!("    This will verify that all required tools are installed.");
        println!();

        self.print_section("Step 2: Create a Wallet");
        println!("    Create a new development wallet:");
        println!();
        self.print_command("solprivacy wallet new --name dev");
        println!();
        println!("    Get some devnet SOL for testing:");
        println!();
        self.print_command("solprivacy wallet airdrop --name dev --amount 2");
        println!();

        self.print_section("Step 3: Initialize a Project");
        println!("    Create a new privacy-enabled project:");
        println!();
        self.print_command("solprivacy init my-privacy-app --template anchor");
        println!();

        self.print_section("Step 4: Start Local Development");
        println!("    Start the local Solana validator:");
        println!();
        self.print_command("solprivacy dev start");
        println!();

        self.print_next_step("wallet", "Learn about wallet management");

        Ok(())
    }

    async fn tutorial_wallet(&self) -> Result<()> {
        println!("{} Tutorial: Wallet Management", "👛".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  SolPrivacy manages wallets for you, making it easy to work with");
        println!("  multiple accounts during development.");
        println!();

        self.print_section("Creating Wallets");
        println!("    Create a new wallet with a custom name:");
        println!();
        self.print_command("solprivacy wallet new --name alice");
        self.print_command("solprivacy wallet new --name bob");
        println!();

        self.print_section("Listing Wallets");
        println!("    See all your wallets:");
        println!();
        self.print_command("solprivacy wallet list");
        println!();

        self.print_section("Getting Wallet Address");
        println!("    Get the public address of a wallet:");
        println!();
        self.print_command("solprivacy wallet address --name alice");
        println!();

        self.print_section("Checking Balance");
        println!("    Check SOL balance:");
        println!();
        self.print_command("solprivacy wallet balance --name alice");
        println!();

        self.print_section("Requesting Airdrops");
        println!("    Get devnet SOL for testing:");
        println!();
        self.print_command("solprivacy wallet airdrop --name alice --amount 2");
        println!();

        self.print_section("Importing Existing Wallets");
        println!("    Import from a private key file:");
        println!();
        self.print_command("solprivacy wallet import --name imported --file ~/my-keypair.json");
        println!();

        self.print_tip("Always keep your private keys secure. Never share them or commit them to git!");

        self.print_next_step("confidential", "Learn about confidential transfers");

        Ok(())
    }

    async fn tutorial_confidential(&self) -> Result<()> {
        println!("{} Tutorial: Token-2022 Confidential Transfers", "🔐".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Confidential Transfers encrypt token balances and transfer amounts");
        println!("  using ElGamal encryption and zero-knowledge proofs.");
        println!();

        self.print_section("How It Works");
        println!("    {} Public balance → encrypted \"pending\" balance (deposit)", "1.".bright_cyan());
        println!("    {} Pending balance → usable \"available\" balance (apply)", "2.".bright_cyan());
        println!("    {} Transfer between encrypted balances (transfer)", "3.".bright_cyan());
        println!("    {} Encrypted balance → public balance (withdraw)", "4.".bright_cyan());
        println!();

        self.print_section("Step 1: Generate ElGamal Keys");
        println!("    Each account needs an ElGamal keypair for encryption:");
        println!();
        self.print_command("solprivacy keygen elgamal --output alice-elgamal.json");
        self.print_command("solprivacy keygen elgamal --output bob-elgamal.json");
        println!();

        self.print_section("Step 2: Create a Confidential Token");
        println!("    Create a Token-2022 mint with confidential transfer extension:");
        println!();
        self.print_command("solprivacy mint create --confidential --decimals 9 --name \"Private Token\"");
        println!();

        self.print_section("Step 3: Configure Accounts");
        println!("    Enable confidential transfers on token accounts:");
        println!();
        self.print_command("solprivacy confidential configure --account <TOKEN_ACCOUNT> --elgamal alice-elgamal.json");
        println!();

        self.print_section("Step 4: Deposit to Confidential Balance");
        println!("    Move tokens from public to encrypted balance:");
        println!();
        self.print_command("solprivacy confidential deposit --account <ACCOUNT> --mint <MINT> --amount 1000 --decimals 9");
        println!();

        self.print_section("Step 5: Apply Pending Balance");
        println!("    Make pending balance available for transfers:");
        println!();
        self.print_command("solprivacy confidential apply --account <ACCOUNT> --mint <MINT>");
        println!();

        self.print_section("Step 6: Confidential Transfer");
        println!("    Transfer with encrypted amounts:");
        println!();
        self.print_command("solprivacy confidential transfer --mint <MINT> --to <RECIPIENT> --amount 500");
        println!();

        self.print_section("Step 7: Withdraw");
        println!("    Return encrypted balance to public:");
        println!();
        self.print_command("solprivacy confidential withdraw --account <ACCOUNT> --mint <MINT> --amount 500 --decimals 9");
        println!();

        self.print_tip("Use --dry-run on any command to see what it will do without executing");

        self.print_next_step("compression", "Learn about ZK compression with Light Protocol");

        Ok(())
    }

    async fn tutorial_compression(&self) -> Result<()> {
        println!("{} Tutorial: Light Protocol ZK Compression", "📦".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Light Protocol uses zero-knowledge proofs to compress Solana state,");
        println!("  reducing account costs by ~400x while maintaining security.");
        println!();

        self.print_section("Cost Comparison");
        println!("    Regular account:     ~890,000 lamports (~$0.13)");
        println!("    Compressed account:  ~5,000 lamports (~$0.0007)");
        println!("    Savings:             {} per account", "99.4%".bright_green());
        println!();

        self.print_section("How It Works");
        println!("    {} State is stored in Merkle trees instead of individual accounts", "•".bright_cyan());
        println!("    {} ZK proofs verify state transitions", "•".bright_cyan());
        println!("    {} Photon indexer tracks compressed state", "•".bright_cyan());
        println!("    {} Same security guarantees as regular accounts", "•".bright_cyan());
        println!();

        self.print_section("Step 1: Setup Light Protocol");
        println!("    View setup instructions:");
        println!();
        self.print_command("solprivacy light setup");
        println!();

        self.print_section("Step 2: Configure Photon RPC");
        println!("    Set up the Photon indexer endpoint:");
        println!();
        self.print_command("solprivacy light config --url https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
        println!();

        self.print_section("Step 3: Check Indexer Health");
        println!("    Verify connection to Photon:");
        println!();
        self.print_command("solprivacy light health");
        println!();

        self.print_section("Step 4: Query Compressed Accounts");
        println!("    Find compressed accounts by owner:");
        println!();
        self.print_command("solprivacy light accounts --owner <PUBKEY>");
        println!();

        self.print_section("Step 5: Query Compressed Token Balances");
        println!("    Check compressed token balances:");
        println!();
        self.print_command("solprivacy light balance --owner <PUBKEY> --mint <MINT>");
        println!();

        self.print_section("Step 6: Compare Costs");
        println!("    See cost savings for your use case:");
        println!();
        self.print_command("solprivacy light compare");
        println!();

        self.print_tip("Light Protocol is ideal for NFT collections, airdrops, and any app with many accounts");

        self.print_next_step("circuits", "Learn about custom ZK circuits with Noir");

        Ok(())
    }

    async fn tutorial_circuits(&self) -> Result<()> {
        println!("{} Tutorial: Noir ZK Circuits", "🔮".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Noir is a domain-specific language for writing zero-knowledge circuits.");
        println!("  Use it to build custom privacy features for your dApp.");
        println!();

        self.print_section("Available Circuit Templates");
        println!("    {} - Prove ownership without revealing secret", "ownership".bright_cyan());
        println!("    {} - Prove inclusion in a set (whitelist/blacklist)", "merkle".bright_cyan());
        println!("    {} - Verify signatures anonymously", "signature".bright_cyan());
        println!("    {} - Prove value is in range (compliance)", "range".bright_cyan());
        println!("    {} - Prove sufficient balance (solvency)", "balance".bright_cyan());
        println!("    {} - Prevent double-spending (voting)", "nullifier".bright_cyan());
        println!("    {} - Complete private payment circuit", "privacy-transfer".bright_cyan());
        println!();

        self.print_section("Step 1: Install Noir");
        println!("    Follow the installation guide:");
        println!();
        self.print_command("solprivacy zk setup");
        println!();

        self.print_section("Step 2: Initialize a Circuit");
        println!("    Create a new circuit from a template:");
        println!();
        self.print_command("solprivacy zk init --circuit ownership --output ./my-circuit");
        println!();
        println!("    This creates:");
        println!("      {} src/main.nr - The circuit code", "•".bright_black());
        println!("      {} Nargo.toml - Project configuration", "•".bright_black());
        println!("      {} Prover.toml - Private inputs", "•".bright_black());
        println!("      {} Verifier.toml - Public inputs", "•".bright_black());
        println!();

        self.print_section("Step 3: Understand the Circuit");
        println!("    Example ownership proof circuit:");
        println!();
        println!("    {}",  "fn main(secret: Field, pub commitment: Field) {".bright_black());
        println!("    {}",  "    let computed = std::hash::poseidon::bn254::hash_1([secret]);".bright_black());
        println!("    {}",  "    assert(computed == commitment);".bright_black());
        println!("    {}",  "}".bright_black());
        println!();
        println!("    This proves you know a secret that hashes to the commitment,");
        println!("    without revealing the secret itself.");
        println!();

        self.print_section("Step 4: Compile the Circuit");
        println!("    Check syntax and compile:");
        println!();
        self.print_command("solprivacy zk compile --path ./my-circuit");
        println!();

        self.print_section("Step 5: Generate a Proof");
        println!("    Edit Prover.toml with your inputs, then:");
        println!();
        self.print_command("solprivacy zk prove --path ./my-circuit --inputs Prover.toml");
        println!();

        self.print_section("Step 6: Verify the Proof");
        println!("    Verify locally:");
        println!();
        self.print_command("solprivacy zk verify --path ./my-circuit");
        println!();

        self.print_section("Step 7: Deploy to Solana (Advanced)");
        println!("    Use Sunspot to generate a Solana verifier program:");
        println!();
        self.print_command("solprivacy zk solana install");
        self.print_command("solprivacy zk solana setup --path ./my-circuit");
        self.print_command("solprivacy zk solana verifier --path ./my-circuit --output ./verifier");
        println!();

        self.print_tip("Start with the 'ownership' template - it's the simplest way to learn!");

        self.print_next_step("dapp", "Build a complete privacy dApp");

        Ok(())
    }

    async fn tutorial_dapp(&self) -> Result<()> {
        println!("{} Tutorial: Building a Privacy dApp", "🏗️".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Let's build a complete privacy-preserving application that combines");
        println!("  confidential transfers, compression, and ZK proofs.");
        println!();

        self.print_section("Architecture Overview");
        println!();
        println!("    ┌─────────────────────────────────────────────────────────┐");
        println!("    │                      Your dApp                          │");
        println!("    ├─────────────────────────────────────────────────────────┤");
        println!("    │  Frontend (React/Next.js)                               │");
        println!("    │    └─ Wallet adapter, proof generation                  │");
        println!("    ├─────────────────────────────────────────────────────────┤");
        println!("    │  Solana Programs                                        │");
        println!("    │    ├─ Token-2022 (Confidential Transfers)               │");
        println!("    │    ├─ Light Protocol (Compressed State)                 │");
        println!("    │    └─ Noir Verifier (Custom ZK Proofs)                  │");
        println!("    ├─────────────────────────────────────────────────────────┤");
        println!("    │  Infrastructure                                         │");
        println!("    │    ├─ Helius RPC (DAS API, Priority Fees)               │");
        println!("    │    ├─ Photon Indexer (Compression)                      │");
        println!("    │    └─ Range Protocol (Compliance)                       │");
        println!("    └─────────────────────────────────────────────────────────┘");
        println!();

        self.print_section("Step 1: Project Setup");
        println!("    Initialize a new Anchor project:");
        println!();
        self.print_command("solprivacy init privacy-dapp --template anchor");
        self.print_command("cd privacy-dapp");
        println!();

        self.print_section("Step 2: Configure Providers");
        println!("    Set up Helius for enhanced RPC:");
        println!();
        self.print_command("solprivacy config provider helius --api-key YOUR_KEY");
        println!();
        println!("    Configure compliance checking:");
        println!();
        self.print_command("solprivacy compliance config --api-key YOUR_RANGE_KEY");
        println!();

        self.print_section("Step 3: Create Confidential Token");
        println!("    Create your privacy token:");
        println!();
        self.print_command("solprivacy mint create --confidential --decimals 6 --name \"PrivacyCoin\"");
        println!();

        self.print_section("Step 4: Add ZK Verification");
        println!("    Create a custom circuit for your use case:");
        println!();
        self.print_command("solprivacy zk init --circuit balance --output ./circuits/balance-proof");
        println!();

        self.print_section("Step 5: Implement Compliance");
        println!("    Add compliance checks to your app:");
        println!();
        println!("    {}:", "In your transaction flow".bright_white());
        println!("    {} Check sender risk score", "1.".bright_black());
        println!("    {} Check recipient risk score", "2.".bright_black());
        println!("    {} If both pass, proceed with transfer", "3.".bright_black());
        println!("    {} Log compliance check for audit", "4.".bright_black());
        println!();
        self.print_command("solprivacy compliance check --address <SENDER>");
        println!();

        self.print_section("Step 6: Use Compression for Scale");
        println!("    For high-volume operations, use compressed accounts:");
        println!();
        self.print_command("solprivacy light create-mint --name \"CompressedToken\"");
        println!();

        self.print_section("Step 7: Frontend Integration");
        println!("    Install the SDKs in your frontend:");
        println!();
        println!("    {}",  "npm install @solana/web3.js @solana/spl-token".bright_black());
        println!("    {}",  "npm install @lightprotocol/stateless.js".bright_black());
        println!("    {}",  "npm install @noir-lang/noir_js".bright_black());
        println!();

        self.print_section("Example: Private Payment Flow");
        println!();
        println!("    {}:", "User wants to send private payment".bright_white());
        println!("    {} Frontend generates ZK proof of sufficient balance", "1.".bright_cyan());
        println!("    {} Compliance API checks recipient", "2.".bright_cyan());
        println!("    {} Confidential transfer executes on-chain", "3.".bright_cyan());
        println!("    {} Recipient's balance updated (encrypted)", "4.".bright_cyan());
        println!("    {} Transaction logged (amounts hidden)", "5.".bright_cyan());
        println!();

        self.print_tip("Clone example projects with: solprivacy examples clone privacy-payment");

        self.print_next_step("compliance", "Learn about compliance integration");

        Ok(())
    }

    async fn tutorial_compliance(&self) -> Result<()> {
        println!("{} Tutorial: Compliance Integration", "🛡️".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Range Protocol provides real-time risk intelligence for Solana addresses.");
        println!("  Integrate compliance checks to build trustworthy privacy applications.");
        println!();

        self.print_section("Why Compliance Matters");
        println!("    {} Prevent interaction with sanctioned addresses", "•".bright_cyan());
        println!("    {} Detect stolen/hacked funds", "•".bright_cyan());
        println!("    {} Meet regulatory requirements", "•".bright_cyan());
        println!("    {} Build user trust", "•".bright_cyan());
        println!();

        self.print_section("Risk Score Scale");
        println!();
        println!("    {} {} - Clean address, no concerns", "0-2".bright_green(), "Low Risk".bright_green());
        println!("    {} {} - Some activity flags, monitor", "3-5".bright_yellow(), "Medium Risk".bright_yellow());
        println!("    {} {} - Significant concerns, caution", "6-8".bright_red(), "High Risk".bright_red());
        println!("    {} {} - Known malicious, block", "9-10".bright_red().bold(), "Critical Risk".bright_red().bold());
        println!();

        self.print_section("Step 1: Get API Access");
        println!("    Sign up at Range Protocol:");
        println!();
        println!("    {}",  "https://range.org".bright_black());
        println!();

        self.print_section("Step 2: Configure API Key");
        println!("    Set your Range API key:");
        println!();
        self.print_command("solprivacy compliance config --api-key YOUR_KEY");
        println!();

        self.print_section("Step 3: Check Single Address");
        println!("    Check risk score for an address:");
        println!();
        self.print_command("solprivacy compliance check --address <PUBKEY>");
        println!();

        self.print_section("Step 4: Batch Checking");
        println!("    Check multiple addresses at once:");
        println!();
        self.print_command("solprivacy compliance batch --addresses \"addr1,addr2,addr3\"");
        println!();

        self.print_section("Step 5: Transaction Compliance");
        println!("    Validate a transaction:");
        println!();
        self.print_command("solprivacy compliance transaction --signature <TX_SIG>");
        println!();

        self.print_section("Integration Pattern");
        println!();
        println!("    {}:", "Before processing a transfer".bright_white());
        println!();
        println!("    {}",  "// 1. Check sender".bright_black());
        println!("    {}",  "let sender_risk = check_risk(sender_address);".bright_black());
        println!("    {}",  "if sender_risk.score > 5 {".bright_black());
        println!("    {}",  "    return Err(\"Sender risk too high\");".bright_black());
        println!("    {}",  "}".bright_black());
        println!();
        println!("    {}",  "// 2. Check recipient".bright_black());
        println!("    {}",  "let recipient_risk = check_risk(recipient_address);".bright_black());
        println!("    {}",  "if recipient_risk.score > 7 {".bright_black());
        println!("    {}",  "    return Err(\"Recipient flagged\");".bright_black());
        println!("    {}",  "}".bright_black());
        println!();
        println!("    {}",  "// 3. Proceed with transfer".bright_black());
        println!("    {}",  "execute_confidential_transfer(...);".bright_black());
        println!();

        self.print_tip("Privacy and compliance can coexist - check addresses without revealing amounts!");

        println!();
        println!("{}", "─".repeat(60).bright_black());
        println!();
        println!("  {} You've completed all tutorials!", "🎉".bright_green());
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    {} Explore example projects: {}", "•".bright_cyan(), "solprivacy examples list".bright_black());
        println!("    {} Read the docs: {}", "•".bright_cyan(), "solprivacy docs".bright_black());
        println!("    {} Join the community: {}", "•".bright_cyan(), "https://discord.gg/solprivacy".bright_black());

        Ok(())
    }

    async fn show_learning_path(&self) -> Result<()> {
        println!("{} Recommended Learning Path", "🗺️".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        println!("  Follow this path to master privacy development on Solana:");
        println!();

        let path = vec![
            ("1", "begin", "Getting Started", "Understand SolPrivacy basics", true),
            ("2", "wallet", "Wallet Management", "Create and manage wallets", true),
            ("3", "confidential", "Confidential Transfers", "Learn Token-2022 CT", true),
            ("4", "compression", "ZK Compression", "Master Light Protocol", false),
            ("5", "circuits", "Noir Circuits", "Build custom ZK proofs", false),
            ("6", "compliance", "Compliance", "Add risk assessment", false),
            ("7", "dapp", "Privacy dApp", "Build complete app", false),
        ];

        for (num, cmd, name, desc, is_essential) in path {
            let marker = if is_essential { "●".bright_green() } else { "○".bright_black() };
            let essential = if is_essential { "(essential)" } else { "(advanced)" };
            
            println!("    {} {} {} {}", 
                format!("{}.", num).bright_cyan(),
                marker,
                name.bright_white(),
                essential.bright_black()
            );
            println!("       {}", desc.bright_black());
            println!("       {}", format!("solprivacy quickstart {}", cmd).bright_black());
            println!();
        }

        println!("  {}:", "Time Investment".bright_white());
        println!("    Essential tutorials: ~20 minutes");
        println!("    Advanced tutorials:  ~45 minutes");
        println!("    Total:               ~1 hour");
        println!();

        println!("  Start with: {}", "solprivacy quickstart begin".bright_cyan());

        Ok(())
    }

    fn print_section(&self, title: &str) {
        println!("  {}:", title.bright_white());
    }

    fn print_command(&self, cmd: &str) {
        println!("    {} {}", "$".bright_green(), cmd.bright_cyan());
    }

    fn print_tip(&self, tip: &str) {
        println!();
        println!("  {} {}", "💡 Tip:".bright_yellow(), tip);
    }

    fn print_next_step(&self, cmd: &str, desc: &str) {
        println!();
        println!("{}", "─".repeat(60).bright_black());
        println!();
        println!("  {}:", "Next Step".bright_white());
        println!("    {} - {}", desc, format!("solprivacy quickstart {}", cmd).bright_cyan());
    }
}
