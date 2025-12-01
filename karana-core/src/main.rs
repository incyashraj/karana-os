use anyhow::Result;
use clap::Parser;
use karana_core::monad::KaranaMonad;
use karana_core::installer::{Cli, Commands, run_install};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { mode }) => {
            run_install(mode)?;
        }
        Some(Commands::Boot) | None => {
            // Default behavior: Boot the OS Kernel
            let mut monad = KaranaMonad::new().await?;
            monad.ignite().await?;
        }
    }

    Ok(())
}
