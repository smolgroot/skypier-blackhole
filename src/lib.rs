mod cli;
mod config;
mod dns;
mod blocklist;
mod logger;

pub use cli::Cli;
pub use config::Config;
pub use dns::DnsServer;
pub use blocklist::BlocklistManager;
pub use logger::setup_logging;

pub type Result<T> = std::result::Result<T, anyhow::Error>;
