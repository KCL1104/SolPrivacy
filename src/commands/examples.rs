use crate::error::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Clone and manage example projects
#[derive(Args)]
pub struct ExamplesCommand {
    #[command(subcommand)]
    pub action: ExamplesAction,
}

#[derive(Subcommand)]
pub enum ExamplesAction {
    /// List available example projects
    List,

    /// Clone an example project
    Clone {
        /// Example name to clone
        #[arg()]
        name: String,

        /// Output directory (defaults to example name)
        #[arg(short, long)]
        output: Option<String>,
    },
}

struct Example {
    name: &'static str,
    description: &'static str,
    repo: &'static str,
    path: Option<&'static str>,
}

impl ExamplesCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ExamplesAction::List => self.list_examples(),
            ExamplesAction::Clone { name, output } => self.clone_example(name, output.as_deref()),
        }
    }

    fn get_examples() -> Vec<Example> {
        vec![
            Example {
                name: "noir-solana",
                description: "Noir ZK proofs verified on Solana via Sunspot",
                repo: "https://github.com/solana-foundation/noir-examples",
                path: None,
            },
            Example {
                name: "token2022-confidential",
                description: "Token-2022 Confidential Transfer example",
                repo: "https://github.com/solana-labs/solana-program-library",
                path: Some("token/program-2022/src/extension/confidential_transfer"),
            },
            Example {
                name: "light-compressed-token",
                description: "Light Protocol compressed token example",
                repo: "https://github.com/Lightprotocol/light-protocol",
                path: Some("examples"),
            },
            Example {
                name: "anchor-privacy",
                description: "Anchor program with privacy features",
                repo: "https://github.com/coral-xyz/anchor",
                path: Some("examples"),
            },
        ]
    }

    fn list_examples(&self) -> Result<()> {
        println!("{} Available Example Projects", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        let examples = Self::get_examples();

        for example in &examples {
            println!("  {} {}", "•".bright_cyan(), example.name.bright_white());
            println!("    {}", example.description);
            println!("    {}", example.repo.bright_blue());
            println!();
        }

        println!("  {}:", "Usage".bright_white());
        println!("    solprivacy examples clone <name>");
        println!();
        println!("  {}:", "Example".bright_white());
        println!("    solprivacy examples clone noir-solana");
        println!("    solprivacy examples clone token2022-confidential --output my-project");

        Ok(())
    }

    fn clone_example(&self, name: &str, output: Option<&str>) -> Result<()> {
        println!("{} Clone Example Project", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let examples = Self::get_examples();

        let example = match examples.iter().find(|e| e.name == name) {
            Some(e) => e,
            None => {
                println!("{} Unknown example: {}", "✗".bright_red(), name);
                println!();
                println!("  Available examples:");
                for ex in &examples {
                    println!("    • {}", ex.name);
                }
                return Ok(());
            }
        };

        let output_dir = output.unwrap_or(example.name);

        if Path::new(output_dir).exists() {
            println!(
                "{} Directory '{}' already exists!",
                "✗".bright_red(),
                output_dir
            );
            return Ok(());
        }

        println!("  {}:", "Example".bright_white());
        println!("  ├─ Name: {}", example.name.bright_cyan());
        println!("  ├─ Description: {}", example.description);
        println!("  └─ Output: {}", output_dir);
        println!();

        // Check if git is available
        if Command::new("git").arg("--version").output().is_err() {
            println!("{} Git is not installed!", "✗".bright_red());
            println!("  Install git and try again.");
            return Ok(());
        }

        println!("{} Cloning repository...", "→".bright_cyan());

        // Clone the repository
        let clone_result = if let Some(subpath) = example.path {
            // Sparse checkout for specific path
            self.sparse_clone(example.repo, output_dir, subpath)
        } else {
            // Full clone
            Command::new("git")
                .args(["clone", "--depth", "1", example.repo, output_dir])
                .status()
        };

        match clone_result {
            Ok(status) if status.success() => {
                println!("{} Example cloned successfully!", "✓".bright_green());
                println!();
                println!("  {}:", "Next Steps".bright_white());
                println!("    1. cd {}", output_dir);

                match example.name {
                    "noir-solana" => {
                        println!("    2. nargo check");
                        println!("    3. nargo prove");
                    }
                    "token2022-confidential" => {
                        println!("    2. cargo build");
                        println!("    3. Read the README.md");
                    }
                    "light-compressed-token" => {
                        println!("    2. npm install");
                        println!("    3. npm run test");
                    }
                    _ => {
                        println!("    2. Read the README.md");
                        println!("    3. cargo build (or npm install)");
                    }
                }
            }
            Ok(_) => {
                println!("{} Clone failed", "✗".bright_red());
                println!("  Try cloning manually: git clone {}", example.repo);
            }
            Err(e) => {
                println!("{} Failed to run git: {}", "✗".bright_red(), e);
            }
        }

        Ok(())
    }

    fn sparse_clone(
        &self,
        repo: &str,
        output_dir: &str,
        subpath: &str,
    ) -> std::io::Result<std::process::ExitStatus> {
        // For sparse checkout, we do a regular clone with depth 1
        // In practice, for simplicity we just do a full shallow clone
        // and inform the user about the relevant path

        let status = Command::new("git")
            .args(["clone", "--depth", "1", repo, output_dir])
            .status()?;

        if status.success() {
            println!();
            println!("  {} The relevant code is in:", "ℹ".bright_blue());
            println!("    {}/{}", output_dir, subpath);
        }

        Ok(status)
    }
}
