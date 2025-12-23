use clap::{Parser, Subcommand};
use colored::*;
use crate::{BlocklistManager, BlocklistDownloader, Config, DnsServer, Result};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use futures::stream::StreamExt;
use std::sync::Arc;
use std::fs;
use std::io::Write;

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
    
    // Load from remote cache if it exists
    let cache_dir = std::path::Path::new(&config.blocklist.custom_list)
        .parent()
        .unwrap_or(std::path::Path::new("/tmp"));
    let cache_file = cache_dir.join("remote-blocklist-cache.txt");
    
    if cache_file.exists() {
        tracing::info!("Loading remote blocklist cache from {}", cache_file.display());
        let content = std::fs::read_to_string(&cache_file)?;
        let domains: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| line.trim().to_string())
            .collect();
        tracing::info!("Loaded {} domains from remote cache", domains.len());
        all_domains.extend(domains);
    }
    
    blocklist.load_domains(all_domains).await?;
    let count = blocklist.count().await;
    tracing::info!("Loaded {} total domains into blocklist", count);

    
    Ok(())
}

/// Find the PID of the running skypier-blackhole server
fn find_server_pid() -> Result<Option<u32>> {
    let output = std::process::Command::new("pgrep")
        .arg("-f")
        .arg("skypier-blackhole.*start")
        .output()?;
    
    if output.status.success() && !output.stdout.is_empty() {
        let pid_str = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<&str> = pid_str.trim().lines().collect();
        
        // Return the first PID (there should only be one server)
        if let Some(pid) = pids.first() {
            if let Ok(pid_num) = pid.parse::<u32>() {
                return Ok(Some(pid_num));
            }
        }
    }
    
    Ok(None)
}

