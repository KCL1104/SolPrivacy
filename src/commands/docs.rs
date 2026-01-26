use crate::error::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

/// Quick access to privacy development documentation
#[derive(Args)]
pub struct DocsCommand {
    #[command(subcommand)]
    pub topic: Option<DocsTopic>,
}

#[derive(Subcommand)]
pub enum DocsTopic {
    /// Token-2022 Confidential Transfers documentation
    Token2022,

    /// Light Protocol ZK Compression documentation
    Light,

    /// Arcium MXE documentation
    Arcium,

    /// Noir ZK circuit documentation
    Noir,

    /// Privacy Cash SDK documentation
    PrivacyCash,

    /// Helius RPC documentation
    Helius,

    /// QuickNode documentation
    Quicknode,

    /// Solana Privacy Hackathon resources
    Hackathon,
}

struct DocLink {
    name: &'static str,
    url: &'static str,
    description: &'static str,
}

impl DocsCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.topic {
            Some(topic) => self.open_topic(topic),
            None => self.show_all(),
        }
    }

    fn show_all(&self) -> Result<()> {
        println!("{} Privacy Development Documentation", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        let topics = vec![
            DocLink {
                name: "token2022",
                url: "https://solana.com/docs/tokens/extensions/confidential-transfer",
                description: "Token-2022 Confidential Transfers",
            },
            DocLink {
                name: "light",
                url: "https://docs.lightprotocol.com",
                description: "Light Protocol ZK Compression",
            },
            DocLink {
                name: "arcium",
                url: "https://docs.arcium.com",
                description: "Arcium Multi-Party Execution",
            },
            DocLink {
                name: "noir",
                url: "https://noir-lang.org/docs",
                description: "Noir ZK Circuits (Aztec)",
            },
            DocLink {
                name: "privacy-cash",
                url: "https://docs.privacy.cash",
                description: "Privacy Cash SDK",
            },
            DocLink {
                name: "helius",
                url: "https://docs.helius.dev",
                description: "Helius RPC & APIs",
            },
            DocLink {
                name: "quicknode",
                url: "https://www.quicknode.com/docs/solana",
                description: "QuickNode Solana",
            },
            DocLink {
                name: "hackathon",
                url: "https://solana.com/privacyhack",
                description: "Solana Privacy Hackathon 2025",
            },
        ];

        println!("  {}:", "Available Topics".bright_white());
        println!();

        for doc in &topics {
            println!("  {} {}", "•".bright_cyan(), doc.name.bright_white());
            println!("    {}", doc.description.bright_black());
            println!("    {}", doc.url.bright_blue());
            println!();
        }

        println!("  {}:", "Usage".bright_white());
        println!("    solprivacy docs <topic>     Open documentation in browser");
        println!();
        println!("  {}:", "Examples".bright_white());
        println!("    solprivacy docs token2022");
        println!("    solprivacy docs light");
        println!("    solprivacy docs hackathon");

        Ok(())
    }

    fn open_topic(&self, topic: &DocsTopic) -> Result<()> {
        let (name, url, description) = match topic {
            DocsTopic::Token2022 => (
                "Token-2022 Confidential Transfers",
                "https://solana.com/docs/tokens/extensions/confidential-transfer",
                "Learn about encrypted token transfers using ElGamal encryption",
            ),
            DocsTopic::Light => (
                "Light Protocol",
                "https://docs.lightprotocol.com",
                "ZK Compression for 1000x cheaper state storage",
            ),
            DocsTopic::Arcium => (
                "Arcium MXE",
                "https://docs.arcium.com",
                "Multi-Party Execution Environment for private computation",
            ),
            DocsTopic::Noir => (
                "Noir ZK Circuits",
                "https://noir-lang.org/docs",
                "Write zero-knowledge proofs in a Rust-like language",
            ),
            DocsTopic::PrivacyCash => (
                "Privacy Cash SDK",
                "https://docs.privacy.cash",
                "Private SOL transfers using ZK proofs",
            ),
            DocsTopic::Helius => (
                "Helius",
                "https://docs.helius.dev",
                "Premium Solana RPC and APIs",
            ),
            DocsTopic::Quicknode => (
                "QuickNode",
                "https://www.quicknode.com/docs/solana",
                "Fast and reliable Solana RPC",
            ),
            DocsTopic::Hackathon => (
                "Solana Privacy Hackathon",
                "https://solana.com/privacyhack",
                "Build privacy applications on Solana",
            ),
        };

        println!("{} {}", "→".bright_cyan(), name.bright_white());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        println!("  {}", description);
        println!();
        println!("  URL: {}", url.bright_blue());
        println!();

        // Try to open in browser
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(url).spawn();
            println!("{} Opening in browser...", "✓".bright_green());
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
            println!("{} Opening in browser...", "✓".bright_green());
        }

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", url])
                .spawn();
            println!("{} Opening in browser...", "✓".bright_green());
        }

        Ok(())
    }
}
