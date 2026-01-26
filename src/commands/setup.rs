use crate::error::Result;
use clap::Args;
use colored::Colorize;
use std::process::Command;

/// Setup and validate privacy development toolchains
#[derive(Args)]
pub struct SetupCommand {
    /// Install missing tools automatically (where possible)
    #[arg(short, long)]
    pub install: bool,

    /// Check specific tool only
    #[arg(long)]
    pub check: Option<String>,
}

struct ToolStatus {
    name: &'static str,
    installed: bool,
    version: Option<String>,
    install_cmd: &'static str,
}

impl SetupCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{} Privacy Development Toolchain Setup", "→".bright_cyan());
        println!("{}", "─".repeat(55).bright_black());
        println!();

        if let Some(ref tool) = self.check {
            return self.check_single_tool(tool);
        }

        let mut all_ok = true;
        let mut tools_to_install = Vec::new();

        // Check all tools
        let tools = vec![
            self.check_rust(),
            self.check_solana_cli(),
            self.check_anchor(),
            self.check_noir(),
            self.check_light_cli(),
        ];

        println!("  {}:", "Toolchain Status".bright_white());
        println!();

        for tool in &tools {
            let status_icon = if tool.installed {
                "✓".bright_green()
            } else {
                "✗".bright_red()
            };
            let version_str = tool.version.as_deref().unwrap_or("not installed");

            println!(
                "  {} {} {}",
                status_icon,
                format!("{:.<20}", format!("{} ", tool.name)).bright_black(),
                if tool.installed {
                    version_str.bright_green()
                } else {
                    version_str.bright_red()
                }
            );

            if !tool.installed {
                all_ok = false;
                tools_to_install.push(tool);
            }
        }

        println!();

        if all_ok {
            println!("{} All tools installed and ready!", "✓".bright_green());
            println!();
            println!("  {}:", "Quick Start".bright_white());
            println!("    solprivacy wallet new          Create a wallet");
            println!("    solprivacy init my-app         Create a project");
            println!("    solprivacy dev start           Start local validator");
        } else {
            println!("{} Some tools are missing", "⚠".bright_yellow());
            println!();

            if self.install {
                println!("{} Installing missing tools...", "→".bright_cyan());
                println!();
                for tool in tools_to_install {
                    self.install_tool(tool);
                }
            } else {
                println!("  {}:", "Installation Commands".bright_white());
                for tool in tools_to_install {
                    if !tool.install_cmd.is_empty() {
                        println!("    {} {}", "•".bright_cyan(), tool.install_cmd);
                    }
                }
                println!();
                println!("  Run with --install to attempt automatic installation");
            }
        }

        Ok(())
    }

    fn check_single_tool(&self, tool: &str) -> Result<()> {
        let status = match tool.to_lowercase().as_str() {
            "rust" | "rustc" | "cargo" => self.check_rust(),
            "solana" | "solana-cli" => self.check_solana_cli(),
            "anchor" => self.check_anchor(),
            "noir" | "nargo" => self.check_noir(),
            "light" | "light-cli" => self.check_light_cli(),
            _ => {
                println!("{} Unknown tool: {}", "✗".bright_red(), tool);
                println!("  Available: rust, solana, anchor, noir, light");
                return Ok(());
            }
        };

        if status.installed {
            println!(
                "{} {} is installed: {}",
                "✓".bright_green(),
                status.name,
                status.version.unwrap_or_default()
            );
        } else {
            println!("{} {} is not installed", "✗".bright_red(), status.name);
            if !status.install_cmd.is_empty() {
                println!("  Install: {}", status.install_cmd);
            }
        }

        Ok(())
    }

    fn check_rust(&self) -> ToolStatus {
        let output = Command::new("rustc").arg("--version").output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ToolStatus {
                    name: "Rust",
                    installed: true,
                    version: Some(version),
                    install_cmd: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
                }
            }
            _ => ToolStatus {
                name: "Rust",
                installed: false,
                version: None,
                install_cmd: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            },
        }
    }

    fn check_solana_cli(&self) -> ToolStatus {
        let output = Command::new("solana").arg("--version").output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ToolStatus {
                    name: "Solana CLI",
                    installed: true,
                    version: Some(version),
                    install_cmd: "sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\"",
                }
            }
            _ => ToolStatus {
                name: "Solana CLI",
                installed: false,
                version: None,
                install_cmd: "sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\"",
            },
        }
    }

    fn check_anchor(&self) -> ToolStatus {
        let output = Command::new("anchor").arg("--version").output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ToolStatus {
                    name: "Anchor",
                    installed: true,
                    version: Some(version),
                    install_cmd: "cargo install --git https://github.com/coral-xyz/anchor avm --force && avm install latest && avm use latest",
                }
            }
            _ => ToolStatus {
                name: "Anchor",
                installed: false,
                version: None,
                install_cmd: "cargo install --git https://github.com/coral-xyz/anchor avm --force && avm install latest && avm use latest",
            }
        }
    }

    fn check_noir(&self) -> ToolStatus {
        let output = Command::new("nargo").arg("--version").output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ToolStatus {
                    name: "Noir (nargo)",
                    installed: true,
                    version: Some(version),
                    install_cmd: "curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup",
                }
            }
            _ => ToolStatus {
                name: "Noir (nargo)",
                installed: false,
                version: None,
                install_cmd: "curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup",
            }
        }
    }

    fn check_light_cli(&self) -> ToolStatus {
        let output = Command::new("light").arg("--version").output();

        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ToolStatus {
                    name: "Light CLI",
                    installed: true,
                    version: Some(version),
                    install_cmd: "cargo install light-cli",
                }
            }
            _ => ToolStatus {
                name: "Light CLI",
                installed: false,
                version: None,
                install_cmd: "cargo install light-cli",
            },
        }
    }

    fn install_tool(&self, tool: &ToolStatus) {
        if tool.install_cmd.is_empty() {
            println!(
                "  {} {} - manual installation required",
                "⚠".bright_yellow(),
                tool.name
            );
            return;
        }

        println!("  {} Installing {}...", "→".bright_cyan(), tool.name);

        // For security, we just print the command rather than executing it
        println!("    Run: {}", tool.install_cmd.bright_blue());
        println!();
    }
}
