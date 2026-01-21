use clap::{Args, Subcommand};
use colored::Colorize;
use std::process::{Command, Stdio};
use std::fs;
use std::path::PathBuf;
use crate::config::AppConfig;
use crate::error::Result;

/// Local development environment commands
#[derive(Args)]
pub struct DevCommand {
    #[command(subcommand)]
    pub action: DevAction,
}

#[derive(Subcommand)]
pub enum DevAction {
    /// Start a local Solana validator with Token-2022
    Start {
        /// Reset ledger (clean start)
        #[arg(short, long)]
        reset: bool,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Stop the local validator
    Stop,
    
    /// Show status of local services
    Status,
    
    /// Stream validator logs
    Logs {
        /// Number of lines to show (default: 50)
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Reset local development state
    Reset,
}

impl DevCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            DevAction::Start { reset, verbose } => self.start_validator(*reset, *verbose).await,
            DevAction::Stop => self.stop_validator().await,
            DevAction::Status => self.show_status().await,
            DevAction::Logs { lines, follow } => self.show_logs(*lines, *follow).await,
            DevAction::Reset => self.reset_state().await,
        }
    }
    
    fn ledger_dir() -> PathBuf {
        AppConfig::config_dir().join("test-ledger")
    }
    
    fn pid_file() -> PathBuf {
        AppConfig::config_dir().join("validator.pid")
    }
    
    fn log_file() -> PathBuf {
        AppConfig::config_dir().join("validator.log")
    }
    
    async fn start_validator(&self, reset: bool, verbose: bool) -> Result<()> {
        println!("{} Local Development Environment", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        // Check if solana-test-validator is available
        let check = Command::new("solana-test-validator").arg("--version").output();
        if check.is_err() || !check.unwrap().status.success() {
            println!("{} solana-test-validator not found!", "✗".bright_red());
            println!();
            println!("  Install Solana CLI first:");
            println!("    sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\"");
            return Ok(());
        }
        
        // Check if already running
        if self.is_validator_running() {
            println!("{} Validator already running!", "⚠".bright_yellow());
            println!();
            println!("  Stop it first: solprivacy dev stop");
            return Ok(());
        }
        
        // Reset ledger if requested
        if reset {
            println!("{} Resetting ledger...", "→".bright_cyan());
            let _ = fs::remove_dir_all(Self::ledger_dir());
        }
        
        // Create config dir
        fs::create_dir_all(AppConfig::config_dir())?;
        
        println!("{} Starting local validator...", "→".bright_cyan());
        println!();
        
        // Build command
        let mut cmd = Command::new("solana-test-validator");
        cmd.arg("--ledger").arg(Self::ledger_dir());
        cmd.arg("--rpc-port").arg("8899");
        cmd.arg("--quiet");
        
        if verbose {
            cmd.arg("--log");
        }
        
        // Redirect output to log file
        let log_file = fs::File::create(Self::log_file())?;
        cmd.stdout(Stdio::from(log_file.try_clone()?));
        cmd.stderr(Stdio::from(log_file));
        
        // Start as background process
        match cmd.spawn() {
            Ok(child) => {
                // Save PID
                fs::write(Self::pid_file(), child.id().to_string())?;
                
                println!("{} Validator starting...", "✓".bright_green());
                println!();
                println!("  {}:", "Local Environment".bright_white());
                println!("  ├─ RPC URL: {}", "http://127.0.0.1:8899".bright_cyan());
                println!("  ├─ WebSocket: {}", "ws://127.0.0.1:8900".bright_cyan());
                println!("  ├─ Ledger: {}", Self::ledger_dir().display());
                println!("  └─ PID: {}", child.id());
                println!();
                println!("  {}:", "Commands".bright_white());
                println!("    solprivacy dev status    Check status");
                println!("    solprivacy dev logs -f   Stream logs");
                println!("    solprivacy dev stop      Stop validator");
                println!();
                println!("  {}:", "Configure CLI".bright_white());
                println!("    solprivacy config network localnet");
                
                // Wait a moment and check if it started
                std::thread::sleep(std::time::Duration::from_secs(2));
                
                if self.is_validator_running() {
                    println!();
                    println!("{} Validator is running!", "✓".bright_green());
                } else {
                    println!();
                    println!("{} Validator may have failed to start", "⚠".bright_yellow());
                    println!("  Check logs: solprivacy dev logs");
                }
            }
            Err(e) => {
                println!("{} Failed to start validator: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn stop_validator(&self) -> Result<()> {
        println!("{} Stopping Local Validator", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_validator_running() {
            println!("{} Validator is not running", "ℹ".bright_blue());
            return Ok(());
        }
        
        // Read PID and kill
        if let Ok(pid_str) = fs::read_to_string(Self::pid_file()) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                #[cfg(unix)]
                {
                    let _ = Command::new("kill").arg(pid.to_string()).status();
                }
                
                #[cfg(not(unix))]
                {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F"])
                        .status();
                }
                
                // Clean up PID file
                let _ = fs::remove_file(Self::pid_file());
                
                println!("{} Validator stopped", "✓".bright_green());
            }
        } else {
            // Try to find and kill any solana-test-validator
            #[cfg(unix)]
            {
                let _ = Command::new("pkill").arg("-f").arg("solana-test-validator").status();
            }
            
            println!("{} Validator stopped", "✓".bright_green());
        }
        
        Ok(())
    }
    
    async fn show_status(&self) -> Result<()> {
        println!("{} Development Environment Status", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let is_running = self.is_validator_running();
        
        println!("  {}:", "Local Validator".bright_white());
        if is_running {
            println!("  ├─ Status: {}", "Running".bright_green());
            println!("  ├─ RPC: http://127.0.0.1:8899");
            
            // Try to get actual status
            if let Ok(output) = Command::new("solana")
                .args(["cluster-version", "--url", "http://127.0.0.1:8899"])
                .output()
            {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("  └─ Version: {}", version.trim());
                }
            }
        } else {
            println!("  └─ Status: {}", "Stopped".bright_red());
        }
        
        println!();
        
        // Show current config
        let config = AppConfig::load()?;
        println!("  {}:", "Current Config".bright_white());
        println!("  ├─ Network: {}", config.network);
        println!("  └─ RPC: {}", config.get_rpc_url());
        
        if !is_running && config.network == "localnet" {
            println!();
            println!("{} Network is localnet but validator not running!", "⚠".bright_yellow());
            println!("  Start with: solprivacy dev start");
        }
        
        Ok(())
    }
    
    async fn show_logs(&self, lines: usize, follow: bool) -> Result<()> {
        let log_path = Self::log_file();
        
        if !log_path.exists() {
            println!("{} No logs found", "ℹ".bright_blue());
            println!("  Start validator first: solprivacy dev start");
            return Ok(());
        }
        
        if follow {
            println!("{} Following validator logs (Ctrl+C to stop)", "→".bright_cyan());
            println!("{}", "─".repeat(50).bright_black());
            
            // Use tail -f
            let _ = Command::new("tail")
                .args(["-f", "-n", &lines.to_string()])
                .arg(&log_path)
                .status();
        } else {
            println!("{} Validator Logs (last {} lines)", "→".bright_cyan(), lines);
            println!("{}", "─".repeat(50).bright_black());
            println!();
            
            // Read last N lines
            let content = fs::read_to_string(&log_path)?;
            let all_lines: Vec<&str> = content.lines().collect();
            let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
            
            for line in &all_lines[start..] {
                println!("{}", line);
            }
        }
        
        Ok(())
    }
    
    async fn reset_state(&self) -> Result<()> {
        println!("{} Resetting Development State", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        // Stop validator if running
        if self.is_validator_running() {
            println!("{} Stopping validator...", "→".bright_cyan());
            self.stop_validator().await?;
        }
        
        // Remove ledger
        if Self::ledger_dir().exists() {
            println!("{} Removing ledger...", "→".bright_cyan());
            fs::remove_dir_all(Self::ledger_dir())?;
        }
        
        // Remove log file
        if Self::log_file().exists() {
            println!("{} Removing logs...", "→".bright_cyan());
            fs::remove_file(Self::log_file())?;
        }
        
        println!();
        println!("{} Development state reset!", "✓".bright_green());
        println!();
        println!("  Start fresh: solprivacy dev start");
        
        Ok(())
    }
    
    fn is_validator_running(&self) -> bool {
        // Check if we can connect to local RPC
        let output = Command::new("solana")
            .args(["cluster-version", "--url", "http://127.0.0.1:8899"])
            .output();
        
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}
