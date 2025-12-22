use clap::{Parser, Subcommand};
use crate::{Config, Result};

#[derive(Parser)]
#[command(name = "skypier-blackhole")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
    pub config: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the DNS server
    Start,
    
    /// Stop the DNS server
    Stop,
    
    /// Reload blocklists without restarting
    Reload,
    
    /// Show server status and statistics
    Status,
    
    /// Add a domain to the blocklist
    Add { domain: String },
    
    /// Remove a domain from the blocklist
    Remove { domain: String },
    
    /// List blocklist statistics
    List,
    
    /// Force update blocklists from remote sources
    Update,
    
    /// Test if a domain is blocked
    Test { domain: String },
}

impl Cli {
    pub async fn execute(&self, _config: Config) -> Result<()> {
        match &self.command {
            Some(Commands::Start) => {
                tracing::info!("Starting DNS server...");
                // TODO: Implement DNS server start
                Ok(())
            }
            Some(Commands::Stop) => {
                tracing::info!("Stopping DNS server...");
                // TODO: Implement server stop
                Ok(())
            }
            Some(Commands::Reload) => {
                tracing::info!("Reloading blocklists...");
                // TODO: Implement reload
                Ok(())
            }
            Some(Commands::Status) => {
                tracing::info!("Fetching server status...");
                // TODO: Implement status
                Ok(())
            }
            Some(Commands::Add { domain }) => {
                tracing::info!("Adding domain to blocklist: {}", domain);
                // TODO: Implement add
                Ok(())
            }
            Some(Commands::Remove { domain }) => {
                tracing::info!("Removing domain from blocklist: {}", domain);
                // TODO: Implement remove
                Ok(())
            }
            Some(Commands::List) => {
                tracing::info!("Listing blocklist statistics...");
                // TODO: Implement list
                Ok(())
            }
            Some(Commands::Update) => {
                tracing::info!("Forcing blocklist update...");
                // TODO: Implement update
                Ok(())
            }
            Some(Commands::Test { domain }) => {
                tracing::info!("Testing domain: {}", domain);
                // TODO: Implement test
                Ok(())
            }
            None => {
                // Default action: start server
                tracing::info!("Starting DNS server (default action)...");
                // TODO: Implement DNS server start
                Ok(())
            }
        }
    }
}
