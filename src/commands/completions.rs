use crate::error::Result;
use crate::Cli;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use std::io; // Import the main Cli struct to generate completions for it

/// Generate shell completion scripts
#[derive(Args)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

impl CompletionsCommand {
    pub fn run(&self) -> Result<()> {
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        generate(self.shell, &mut cmd, bin_name, &mut io::stdout());
        Ok(())
    }
}
