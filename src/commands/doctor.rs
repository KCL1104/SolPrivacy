use crate::config::AppConfig;
use crate::error::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Comprehensive diagnostic and troubleshooting commands
#[derive(Args)]
pub struct DoctorCommand {
    #[command(subcommand)]
    pub action: DoctorAction,
}

#[derive(Subcommand)]
pub enum DoctorAction {
    /// Run full diagnostic suite
    Check {
        /// Run verbose diagnostics with detailed output
        #[arg(short, long)]
        verbose: bool,

        /// Export diagnostics to JSON file
        #[arg(long)]
        export: Option<String>,
    },

    /// Check environment variables and system configuration
    Env,

    /// Verify all toolchain dependencies
    Toolchain {
        /// Show installation instructions for missing tools
        #[arg(short, long)]
        fix: bool,
    },

    /// Test network connectivity to Solana clusters
    Network {
        /// Test all networks (devnet, testnet, mainnet)
        #[arg(short, long)]
        all: bool,
    },

    /// Validate project configuration files
    Config,

    /// Check for common issues and suggest fixes
    Troubleshoot {
        /// Specific issue to troubleshoot (rpc, keys, builds, proofs)
        #[arg(short, long)]
        issue: Option<String>,
    },

    /// Generate diagnostic report for support
    Report {
        /// Output file path
        #[arg(short, long, default_value = "solprivacy-diagnostic.txt")]
        output: String,
    },
}

