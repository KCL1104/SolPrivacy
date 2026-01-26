use clap::Args;
use colored::Colorize;
use crate::error::Result;
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};
use std::process::Command;

/// Interactive privacy workflow wizard
#[derive(Args)]
pub struct WizardCommand {
}

impl WizardCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{} Interactive Privacy Wizard", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!("This wizard will guide you through the Confidential Transfer lifecycle.");
        println!();

        let options = &[
            "1. Generate Privacy Keys (ElGamal + Auditor)",
            "2. Create Confidential Token",
            "3. Configure Account for Privacy",
            "4. Deposit Public Tokens (Public -> Private)",
            "5. Send Confidential Tokens (Private -> Private)",
            "6. Audit Transactions (Compliance)",
            "Exit"
        ];

        loop {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an action")
                .default(0)
                .items(&options[..])
                .interact()
                .unwrap();

            match selection {
                0 => self.step_keys().await?,
                1 => self.step_mint().await?,
                2 => self.step_configure().await?,
                3 => self.step_deposit().await?,
                4 => self.step_transfer().await?,
                5 => self.step_audit().await?,
                _ => break,
            }
            println!();
        }

        Ok(())
    }

    async fn step_keys(&self) -> Result<()> {
        println!("\n{} Step 1: Key Generation", "→".bright_cyan());
        
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Generate new ElGamal Keypair?")
            .default(true)
            .interact()
            .unwrap()
        {
            let path: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Output path")
                .default("keys/elgamal.json".to_string())
                .interact_text()
                .unwrap();
                
            self.run_cmd("keygen", &["elgamal", "-o", &path])?;
        }

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Generate new Auditor Keypair?")
            .default(true)
            .interact()
            .unwrap()
        {
            let path: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Output path")
                .default("keys/auditor.json".to_string())
                .interact_text()
                .unwrap();
                
            self.run_cmd("keygen", &["auditor", "-o", &path])?;
        }
        
        Ok(())
    }

    async fn step_mint(&self) -> Result<()> {
        println!("\n{} Step 2: Create Confidential Token", "→".bright_cyan());
        
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Token Name")
            .default("PrivateToken".to_string())
            .interact_text()
            .unwrap();
            
        let symbol: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Token Symbol")
            .default("PRV".to_string())
            .interact_text()
            .unwrap();
            
        let auditor: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Auditor Key Path (optional)")
            .default("keys/auditor.json".to_string())
            .allow_empty(true)
            .interact_text()
            .unwrap();
            
        let mut args = vec!["mint", "create", "--name", &name, "--symbol", &symbol, "--supply", "1000000"];
        if !auditor.is_empty() {
             args.push("--auditor");
             args.push(&auditor);
        }
        
        self.run_cmd("mint", &args[1..])?; // Wrapper adds 'mint'
        // Just call logic directly
        
        // Actually self.run_cmd runs "solprivacy <args>" as a subprocess? 
        // No, I'll direct call the binary or just print. 
        // For robustness in this wizard within the same binary, shelling out to own binary is safest 
        // to avoid async runtime issues or complex struct construction from string inputs.
        
        Ok(())
    }

    async fn step_configure(&self) -> Result<()> {
        println!("\n{} Step 3: Configure Account", "→".bright_cyan());
        
        let account: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Token Account (ATA)")
            .interact_text()
            .unwrap();

        let mint: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Mint Address")
            .interact_text()
            .unwrap();

        let elgamal: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("ElGamal Key Path")
            .default("keys/elgamal.json".to_string())
            .interact_text()
            .unwrap();
            
        self.run_cmd("confidential", &["configure", "--account", &account, "--mint", &mint, "--elgamal-keypair", &elgamal])?;
        
        Ok(())
    }

    async fn step_deposit(&self) -> Result<()> {
        println!("\n{} Step 4: Deposit (Public -> Private)", "→".bright_cyan());
        
        let account: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Token Account (ATA)")
            .interact_text()
            .unwrap();

        let mint: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Mint Address")
            .interact_text()
            .unwrap();
            
        let amount: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Amount")
            .default("100".to_string())
            .interact_text()
            .unwrap();

        self.run_cmd("confidential", &["deposit", "--account", &account, "--mint", &mint, "--amount", &amount])?;
        
        println!("{} Don't forget to apply pending balance!", "ℹ".bright_blue());
        if Confirm::with_theme(&ColorfulTheme::default())
             .with_prompt("Apply pending balance now?")
             .default(true)
             .interact()
             .unwrap() 
        {
             self.run_cmd("confidential", &["apply", "--account", &account])?;
        }
        
        Ok(())
    }

    async fn step_transfer(&self) -> Result<()> {
        println!("\n{} Step 5: Confidential Transfer", "→".bright_cyan());
        // Simplified for wizard
        println!("This step requires ZK proofs. Ensure you have the ElGamal key.");
        
        let mint: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Mint Address")
            .interact_text()
            .unwrap();
            
        let to: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Recipient")
            .interact_text()
            .unwrap();

        let amount: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Amount")
            .interact_text()
            .unwrap();
            
        self.run_cmd("transfer", &["--confidential", "--mint", &mint, "--to", &to, "--amount", &amount])?;
        
        Ok(())
    }

    async fn step_audit(&self) -> Result<()> {
        println!("\n{} Step 6: Compliance Audit", "→".bright_cyan());
        
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Auditor Key Path")
            .default("keys/auditor.json".to_string())
            .interact_text()
            .unwrap();
            
        self.run_cmd("compliance", &["audit", "--keypair", &key])?;
        
        Ok(())
    }

    fn run_cmd(&self, subcommand: &str, args: &[&str]) -> Result<()> {
        // We shell out to the current executable to ensure clean context
        let exe = std::env::current_exe()?;
        let mut cmd = Command::new(exe);
        cmd.arg(subcommand);
        cmd.args(args);
        
        println!("{} Running: solprivacy {} {}", "→".bright_cyan(), subcommand, args.join(" "));
        let status = cmd.status()?;
        
        if !status.success() {
            println!("{} Command failed!", "✗".bright_red());
        }
        
        Ok(())
    }
}
