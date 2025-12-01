use anyhow::Result;
use clap::Parser;
use karana_core::monad::{KaranaMonad, KaranaConfig};
use karana_core::installer::{Cli, Commands, run_install};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { mode }) => {
            run_install(mode)?;
        }
        Some(Commands::Boot { port, peer, path }) => {
            // Default behavior: Boot the OS Kernel
            let config = KaranaConfig {
                port,
                peer,
                base_path: path.unwrap_or_else(|| ".".to_string()),
            };
            let mut monad = KaranaMonad::new(config).await?;
            monad.ignite().await?;
        }
        None => {
             // Fallback for None (should match Boot default)
             let config = KaranaConfig {
                port: 0,
                peer: None,
                base_path: ".".to_string(),
            };
            let mut monad = KaranaMonad::new(config).await?;
            monad.ignite().await?;
        }
    }

    Ok(())
}