/// Send a signal to the running server
fn send_signal(pid: u32, signal: i32) -> Result<()> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    
    let sig = match signal {
        SIGTERM => Signal::SIGTERM,
        SIGHUP => Signal::SIGHUP,
        _ => anyhow::bail!("Unsupported signal"),
    };
    
    kill(Pid::from_raw(pid as i32), sig)?;
    
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
    Start {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Stop the DNS server
    Stop {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Reload blocklists without restarting
    Reload {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Show server status and statistics
    Status {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Add a domain to the blocklist
    Add { 
        /// Domain to add (e.g., ads.example.com or *.tracker.com)
        domain: String,
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Remove a domain from the blocklist
    Remove { 
        /// Domain to remove
        domain: String,
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// List blocklist statistics
    List {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Force update blocklists from remote sources
    Update {
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
    
    /// Test if a domain is blocked
    Test { 
        /// Domain to test
        domain: String,
        /// Path to configuration file
        #[arg(short, long, default_value = "/etc/skypier/blackhole.toml")]
        config: String,
    },
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            Some(Commands::Start { config: config_path }) => {
                let config = Config::load(config_path)?;
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
            Some(Commands::Stop { config: config_path }) => {
                let _config = Config::load(config_path)?;
                println!("{}", "🛑 Stopping Skypier Blackhole DNS Server".bright_yellow().bold());
                println!();
                
                match find_server_pid()? {
                    Some(pid) => {
                        println!("  {} Server PID: {}", "📍".bright_blue(), pid.to_string().bright_cyan());
                        println!("  {} Sending SIGTERM...", "⚡".bright_yellow());
                        
                        send_signal(pid, SIGTERM)?;
                        
                        // Wait a bit for graceful shutdown
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        
                        // Check if still running
                        match find_server_pid()? {
                            Some(_) => {
                                println!("  {} Server is taking longer to stop (this is normal)", "⏳".bright_yellow());
                            }
                            None => {
                                println!("  {} Server stopped successfully", "✓".bright_green().bold());
                            }
                        }
                    }
                    None => {
                        println!("  {} No running server found", "ℹ".bright_blue());
                    }
                }
                
                println!();
                Ok(())
            }
            Some(Commands::Reload { config: config_path }) => {
                let _config = Config::load(config_path)?;
                println!("{}", "🔄 Reloading Blocklists".bright_cyan().bold());
                println!();
                
                match find_server_pid()? {
                    Some(pid) => {
                        println!("  {} Server PID: {}", "📍".bright_blue(), pid.to_string().bright_cyan());
                        println!("  {} Sending SIGHUP (hot-reload)...", "⚡".bright_yellow());
                        
                        send_signal(pid, SIGHUP)?;
                        
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        
                        println!("  {} Reload signal sent successfully", "✓".bright_green().bold());
                        println!("  {} Blocklists are being reloaded with zero downtime", "🔥".bright_green());
                    }
                    None => {
                        println!("  {} No running server found", "✗".bright_red().bold());
                        println!("  {} Start the server first with: {} skypier-blackhole start", 
                                "ℹ".bright_blue(), "→".bright_white());
                    }
                }
                
                println!();
                Ok(())
            }
            Some(Commands::Status { config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{}", "📊 Skypier Blackhole Status".bright_magenta().bold());
                println!("{}", "═".repeat(50).bright_black());
                println!();
                
                // Check if server is running
                match find_server_pid()? {
                    Some(pid) => {
                        println!("  {} Server Status: {}", "●".bright_green(), "RUNNING".bright_green().bold());
                        println!("  {} Process ID: {}", "🔢".bright_blue(), pid.to_string().bright_cyan());
                    }
                    None => {
                        println!("  {} Server Status: {}", "○".bright_red(), "STOPPED".bright_red().bold());
                    }
                }
                
                println!();
                
                // Load config and show blocklist stats
                if std::path::Path::new(&config.blocklist.custom_list).exists() {
                    let blocklist = BlocklistManager::new();
                    load_blocklist_from_config(&config, &blocklist).await?;
                    let count = blocklist.count().await;
                    
                    println!("  {} Blocklist Statistics:", "📋".bright_cyan());
                    println!("    {} Total domains blocked: {}", "•".bright_white(), count.to_string().bright_yellow().bold());
                    println!("    {} Custom list: {}", "•".bright_white(), config.blocklist.custom_list.bright_blue());
                    
                    if !config.blocklist.local_lists.is_empty() {
                        println!("    {} Local lists: {}", "•".bright_white(), config.blocklist.local_lists.len().to_string().bright_yellow());
                    }
                } else {
                    println!("  {} No blocklist found at: {}", "⚠".bright_yellow(), config.blocklist.custom_list.bright_blue());
                }
                
                println!();
                println!("  {} Configuration:", "⚙".bright_cyan());
                println!("    {} Listen: {}", "•".bright_white(), 
                        format!("{}:{}", config.server.listen_addr, config.server.listen_port).bright_green());
                println!("    {} Upstream DNS: {}", "•".bright_white(), 
                        config.server.upstream_dns.first().unwrap_or(&"1.1.1.1:53".to_string()).bright_green());
                
                println!();
                println!("{}", "═".repeat(50).bright_black());
                println!();
                
                Ok(())
            }
            Some(Commands::Add { domain, config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{} {}", "➕ Adding domain:".bright_green().bold(), domain.bright_cyan());
                println!();
                
                // Add to custom blocklist file
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&config.blocklist.custom_list)?;
                
                writeln!(file, "{}", domain)?;
                
                println!("  {} Domain added to: {}", "✓".bright_green(), config.blocklist.custom_list.bright_blue());
                
                // Trigger reload if server is running
                match find_server_pid()? {
                    Some(pid) => {
                        println!("  {} Reloading server...", "🔄".bright_cyan());
                        send_signal(pid, SIGHUP)?;
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        println!("  {} Server reloaded, domain is now blocked", "✓".bright_green().bold());
                    }
                    None => {
                        println!("  {} Server not running - changes will apply on next start", "ℹ".bright_yellow());
                    }
                }
                
                println!();
                Ok(())
            }
            Some(Commands::Remove { domain, config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{} {}", "➖ Removing domain:".bright_red().bold(), domain.bright_cyan());
                println!();
                
                // Read current blocklist
                let content = fs::read_to_string(&config.blocklist.custom_list)?;
                let domains: Vec<String> = content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && line != domain)
                    .collect();
                
                let original_count = content.lines().count();
                let new_count = domains.len();
                
                if original_count == new_count {
                    println!("  {} Domain not found in blocklist", "ℹ".bright_yellow());
                } else {
                    // Write back with trailing newline
                    let content = domains.join("\n") + "\n";
                    fs::write(&config.blocklist.custom_list, content)?;
                    println!("  {} Domain removed from: {}", "✓".bright_green(), config.blocklist.custom_list.bright_blue());
                    
                    // Trigger reload if server is running
                    match find_server_pid()? {
                        Some(pid) => {
                            println!("  {} Reloading server...", "🔄".bright_cyan());
                            send_signal(pid, SIGHUP)?;
                            std::thread::sleep(std::time::Duration::from_millis(300));
                            println!("  {} Server reloaded, domain is now allowed", "✓".bright_green().bold());
                        }
                        None => {
                            println!("  {} Server not running - changes will apply on next start", "ℹ".bright_yellow());
                        }
                    }
                }
                
                println!();
                Ok(())
            }
            Some(Commands::List { config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{}", "📋 Blocklist Statistics".bright_cyan().bold());
                println!("{}", "═".repeat(50).bright_black());
                println!();
                
                let blocklist = BlocklistManager::new();
                load_blocklist_from_config(&config, &blocklist).await?;
                let total = blocklist.count().await;
                
                println!("  {} Total Blocked Domains: {}", "🚫".bright_red(), total.to_string().bright_yellow().bold());
                println!();
                
                // Count by source
                println!("  {} Sources:", "📁".bright_cyan());
                
                if std::path::Path::new(&config.blocklist.custom_list).exists() {
                    let content = fs::read_to_string(&config.blocklist.custom_list)?;
                    let count = content.lines()
                        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                        .count();
                    println!("    {} Custom list: {} domains", "•".bright_white(), count.to_string().bright_green());
                    println!("      {} {}", "↳".bright_black(), config.blocklist.custom_list.bright_blue());
                }
                
                for (idx, local_list) in config.blocklist.local_lists.iter().enumerate() {
                    if std::path::Path::new(local_list).exists() {
                        let content = fs::read_to_string(local_list)?;
                        let count = content.lines()
                            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                            .count();
                        println!("    {} Local list {}: {} domains", "•".bright_white(), idx + 1, count.to_string().bright_green());
                        println!("      {} {}", "↳".bright_black(), local_list.bright_blue());
                    }
                }
                
                if !config.blocklist.remote_lists.is_empty() {
                    println!();
                    println!("  {} Remote Sources (not yet downloaded):", "🌐".bright_cyan());
                    for url in &config.blocklist.remote_lists {
                        println!("    {} {}", "•".bright_white(), url.bright_blue());
                    }
                }
                
                println!();
                println!("{}", "═".repeat(50).bright_black());
                println!();
                
                Ok(())
            }
            Some(Commands::Update { config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{}", "🔄 Updating Blocklists".bright_cyan().bold());
                println!();
                
                if config.blocklist.remote_lists.is_empty() {
                    println!("  {} No remote sources configured", "⚠".bright_yellow());
                    println!();
                    println!("  {} Add remote sources to your config:", "💡".bright_yellow());
                    println!("    {}", "[blocklist]".bright_blue());
                    println!("    {}", "remote_lists = [".bright_blue());
                    println!("    {}", "  \"https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts\"".bright_green());
                    println!("    {}", "]".bright_blue());
                    println!();
                    return Ok(());
                }
                
                println!("  {} Remote sources:", "🌐".bright_cyan());
                for url in &config.blocklist.remote_lists {
                    println!("    {} {}", "•".bright_white(), url.bright_blue());
                }
                println!();
                
                // Download blocklists
                println!("  {} Downloading blocklists...", "⬇".bright_yellow());
                let downloader = BlocklistDownloader::new()?;
                
                match downloader.download_multiple(&config.blocklist.remote_lists).await {
                    Ok(domains) => {
                        println!("  {} Downloaded {} unique domains", "✓".bright_green(), domains.len().to_string().bright_yellow().bold());
                        
                        // Save to a cache file
                        let cache_dir = std::path::Path::new(&config.blocklist.custom_list)
                            .parent()
                            .unwrap_or(std::path::Path::new("/tmp"));
                        let cache_file = cache_dir.join("remote-blocklist-cache.txt");
                        
                        println!("  {} Saving to cache: {}", "💾".bright_cyan(), cache_file.display().to_string().bright_blue());
                        
                        let content = domains.join("\n") + "\n";
                        std::fs::write(&cache_file, content)?;
                        
                        println!("  {} Cache saved successfully", "✓".bright_green());
                        
                        // Trigger reload if server is running
                        match find_server_pid()? {
                            Some(pid) => {
                                println!();
                                println!("  {} Reloading server with new blocklists...", "🔄".bright_cyan());
                                send_signal(pid, SIGHUP)?;
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                println!("  {} Server reloaded successfully", "✓".bright_green().bold());
                                println!("  {} {} domains now active", "🚫".bright_red(), domains.len().to_string().bright_yellow().bold());
                            }
                            None => {
                                println!();
                                println!("  {} Server not running", "ℹ".bright_yellow());
                                println!("  {} Start the server to apply new blocklists:", "→".bright_white());
                                println!("    {}", "skypier-blackhole start".bright_green());
                            }
                        }
                    }
                    Err(e) => {
                        println!("  {} Failed to download blocklists: {}", "✗".bright_red(), e);
                        println!("  {} Check your internet connection and URLs", "ℹ".bright_yellow());
                    }
                }
                
                println!();
                Ok(())
            }
            Some(Commands::Test { domain, config: config_path }) => {
                let config = Config::load(config_path)?;
                println!("{} {}", "🔍 Testing domain:".bright_cyan().bold(), domain.bright_yellow());
                println!();
                
                let blocklist = BlocklistManager::new();
                load_blocklist_from_config(&config, &blocklist).await?;
                
                let is_blocked = blocklist.is_blocked(domain).await;
                
                if is_blocked {
                    println!("  {} Status: {}", "🚫".bright_red(), "BLOCKED".bright_red().bold());
                    println!("  {} This domain will be blocked by the DNS server", "ℹ".bright_blue());
                    println!("  {} DNS queries will receive: {}", "→".bright_white(), "REFUSED".bright_yellow());
                } else {
                    println!("  {} Status: {}", "✓".bright_green(), "ALLOWED".bright_green().bold());
                    println!("  {} This domain will be resolved normally", "ℹ".bright_blue());
                    println!("  {} DNS queries will be forwarded to upstream: {}", 
                            "→".bright_white(), 
                            config.server.upstream_dns.first().unwrap_or(&"1.1.1.1:53".to_string()).bright_cyan());
                }
                
                println!();
                Ok(())
            }
            None => {
                // Default action: show help
                println!("{}", "No command specified. Use --help to see available commands.".bright_yellow());
                println!();
                println!("{}", "Quick start:".bright_cyan().bold());
                println!("  {} Start the DNS server:", "•".bright_white());
                println!("    {}", "skypier-blackhole start".bright_green());
                println!("  {} Check server status:", "•".bright_white());
                println!("    {}", "skypier-blackhole status".bright_green());
                println!("  {} Test a domain:", "•".bright_white());
                println!("    {}", "skypier-blackhole test ads.example.com".bright_green());
                println!();
                
                Ok(())
            }
        }
    }
}
