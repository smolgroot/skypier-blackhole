use clap::Parser;
use skypier_blackhole::{Cli, Config, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Setup logging
    skypier_blackhole::setup_logging()?;
    
    // Load configuration
    let config = Config::load(&cli.config)?;
    
    tracing::info!("Starting Skypier Blackhole DNS resolver");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    // Execute CLI command
    cli.execute(config).await?;
    
    Ok(())
}
