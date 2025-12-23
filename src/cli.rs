use clap::{Parser, Subcommand};
use crate::{BlocklistManager, Config, DnsServer, Result};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use futures::stream::StreamExt;
use std::sync::Arc;

/// Load blocklist from configuration
async fn load_blocklist_from_config(
    config: &Config,
    blocklist: &BlocklistManager,
) -> Result<()> {
    let mut all_domains = Vec::new();
    
    // Load from custom file if it exists
    if std::path::Path::new(&config.blocklist.custom_list).exists() {
        tracing::info!("Loading blocklist from {}", config.blocklist.custom_list);
        let content = std::fs::read_to_string(&config.blocklist.custom_list)?;
        let domains: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| line.trim().to_string())
            .collect();
        all_domains.extend(domains);
    } else {
        tracing::warn!("Blocklist file not found: {}", config.blocklist.custom_list);
    }
    
    // Load from local lists
    for local_list in &config.blocklist.local_lists {
        if std::path::Path::new(local_list).exists() {
            tracing::info!("Loading local blocklist from {}", local_list);
            let content = std::fs::read_to_string(local_list)?;
            let domains: Vec<String> = content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(|line| line.trim().to_string())
                .collect();
            all_domains.extend(domains);
        } else {
            tracing::warn!("Local blocklist file not found: {}", local_list);
        }
    }
    
    blocklist.load_domains(all_domains).await?;
    let count = blocklist.count().await;
    tracing::info!("Loaded {} domains into blocklist", count);
    
    Ok(())
}

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
    pub async fn execute(&self, config: Config) -> Result<()> {
        match &self.command {
            Some(Commands::Start) => {
                tracing::info!("Starting DNS server...");
                
                // Create blocklist manager
                let blocklist = Arc::new(BlocklistManager::new());
                
                // Load initial blocklist
                load_blocklist_from_config(&config, &blocklist).await?;
                tracing::info!("Blocklist manager initialized");
                
                // Create DNS server
                let server = DnsServer::new(config.clone(), Arc::clone(&blocklist))?;
                
                // Setup signal handling for graceful shutdown and reload
                let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGHUP])?;
                let signals_handle = signals.handle();
                
                let config_clone = config.clone();
                let blocklist_clone = Arc::clone(&blocklist);
                
                // Spawn signal handler task
                let signal_task = tokio::spawn(async move {
                    while let Some(signal) = signals.next().await {
                        match signal {
                            SIGTERM | SIGINT => {
                                tracing::info!("Received shutdown signal, stopping server...");
                                // Server will stop when main task exits
                                break;
                            }
                            SIGHUP => {
                                tracing::info!("Received SIGHUP, reloading blocklists...");
                                match load_blocklist_from_config(&config_clone, &blocklist_clone).await {
                                    Ok(_) => {
                                        let count = blocklist_clone.count().await;
                                        tracing::info!("Blocklist reloaded successfully with {} domains", count);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to reload blocklist: {}", e);
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                });
                
                // Start DNS server (blocks until error or signal)
                let server_task = tokio::spawn(async move {
                    server.start().await
                });
                
                // Wait for either server error or signal
                tokio::select! {
                    result = server_task => {
                        match result {
                            Ok(Ok(())) => tracing::info!("DNS server stopped normally"),
                            Ok(Err(e)) => tracing::error!("DNS server error: {}", e),
                            Err(e) => tracing::error!("Server task panicked: {}", e),
                        }
                    }
                    _ = signal_task => {
                        tracing::info!("Signal handler stopped");
                    }
                }
                
                // Cleanup
                signals_handle.close();
                tracing::info!("Server shutdown complete");
                
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
                
                // Create blocklist manager
                let blocklist = Arc::new(BlocklistManager::new());
                
                // Load initial blocklist
                load_blocklist_from_config(&config, &blocklist).await?;
                
                // Create DNS server
                let server = DnsServer::new(config.clone(), Arc::clone(&blocklist))?;
                
                // Setup signal handling
                let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGHUP])?;
                let signals_handle = signals.handle();
                
                let config_clone = config.clone();
                let blocklist_clone = Arc::clone(&blocklist);
                
                // Spawn signal handler task
                let signal_task = tokio::spawn(async move {
                    while let Some(signal) = signals.next().await {
                        match signal {
                            SIGTERM | SIGINT => {
                                tracing::info!("Received shutdown signal, stopping server...");
                                break;
                            }
                            SIGHUP => {
                                tracing::info!("Received SIGHUP, reloading blocklists...");
                                match load_blocklist_from_config(&config_clone, &blocklist_clone).await {
                                    Ok(_) => {
                                        let count = blocklist_clone.count().await;
                                        tracing::info!("Blocklist reloaded successfully with {} domains", count);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to reload blocklist: {}", e);
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                });
                
                // Start DNS server
                let server_task = tokio::spawn(async move {
                    server.start().await
                });
                
                // Wait for either server error or signal
                tokio::select! {
                    result = server_task => {
                        match result {
                            Ok(Ok(())) => tracing::info!("DNS server stopped normally"),
                            Ok(Err(e)) => tracing::error!("DNS server error: {}", e),
                            Err(e) => tracing::error!("Server task panicked: {}", e),
                        }
                    }
                    _ = signal_task => {
                        tracing::info!("Signal handler stopped");
                    }
                }
                
                // Cleanup
                signals_handle.close();
                tracing::info!("Server shutdown complete");
                
                Ok(())
            }
        }
    }
}
