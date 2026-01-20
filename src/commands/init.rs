use clap::Args;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm, theme::ColorfulTheme};
use std::fs;
use std::path::Path;
use crate::config::AppConfig;
use crate::error::Result;

/// Initialize a new privacy-enabled Solana project
#[derive(Args)]
pub struct InitCommand {
    /// Project name
    #[arg(default_value = "my-privacy-app")]
    pub name: String,
    
    /// Privacy stack to use
    #[arg(short, long, value_parser = ["token2022", "privacy-cash", "arcium", "light", "noir"])]
    pub stack: Option<String>,
    
    /// Skip interactive prompts
    #[arg(long)]
    pub no_interactive: bool,
}

impl InitCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{} SolPrivacy Project Initializer", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let (project_name, stack) = if self.no_interactive {
            // Non-interactive mode
            let stack = self.stack.clone().unwrap_or_else(|| "token2022".to_string());
            (self.name.clone(), stack)
        } else {
            // Interactive mode
            self.interactive_setup().await?
        };
        
        // Create project
        self.create_project(&project_name, &stack)?;
        
        Ok(())
    }
    
    async fn interactive_setup(&self) -> Result<(String, String)> {
        let theme = ColorfulTheme::default();
        
        // Step 1: Project name
        println!("  {}:", "Step 1/3".bright_white());
        let project_name: String = Input::with_theme(&theme)
            .with_prompt("  Project name")
            .default(self.name.clone())
            .interact_text()
            .unwrap_or_else(|_| self.name.clone());
        
        println!();
        
        // Step 2: Select privacy stack
        println!("  {}:", "Step 2/3".bright_white());
        let stacks = vec![
            "Token-2022 Confidential Transfer",
            "Privacy Cash (ZK Proofs)",
            "Arcium MXE (Multi-Party Computation)",
            "Light Protocol (ZK Compression)",
            "Noir ZK Circuits (Aztec)",
        ];
        
        let stack_idx = Select::with_theme(&theme)
            .with_prompt("  Select privacy stack")
            .items(&stacks)
            .default(0)
            .interact()
            .unwrap_or(0);
        
        let stack = match stack_idx {
            0 => "token2022",
            1 => "privacy-cash",
            2 => "arcium",
            3 => "light",
            4 => "noir",
            _ => "token2022",
        }.to_string();
        
        println!();
        
        // Step 3: Configure RPC?
        println!("  {}:", "Step 3/3".bright_white());
        let configure_rpc = Confirm::with_theme(&theme)
            .with_prompt("  Configure RPC provider now?")
            .default(true)
            .interact()
            .unwrap_or(false);
        
        if configure_rpc {
            self.configure_rpc_interactive(&theme).await?;
        }
        
        println!();
        
        Ok((project_name, stack))
    }
    
    async fn configure_rpc_interactive(&self, theme: &ColorfulTheme) -> Result<()> {
        let providers = vec![
            "Helius (recommended)",
            "QuickNode",
            "Custom RPC",
            "Skip for now",
        ];
        
        let provider_idx = Select::with_theme(theme)
            .with_prompt("  Select RPC provider")
            .items(&providers)
            .default(0)
            .interact()
            .unwrap_or(3);
        
        match provider_idx {
            0 => {
                // Helius
                let api_key: String = Input::with_theme(theme)
                    .with_prompt("  Helius API Key")
                    .interact_text()
                    .unwrap_or_default();
                
                if !api_key.is_empty() {
                    let mut config = AppConfig::load().unwrap_or_default();
                    config.rpc.active_provider = "helius".to_string();
                    config.rpc.helius_api_key = Some(api_key);
                    config.save()?;
                    println!("  {} Helius RPC configured", "✓".bright_green());
                }
            }
            1 => {
                // QuickNode
                let endpoint: String = Input::with_theme(theme)
                    .with_prompt("  QuickNode Endpoint URL")
                    .interact_text()
                    .unwrap_or_default();
                
                if !endpoint.is_empty() {
                    let mut config = AppConfig::load().unwrap_or_default();
                    config.rpc.active_provider = "quicknode".to_string();
                    config.rpc.quicknode_endpoint = Some(endpoint);
                    config.save()?;
                    println!("  {} QuickNode RPC configured", "✓".bright_green());
                }
            }
            2 => {
                // Custom
                let url: String = Input::with_theme(theme)
                    .with_prompt("  Custom RPC URL")
                    .interact_text()
                    .unwrap_or_default();
                
                if !url.is_empty() {
                    let mut config = AppConfig::load().unwrap_or_default();
                    config.rpc.active_provider = "custom".to_string();
                    config.rpc.custom_rpc_url = Some(url);
                    config.save()?;
                    println!("  {} Custom RPC configured", "✓".bright_green());
                }
            }
            _ => {
                println!("  {} Skipping RPC configuration", "ℹ".bright_blue());
            }
        }
        
        Ok(())
    }
    
    fn create_project(&self, name: &str, stack: &str) -> Result<()> {
        println!("{} Creating project: {}", "→".bright_cyan(), name.bright_white());
        println!();
        
        let base_path = Path::new(name);
        
        // Check if directory exists
        if base_path.exists() {
            println!("{} Directory '{}' already exists!", "✗".bright_red(), name);
            println!("  Use a different name or delete the existing directory.");
            return Ok(());
        }
        
        // Create directories
        fs::create_dir_all(base_path.join("src"))?;
        fs::create_dir_all(base_path.join("keys"))?;
        
        println!("  ├─ Created {}/", name);
        println!("  ├─ Created {}/src/", name);
        println!("  ├─ Created {}/keys/", name);
        
        // Generate files based on stack
        self.generate_cargo_toml(base_path, name, stack)?;
        self.generate_main_rs(base_path, stack)?;
        self.generate_readme(base_path, name, stack)?;
        self.generate_gitignore(base_path)?;
        
        println!();
        println!("{} Project created successfully!", "✓".bright_green());
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", name);
        
        match stack {
            "token2022" => {
                println!("    2. solprivacy keygen auditor -o keys/auditor.json");
                println!("    3. cargo build && cargo run");
            }
            "privacy-cash" => {
                println!("    2. solprivacy fund --private --to <ADDR> --amount 0.1 --dry-run");
                println!("    3. cargo build && cargo run");
            }
            "noir" => {
                println!("    2. noirup -v 1.0.0-beta.13  # Install Noir");
                println!("    3. nargo check && nargo prove");
            }
            _ => {
                println!("    2. cargo build && cargo run");
            }
        }
        
        Ok(())
    }
    
    fn generate_cargo_toml(&self, base_path: &Path, name: &str, stack: &str) -> Result<()> {
        let deps = match stack {
            "token2022" => r#"solana-sdk = "2"
solana-client = "2"
spl-token-2022 = "6""#,
            "privacy-cash" => r#"solana-sdk = "2"
# privacy-cash = "0.2"  # Uncomment when available"#,
            "light" => r#"solana-sdk = "2"
# light-sdk = "0.x"  # Add when ready"#,
            _ => r#"solana-sdk = "2""#,
        };
        
        let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1"
"#, name.replace("-", "_"), deps);
        
        fs::write(base_path.join("Cargo.toml"), cargo_toml)?;
        println!("  ├─ Created Cargo.toml");
        Ok(())
    }
    
    fn generate_main_rs(&self, base_path: &Path, stack: &str) -> Result<()> {
        let main_rs = match stack {
            "token2022" => r#"//! Token-2022 Confidential Transfer Project
//! Generated by SolPrivacy CLI

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Token-2022 Confidential Transfer Demo");
    println!("=====================================\n");
    
    // Step 1: Load keypairs
    println!("Step 1: Generate auditor key with:");
    println!("  solprivacy keygen auditor -o keys/auditor.json\n");
    
    // Step 2: Create confidential mint
    println!("Step 2: Create confidential token");
    println!("  (Requires Token-2022 with ConfidentialTransfer extension)\n");
    
    // Step 3: Transfer
    println!("Step 3: Perform confidential transfer");
    println!("  Debug with: solprivacy debug --decrypt --tx <SIG> --auditor-key keys/auditor.json");
    
    Ok(())
}
"#,
            "privacy-cash" => r#"//! Privacy Cash Integration Project
//! Generated by SolPrivacy CLI

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Privacy Cash Integration Demo");
    println!("=============================\n");
    
    println!("Send funds privately:");
    println!("  solprivacy fund --private --from <KEY> --to <ADDR> --amount 0.5\n");
    
    println!("How it works:");
    println!("  1. Deposit into Privacy Cash pool");
    println!("  2. Generate ZK proof (Groth16)");
    println!("  3. Withdraw to recipient (unlinkable)");
    
    Ok(())
}
"#,
            "noir" => r#"//! Noir ZK Circuit Project