impl DoctorCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            DoctorAction::Check { verbose, export } => {
                self.run_full_check(*verbose, export.clone()).await
            }
            DoctorAction::Env => self.check_environment().await,
            DoctorAction::Toolchain { fix } => self.check_toolchain(*fix).await,
            DoctorAction::Network { all } => self.check_network(*all).await,
            DoctorAction::Config => self.check_config().await,
            DoctorAction::Troubleshoot { issue } => self.troubleshoot(issue.clone()).await,
            DoctorAction::Report { output } => self.generate_report(output).await,
        }
    }

    async fn run_full_check(&self, verbose: bool, export: Option<String>) -> Result<()> {
        println!("{} SolPrivacy Diagnostic Suite", "🔍".bright_cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        let mut issues: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Environment Check
        println!("  {} {}", "1.".bright_white(), "Environment".bright_cyan());
        let env_result = self.check_env_quick(verbose);
        if !env_result.0 {
            issues.extend(env_result.1.clone());
        }
        warnings.extend(env_result.2.clone());
        println!();

        // 2. Toolchain Check
        println!("  {} {}", "2.".bright_white(), "Toolchain".bright_cyan());
        let tool_result = self.check_toolchain_quick(verbose);
        if !tool_result.0 {
            issues.extend(tool_result.1.clone());
        }
        warnings.extend(tool_result.2.clone());
        println!();

        // 3. Configuration Check
        println!(
            "  {} {}",
            "3.".bright_white(),
            "Configuration".bright_cyan()
        );
        let config_result = self.check_config_quick(verbose);
        if !config_result.0 {
            issues.extend(config_result.1.clone());
        }
        warnings.extend(config_result.2.clone());
        println!();

        // 4. Network Check
        println!("  {} {}", "4.".bright_white(), "Network".bright_cyan());
        let net_result = self.check_network_quick(verbose).await;
        if !net_result.0 {
            issues.extend(net_result.1.clone());
        }
        warnings.extend(net_result.2.clone());
        println!();

        // Summary
        println!("{}", "═".repeat(60).bright_black());
        println!();

        if issues.is_empty() && warnings.is_empty() {
            println!(
                "{} All checks passed! Your environment is ready.",
                "✓".bright_green()
            );
        } else {
            if !issues.is_empty() {
                println!("{} {} issue(s) found:", "✗".bright_red(), issues.len());
                for issue in &issues {
                    println!("    {} {}", "•".bright_red(), issue);
                }
                println!();
            }

            if !warnings.is_empty() {
                println!("{} {} warning(s):", "⚠".bright_yellow(), warnings.len());
                for warning in &warnings {
                    println!("    {} {}", "•".bright_yellow(), warning);
                }
                println!();
            }

            println!(
                "  Run {} for detailed troubleshooting",
                "solprivacy doctor troubleshoot".bright_cyan()
            );
        }

        // Export if requested
        if let Some(path) = export {
            let report = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "issues": issues,
                "warnings": warnings,
                "passed": issues.is_empty(),
            });
            fs::write(
                &path,
                serde_json::to_string_pretty(&report).unwrap_or_default(),
            )?;
            println!();
            println!("  Diagnostic report exported to: {}", path.bright_green());
        }

        Ok(())
    }

    fn check_env_quick(&self, verbose: bool) -> (bool, Vec<String>, Vec<String>) {
        let mut ok = true;
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check OS
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        if verbose {
            println!("    {} OS: {} ({})", "•".bright_black(), os, arch);
        }

        // Check HOME
        if std::env::var("HOME").is_err() && std::env::var("USERPROFILE").is_err() {
            issues.push("HOME environment variable not set".to_string());
            ok = false;
            println!("    {} HOME variable", "✗".bright_red());
        } else if verbose {
            println!("    {} HOME variable set", "✓".bright_green());
        }

        // Check PATH for common directories
        if let Ok(path) = std::env::var("PATH") {
            let has_cargo = path.contains(".cargo/bin") || path.contains(".cargo\\bin");
            let has_solana = path.contains(".local/share/solana") || path.contains("solana");

            if !has_cargo {
                warnings.push("~/.cargo/bin not in PATH".to_string());
                if verbose {
                    println!("    {} Cargo bin in PATH", "⚠".bright_yellow());
                }
            } else if verbose {
                println!("    {} Cargo bin in PATH", "✓".bright_green());
            }

            if !has_solana {
                warnings.push("Solana CLI not in PATH".to_string());
                if verbose {
                    println!("    {} Solana in PATH", "⚠".bright_yellow());
                }
            } else if verbose {
                println!("    {} Solana in PATH", "✓".bright_green());
            }
        }

        // Check Solana config
        let solana_config = dirs::home_dir()
            .map(|h| h.join(".config/solana/cli/config.yml"))
            .filter(|p| p.exists());

        if solana_config.is_none() {
            warnings.push(
                "Solana CLI not configured (run: solana config set --url devnet)".to_string(),
            );
            if verbose {
                println!("    {} Solana CLI config", "⚠".bright_yellow());
            }
        } else if verbose {
            println!("    {} Solana CLI config exists", "✓".bright_green());
        }

        if ok && issues.is_empty() {
            println!("    {} Environment OK", "✓".bright_green());
        }

        (ok, issues, warnings)
    }

    fn check_toolchain_quick(&self, verbose: bool) -> (bool, Vec<String>, Vec<String>) {
        let mut ok = true;
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Essential tools
        let essential = vec![
            ("rustc", "Rust compiler"),
            ("cargo", "Cargo package manager"),
            ("solana", "Solana CLI"),
        ];

        // Optional but recommended
        let optional = vec![
            ("anchor", "Anchor framework"),
            ("nargo", "Noir compiler"),
            ("light", "Light Protocol CLI"),
        ];

        for (cmd, name) in essential {
            if self.check_command(cmd) {
                if verbose {
                    let version = self.get_version(cmd).unwrap_or_default();
                    println!(
                        "    {} {} {}",
                        "✓".bright_green(),
                        name,
                        version.bright_black()
                    );
                }
            } else {
                issues.push(format!("{} not installed", name));
                ok = false;
                println!("    {} {} not found", "✗".bright_red(), name);
            }
        }

        for (cmd, name) in optional {
            if self.check_command(cmd) {
                if verbose {
                    let version = self.get_version(cmd).unwrap_or_default();
                    println!(
                        "    {} {} {}",
                        "✓".bright_green(),
                        name,
                        version.bright_black()
                    );
                }
            } else {
                warnings.push(format!("{} not installed (optional)", name));
                if verbose {
                    println!("    {} {} (optional)", "○".bright_black(), name);
                }
            }
        }

        if ok {
            println!("    {} Essential tools OK", "✓".bright_green());
        }

        (ok, issues, warnings)
    }

    fn check_config_quick(&self, verbose: bool) -> (bool, Vec<String>, Vec<String>) {
        let ok = true;
        let issues = Vec::new();
        let mut warnings = Vec::new();

        // Check SolPrivacy config
        match AppConfig::load() {
            Ok(config) => {
                if verbose {
                    println!("    {} SolPrivacy config loaded", "✓".bright_green());
                    println!("    {} RPC: {}", "•".bright_black(), config.get_rpc_url());
                }

                // Check if using mainnet without explicit confirmation
                if config.network == "mainnet" {
                    warnings.push("Using mainnet RPC - ensure this is intentional".to_string());
                    if verbose {
                        println!("    {} Mainnet RPC configured", "⚠".bright_yellow());
                    }
                }
            }
            Err(_) => {
                warnings.push("No SolPrivacy config found (using defaults)".to_string());
                if verbose {
                    println!("    {} Using default configuration", "○".bright_black());
                }
            }
        }

        // Check for local project
        if (Path::new("Cargo.toml").exists() || Path::new("Anchor.toml").exists()) && verbose {
            println!(
                "    {} Project detected in current directory",
                "✓".bright_green()
            );
        }

        if ok {
            println!("    {} Configuration OK", "✓".bright_green());
        }

        (ok, issues, warnings)
    }

    async fn check_network_quick(&self, verbose: bool) -> (bool, Vec<String>, Vec<String>) {
        let mut ok = true;
        let mut issues = Vec::new();
        let warnings = Vec::new();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        // Test devnet
        let devnet_ok = client
            .post("https://api.devnet.solana.com")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getHealth"
            }))
            .send()
            .await
            .is_ok();

        if devnet_ok {
            if verbose {
                println!("    {} Devnet reachable", "✓".bright_green());
            }
        } else {
            issues.push("Cannot reach Solana devnet".to_string());
            ok = false;
            println!("    {} Devnet unreachable", "✗".bright_red());
        }

        if ok {
            println!("    {} Network OK", "✓".bright_green());
        }

        (ok, issues, warnings)
    }

    async fn check_environment(&self) -> Result<()> {
        println!("{} Environment Check", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        // System info
        println!("  {}:", "System".bright_white());
        println!("    OS:           {}", std::env::consts::OS);
        println!("    Architecture: {}", std::env::consts::ARCH);
        println!();

        // Environment variables
        println!("  {}:", "Environment Variables".bright_white());

        let vars = vec![
            ("HOME", std::env::var("HOME").ok()),
            (
                "PATH",
                std::env::var("PATH")
                    .ok()
                    .map(|p| format!("{}...", &p[..p.len().min(50)])),
            ),
            ("SOLANA_CONFIG", std::env::var("SOLANA_CONFIG").ok()),
            (
                "ANCHOR_PROVIDER_URL",
                std::env::var("ANCHOR_PROVIDER_URL").ok(),
            ),
            (
                "HELIUS_API_KEY",
                std::env::var("HELIUS_API_KEY")
                    .ok()
                    .map(|_| "[SET]".to_string()),
            ),
        ];

        for (name, value) in vars {
            let display = value.as_deref().unwrap_or("[not set]");
            let icon = if value.is_some() {
                "✓".bright_green()
            } else {
                "○".bright_black()
            };
            println!("    {} {:20} {}", icon, name, display.bright_black());
        }
        println!();

        // Solana CLI config
        println!("  {}:", "Solana CLI Configuration".bright_white());
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".config/solana/cli/config.yml");
            if config_path.exists() {
                println!("    {} Config file exists", "✓".bright_green());

                // Try to get current config
                if let Ok(output) = Command::new("solana").arg("config").arg("get").output() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines().take(5) {
                        println!("    {}", line.bright_black());
                    }
                }
            } else {
                println!("    {} No config file found", "○".bright_black());
                println!();
                println!(
                    "    Run: {}",
                    "solana config set --url devnet".bright_cyan()
                );
            }
        }

        Ok(())
    }

    async fn check_toolchain(&self, fix: bool) -> Result<()> {
        println!("{} Toolchain Verification", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        let tools = vec![
            (
                "rustc",
                "Rust Compiler",
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            ),
            ("cargo", "Cargo", "Installed with Rust"),
            (
                "solana",
                "Solana CLI",
                "sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\"",
            ),
            (
                "anchor",
                "Anchor Framework",
                "cargo install --git https://github.com/coral-xyz/anchor avm --locked",
            ),
            (
                "nargo",
                "Noir Compiler",
                "curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash",
            ),
            ("light", "Light Protocol CLI", "cargo install light-cli"),
        ];

        let mut missing = Vec::new();

        println!("  {}:", "Installed Tools".bright_white());
        println!();

        for (cmd, name, install) in &tools {
            let installed = self.check_command(cmd);
            let version = if installed {
                self.get_version(cmd)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "not installed".to_string()
            };

            let icon = if installed {
                "✓".bright_green()
            } else {
                "✗".bright_red()
            };
            let ver_color = if installed {
                version.bright_green()
            } else {
                version.bright_red()
            };

            println!("    {} {:.<25} {}", icon, format!("{} ", name), ver_color);

            if !installed {
                missing.push((name, install));
            }
        }

        println!();

        if missing.is_empty() {
            println!("{} All tools installed!", "✓".bright_green());
        } else if fix {
            println!("  {}:", "Installation Commands".bright_white());
            for (name, install) in &missing {
                println!();
                println!("    {} {}:", "→".bright_cyan(), name);
                println!("      {}", install.bright_black());
            }
        } else {
            println!("{} {} tool(s) missing", "⚠".bright_yellow(), missing.len());
            println!();
            println!(
                "  Run {} to see installation commands",
                "solprivacy doctor toolchain --fix".bright_cyan()
            );
        }

        Ok(())
    }

    async fn check_network(&self, all: bool) -> Result<()> {
        println!("{} Network Connectivity Check", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let endpoints = if all {
            vec![
                ("Devnet", "https://api.devnet.solana.com"),
                ("Testnet", "https://api.testnet.solana.com"),
                ("Mainnet", "https://api.mainnet-beta.solana.com"),
            ]
        } else {
            vec![("Devnet", "https://api.devnet.solana.com")]
        };

        println!("  {}:", "Solana RPC Endpoints".bright_white());
        println!();

        for (name, url) in endpoints {
            let start = std::time::Instant::now();
            let result = client
                .post(url)
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "getHealth"
                }))
                .send()
                .await;

            let latency = start.elapsed().as_millis();

            match result {
                Ok(resp) if resp.status().is_success() => {
                    println!(
                        "    {} {:.<20} {} ({}ms)",
                        "✓".bright_green(),
                        format!("{} ", name),
                        "reachable".bright_green(),
                        latency
                    );
                }
                _ => {
                    println!(
                        "    {} {:.<20} {}",
                        "✗".bright_red(),
                        format!("{} ", name),
                        "unreachable".bright_red()
                    );
                }
            }
        }

        println!();

        // Test Helius if configured
        println!("  {}:", "Enhanced RPC Providers".bright_white());
        println!();

        if let Ok(config) = AppConfig::load() {
            if config.rpc.helius_api_key.is_some() {
                println!("    {} Helius API key configured", "✓".bright_green());
            } else {
                println!("    {} Helius API key not set", "○".bright_black());
            }
        }

        // Test Photon/Light Protocol
        let photon_result = client
            .post("https://devnet.helius-rpc.com/?api-key=test")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getHealth"
            }))
            .send()
            .await;

        if photon_result.is_ok() {
            println!("    {} Photon RPC available", "✓".bright_green());
        } else {
            println!("    {} Photon RPC (requires API key)", "○".bright_black());
        }

        Ok(())
    }

    async fn check_config(&self) -> Result<()> {
        println!("{} Configuration Validation", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        // SolPrivacy config
        println!("  {}:", "SolPrivacy Configuration".bright_white());
        match AppConfig::load() {
            Ok(config) => {
                println!("    {} Config file loaded", "✓".bright_green());
                println!();
                println!("    RPC URL:     {}", config.get_rpc_url().bright_black());
                println!("    Network:     {}", config.network.bright_black());
                println!(
                    "    Helius Key:  {}",
                    if config.rpc.helius_api_key.is_some() {
                        "[configured]"
                    } else {
                        "[not set]"
                    }
                    .bright_black()
                );
                println!(
                    "    Photon URL:  {}",
                    config
                        .rpc
                        .photon_url
                        .as_deref()
                        .unwrap_or("(default)")
                        .bright_black()
                );
            }
            Err(_) => {
                println!("    {} No config file (using defaults)", "○".bright_black());
            }
        }
        println!();

        // Local project check
        println!("  {}:", "Project Detection".bright_white());

        let checks = vec![
            ("Cargo.toml", "Rust/Cargo project"),
            ("Anchor.toml", "Anchor project"),
            ("package.json", "Node.js project"),
            ("Nargo.toml", "Noir circuit project"),
        ];

        let mut found_any = false;
        for (file, desc) in checks {
            if Path::new(file).exists() {
                println!("    {} {} detected", "✓".bright_green(), desc);
                found_any = true;
            }
        }

        if !found_any {
            println!(
                "    {} No project files in current directory",
                "○".bright_black()
            );
        }

        Ok(())
    }

    async fn troubleshoot(&self, issue: Option<String>) -> Result<()> {
        println!("{} Troubleshooting Guide", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        match issue.as_deref() {
            Some("rpc") => self.troubleshoot_rpc(),
            Some("keys") => self.troubleshoot_keys(),
            Some("builds") => self.troubleshoot_builds(),
            Some("proofs") => self.troubleshoot_proofs(),
            _ => self.troubleshoot_all(),
        }
    }

    fn troubleshoot_rpc(&self) -> Result<()> {
        println!("  {}:", "RPC Connection Issues".bright_white());
        println!();
        println!("    {} Check network connectivity", "1.".bright_cyan());
        println!("       solprivacy doctor network --all");
        println!();
        println!("    {} Verify RPC URL configuration", "2.".bright_cyan());
        println!("       solprivacy config show");
        println!();
        println!("    {} Try alternative RPC endpoints", "3.".bright_cyan());
        println!("       solprivacy config provider helius  # requires API key");
        println!();
        println!("    {} Rate limiting", "4.".bright_cyan());
        println!("       Public RPCs have rate limits. Consider using Helius or Triton.");
        println!();
        println!("    {} Common errors", "5.".bright_cyan());
        println!("       • 429 Too Many Requests: Wait or use paid RPC");
        println!("       • Connection refused: Check firewall/VPN");
        println!("       • Timeout: Network issues or RPC overloaded");
        Ok(())
    }

    fn troubleshoot_keys(&self) -> Result<()> {
        println!("  {}:", "Key/Wallet Issues".bright_white());
        println!();
        println!("    {} Check wallet exists", "1.".bright_cyan());
        println!("       solprivacy wallet list");
        println!();
        println!("    {} Create new wallet", "2.".bright_cyan());
        println!("       solprivacy wallet new --name default");
        println!();
        println!("    {} Check Solana CLI keypair", "3.".bright_cyan());
        println!("       solana address");
        println!();
        println!("    {} ElGamal key generation", "4.".bright_cyan());
        println!("       solprivacy keygen elgamal --output elgamal.json");
        println!();
        println!("    {} Common errors", "5.".bright_cyan());
        println!("       • 'No such file': Wallet file missing or wrong path");
        println!("       • 'Invalid keypair': Corrupted or wrong format");
        println!("       • 'Insufficient funds': Need SOL for transactions");
        Ok(())
    }

    fn troubleshoot_builds(&self) -> Result<()> {
        println!("  {}:", "Build/Compilation Issues".bright_white());
        println!();
        println!("    {} Update Rust toolchain", "1.".bright_cyan());
        println!("       rustup update");
        println!();
        println!("    {} Clear build cache", "2.".bright_cyan());
        println!("       cargo clean");
        println!();
        println!(
            "    {} Check Solana version compatibility",
            "3.".bright_cyan()
        );
        println!("       solana --version");
        println!("       # Ensure SDK version matches in Cargo.toml");
        println!();
        println!("    {} Anchor build issues", "4.".bright_cyan());
        println!("       anchor build -- --features no-entrypoint");
        println!();
        println!("    {} Common errors", "5.".bright_cyan());
        println!("       • 'unresolved import': Missing dependency in Cargo.toml");
        println!("       • 'version conflict': Run `cargo update`");
        println!("       • 'BPF target': Install with `rustup target add sbf-solana-solana`");
        Ok(())
    }

    fn troubleshoot_proofs(&self) -> Result<()> {
        println!("  {}:", "ZK Proof Issues".bright_white());
        println!();
        println!("    {} Verify Noir installation", "1.".bright_cyan());
        println!("       nargo --version");
        println!();
        println!("    {} Check circuit syntax", "2.".bright_cyan());
        println!("       nargo check");
        println!();
        println!("    {} Compile circuit", "3.".bright_cyan());
        println!("       nargo compile");
        println!();
        println!("    {} Verify inputs format", "4.".bright_cyan());
        println!("       # Prover.toml must have correct field types");
        println!("       # Use quotes for large numbers: value = \"12345\"");
        println!();
        println!("    {} Common errors", "5.".bright_cyan());
        println!("       • 'Constraint not satisfied': Input doesn't meet circuit requirements");
        println!("       • 'Type mismatch': Check field types in Prover.toml");
        println!("       • 'Cannot find module': Missing dependency in Nargo.toml");
        Ok(())
    }

    fn troubleshoot_all(&self) -> Result<()> {
        println!("  {}:", "Common Issue Categories".bright_white());
        println!();
        println!(
            "    {} {} - Connection and endpoint issues",
            "rpc".bright_cyan(),
            " ".repeat(10)
        );
        println!("       solprivacy doctor troubleshoot --issue rpc");
        println!();
        println!(
            "    {} {} - Wallet and keypair problems",
            "keys".bright_cyan(),
            " ".repeat(9)
        );
        println!("       solprivacy doctor troubleshoot --issue keys");
        println!();
        println!(
            "    {} {} - Compilation and build errors",
            "builds".bright_cyan(),
            " ".repeat(7)
        );
        println!("       solprivacy doctor troubleshoot --issue builds");
        println!();
        println!(
            "    {} {} - ZK proof generation issues",
            "proofs".bright_cyan(),
            " ".repeat(7)
        );
        println!("       solprivacy doctor troubleshoot --issue proofs");
        println!();
        println!("  {}:", "Quick Fixes".bright_white());
        println!();
        println!(
            "    • Run full diagnostic: {}",
            "solprivacy doctor check -v".bright_cyan()
        );
        println!(
            "    • Verify toolchain:    {}",
            "solprivacy doctor toolchain --fix".bright_cyan()
        );
        println!(
            "    • Test network:        {}",
            "solprivacy doctor network --all".bright_cyan()
        );
        Ok(())
    }

    async fn generate_report(&self, output: &str) -> Result<()> {
        println!("{} Generating Diagnostic Report", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        let mut report = String::new();

        report.push_str("SolPrivacy Diagnostic Report\n");
        report.push_str(&"=".repeat(50));
        report.push_str("\n\n");

        // System info
        report.push_str("## System Information\n\n");
        report.push_str(&format!("OS: {}\n", std::env::consts::OS));
        report.push_str(&format!("Architecture: {}\n", std::env::consts::ARCH));
        report.push_str(&format!(
            "Generated: {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));

        // Toolchain versions
        report.push_str("## Toolchain Versions\n\n");
        for cmd in &["rustc", "cargo", "solana", "anchor", "nargo", "light"] {
            let version = self
                .get_version(cmd)
                .unwrap_or_else(|| "not installed".to_string());
            report.push_str(&format!("{}: {}\n", cmd, version));
        }
        report.push('\n');

        // Configuration
        report.push_str("## Configuration\n\n");
        if let Ok(config) = AppConfig::load() {
            report.push_str(&format!("RPC URL: {}\n", config.get_rpc_url()));
            report.push_str(&format!("Network: {}\n", config.network));
            report.push_str(&format!(
                "Helius API: {}\n",
                if config.rpc.helius_api_key.is_some() {
                    "configured"
                } else {
                    "not set"
                }
            ));
        } else {
            report.push_str("Config: Using defaults\n");
        }
        report.push('\n');

        // Environment
        report.push_str("## Environment Variables\n\n");
        for var in &["HOME", "SOLANA_CONFIG", "ANCHOR_PROVIDER_URL"] {
            let value = std::env::var(var).unwrap_or_else(|_| "not set".to_string());
            report.push_str(&format!("{}: {}\n", var, value));
        }

        fs::write(output, &report)?;

        println!(
            "  {} Report saved to: {}",
            "✓".bright_green(),
            output.bright_cyan()
        );
        println!();
        println!("  Share this file when requesting support.");

        Ok(())
    }

    fn check_command(&self, cmd: &str) -> bool {
        Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn get_version(&self, cmd: &str) -> Option<String> {
        Command::new(cmd)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .map(|s| s.trim().to_string())
            })
    }
}
