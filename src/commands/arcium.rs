use crate::error::{Result, SolPrivacyError};
use clap::{Args, Subcommand};
use colored::Colorize;
use std::process::{Command, Stdio};

/// Manage Arcium Confidential Computing nodes and projects
#[derive(Args)]
pub struct ArciumCommand {
    #[command(subcommand)]
    pub action: ArciumAction,
}

#[derive(Subcommand)]
pub enum ArciumAction {
    /// Install Arcium CLI tooling
    Install,

    /// Initialize a new Arcium MXE project
    Init {
        /// Project name
        name: String,
    },

    /// Run a local Arcium node (requires Docker)
    Run {
        /// Docker image to use
        #[arg(long, default_value = "arcium-hq/arx-node:latest")]
        image: String,

        /// Run in background
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop the local Arcium node
    Stop,

    /// Build Arcium project (compile MXE)
    Build,

    /// Test Arcium project (runs local cluster tests)
    Test,

    /// Show node information
    Info,
}

impl ArciumCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ArciumAction::Install => self.install().await,
            ArciumAction::Init { name } => self.init(name).await,
            ArciumAction::Run { image, detach } => self.run_node(image, *detach).await,
            ArciumAction::Stop => self.stop_node().await,
            ArciumAction::Build => self.build().await,
            ArciumAction::Test => self.test().await,
            ArciumAction::Info => self.info().await,
        }
    }

    async fn install(&self) -> Result<()> {
        println!("{} Installing Arcium CLI...", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());

        println!("  Downloading installer from https://install.arcium.com...");

        let status = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSfL https://install.arcium.com | sh")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| SolPrivacyError::Other(format!("Failed to execute installer: {}", e)))?;

        if status.success() {
            println!();
            println!("{} Arcium CLI installed successfully!", "✓".bright_green());
            println!("  Please restart your terminal or source your config.");
        } else {
            println!("{} Installation failed.", "✗".bright_red());
        }

        Ok(())
    }

    async fn init(&self, name: &str) -> Result<()> {
        println!(
            "{} Initializing Arcium Project: {}",
            "→".bright_cyan(),
            name
        );

        // Check if arcium CLI is available
        if self.check_arcium_cli().await {
            let status = Command::new("arcium")
                .arg("init")
                .arg(name)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| SolPrivacyError::Other(format!("Failed to run arcium init: {}", e)))?;

            if status.success() {
                println!("{} Project initialized!", "✓".bright_green());
            }
        } else {
            println!("{} Arcium CLI not found.", "⚠".bright_yellow());
            println!("  Training wheels mode: Creating basic structure manually...");
            // Fallback: Create directory structure manually if CLI missing
            // This ensures the command works even without the binary
            std::fs::create_dir_all(format!("{}/encrypted-ixs", name))?;
            std::fs::write(format!("{}/Arcium.toml", name), "# Arcium Configuration\n")?;
            println!("  CREATED {}/encrypted-ixs", name);
            println!("  CREATED {}/Arcium.toml", name);
            println!(
                "  {} Basic structure created. Install Arcium CLI for full features.",
                "✓".bright_green()
            );
        }

        Ok(())
    }

    async fn run_node(&self, image: &str, detach: bool) -> Result<()> {
        println!("{} Starting Local Arcium Node", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());

        // 1. Check Docker
        if Command::new("docker").arg("--version").output().is_err() {
            println!(
                "{} Docker is not installed or not in PATH.",
                "✗".bright_red()
            );
            println!("  Arcium nodes require Docker to run.");
            return Ok(());
        }

        println!("  Image: {}", image.bright_blue());
        println!(
            "  Mode: {}",
            if detach { "Background" } else { "Foreground" }
        );
        println!();

        // 2. Prepare docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("run");

        if detach {
            cmd.arg("-d");
        }

        cmd.args(["--name", "arcium-node"])
            .args(["--network", "host"])
            .arg("--rm"); // Clean up on exit

        // Mounts (assume current dir has keys/config, otherwise node might auto-gen)
        // For robustness, we don't mount rigid paths unless we know they exist.
        // We rely on the image's default behavior for ephemeral dev nodes.

        cmd.arg(image);

        println!("{} Executing: docker run ...", "→".bright_cyan());

        match cmd.spawn() {
            Ok(mut child) => {
                if !detach {
                    let _ = child.wait();
                } else {
                    println!("{} Node started in background.", "✓".bright_green());
                    println!("  View logs: docker logs -f arcium-node");
                }
            }
            Err(e) => {
                println!(
                    "{} Failed to start Docker container: {}",
                    "✗".bright_red(),
                    e
                );
                println!();
                println!("  Try pulling the image manually:");
                println!("    docker pull {}", image);
            }
        }

        Ok(())
    }

    async fn stop_node(&self) -> Result<()> {
        println!("{} Stopping Arcium Node...", "→".bright_cyan());

        let status = Command::new("docker")
            .args(["stop", "arcium-node"])
            .output();

        match status {
            Ok(_) => println!("{} Node stopped.", "✓".bright_green()),
            Err(_) => println!(
                "{} Failed to stop node (is it running?)",
                "⚠".bright_yellow()
            ),
        }

        Ok(())
    }

    async fn build(&self) -> Result<()> {
        println!("{} Building Arcium Project...", "→".bright_cyan());
        self.run_cli_command("build").await
    }

    async fn test(&self) -> Result<()> {
        println!("{} Running Arcium Tests...", "→".bright_cyan());
        self.run_cli_command("test").await
    }

    async fn info(&self) -> Result<()> {
        println!("{} Arcium Info...", "→".bright_cyan());
        // Try arx-info if available, otherwise just check binary
        self.run_cli_command("arx-info").await
    }

    async fn check_arcium_cli(&self) -> bool {
        Command::new("arcium").arg("--version").output().is_ok()
    }

    async fn run_cli_command(&self, cmd: &str) -> Result<()> {
        if !self.check_arcium_cli().await {
            println!("{} Arcium CLI not installed.", "✗".bright_red());
            println!("  Run: solprivacy arcium install");
            return Ok(());
        }

        let status = Command::new("arcium")
            .arg(cmd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| SolPrivacyError::Other(format!("Failed to run arcium {}: {}", cmd, e)))?;

        if !status.success() {
            // Don't error out hard, just log
            println!("{} Command failed.", "✗".bright_red());
        }

        Ok(())
    }
}