//! Generated by SolPrivacy CLI

// This file is a placeholder. 
// The actual Noir circuit is in the `circuits/` directory.

fn main() {
    println!("Noir ZK Circuit Project");
    println!("=======================\n");
    
    println!("Build and prove:");
    println!("  1. nargo check");
    println!("  2. nargo prove");
    println!("  3. nargo verify");
    println!("\nDeploy to Solana:");
    println!("  sunspot verifier-gen -o verifier/");
}
"#,
            _ => r#"//! Privacy Application
//! Generated by SolPrivacy CLI

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Privacy Application");
    println!("===================\n");
    
    println!("Ready to build privacy features!");
    println!("See: https://github.com/user/solprivacy-cli");
    
    Ok(())
}
"#,
        };
        
        fs::write(base_path.join("src/main.rs"), main_rs)?;
        println!("  ├─ Created src/main.rs");
        Ok(())
    }
    
    fn generate_readme(&self, base_path: &Path, name: &str, stack: &str) -> Result<()> {
        let stack_name = match stack {
            "token2022" => "Token-2022 Confidential Transfer",
            "privacy-cash" => "Privacy Cash",
            "arcium" => "Arcium MXE",
            "light" => "Light Protocol ZK Compression",
            "noir" => "Noir ZK Circuits",
            _ => "Privacy Application",
        };
        
        let readme = format!(r#"# {}

{} project generated by SolPrivacy CLI.

## Quick Start

```bash
cargo build
cargo run
```

## Resources

- [SolPrivacy CLI](https://github.com/user/solprivacy-cli)
- [Solana Privacy Hackathon](https://solana.com/privacyhack)
"#, name, stack_name);
        
        fs::write(base_path.join("README.md"), readme)?;
        println!("  ├─ Created README.md");
        Ok(())
    }
    
    fn generate_gitignore(&self, base_path: &Path) -> Result<()> {
        let gitignore = r#"/target
/keys/*.json
.env
"#;
        fs::write(base_path.join(".gitignore"), gitignore)?;
        println!("  └─ Created .gitignore");
        Ok(())
    }
}
