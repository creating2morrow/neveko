//! core command line arguments
use clap::Parser;

/// cmd line args
#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// set release environment
    #[arg(
        short,
        long,
        help = "Set release environment (dev, prod)",
        default_value = "dev"
    )]
    pub release_env: String,
    /// Monero location
    #[arg(
        long,
        help = "Monero download absolute path.",
        default_value = "monero-x86_64-linux-gnu-v0.18.3.3"
    )]
    pub monero_location: String,
    /// Monero RPC host
    #[arg(
        long,
        help = "Monero RPC host.",
        default_value = "http://localhost:38083"
    )]
    pub monero_rpc_host: String,
    /// Monero blockchain location
    #[arg(
        long,
        help = "Monero blockchain location",
        default_value = "/home/user/.bitmonero"
    )]
    pub monero_blockchain_dir: String,
    /// Monero RPC daemon host
    #[arg(
        long,
        help = "Monero RPC daemon.",
        default_value = "http://localhost:38081"
    )]
    pub monero_rpc_daemon: String,
    /// Monero RPC Username
    #[arg(long, help = "Monero RPC username.", default_value = "user")]
    pub monero_rpc_username: String,
    /// Monero RPC credential
    #[arg(long, help = "Monero RPC credential.", default_value = "pass")]
    pub monero_rpc_cred: String,
    /// Token expiration in minutes
    #[arg(
        short,
        long,
        help = "Set the token expiration limit in minutes.",
        default_value = "60"
    )]
    pub token_timeout: i64,
    /// Payment Threshold
    #[arg(
        short,
        long,
        help = "Set a payment threshold in piconeros",
        default_value = "1"
    )]
    pub payment_threshold: u128,
    /// Confirmation Threshold
    #[arg(
        short,
        long,
        help = "Set a confirmation expiration for payments",
        default_value = "720"
    )]
    pub confirmation_threshold: u64,
    /// Application port
    #[arg(long, help = "Set app port", default_value = "9000")]
    pub port: u16,
    /// Auth port
    #[arg(long, help = "Set app auth port", default_value = "9043")]
    pub auth_port: u16,
    /// Contact saving port
    #[arg(long, help = "Set app contact saving port", default_value = "9044")]
    pub contact_port: u16,
    /// Messaging sending port
    #[arg(long, help = "Set app message sending port", default_value = "9045")]
    pub message_port: u16,
    /// Marketplace admin port
    #[arg(long, help = "Set app marketplace admin port", default_value = "9046")]
    pub marketplace_port: u16,
    /// i2p http proxy host
    #[arg(
        long,
        help = "i2p http proxy host",
        default_value = "http://localhost:4444"
    )]
    pub i2p_proxy_host: String,
    /// i2p wallet proxy host (i2p socks)
    #[arg(
        long,
        help = "i2p remote node socks proxy host",
        default_value = "http://localhost:9051"
    )]
    pub i2p_socks_proxy_host: String,
    /// Connect wallet rpc for a remote-node, WARNING: may harm privacy
    #[arg(
        long,
        help = "connect to remote node, don't use locally running monerod",
        default_value = "false"
    )]
    pub remote_node: bool,
    /// Dummy flag for normal mode when not using remote node. Future use.
    #[arg(
        long,
        help = "dummy flag for normal node operations. (Future use)",
        default_value = "false"
    )]
    pub full_node: bool,
    /// Remove all failed-to-send messages from db on app startup
    #[arg(
        long,
        help = "this will clear failed-to-send messages from the database",
        default_value = "false"
    )]
    pub clear_fts: bool,
    /// Remove all disputes from db on app startup
    #[arg(
        long,
        help = "this will clear disputes from the database",
        default_value = "false"
    )]
    pub clear_disputes: bool,
    /// Manually configure i2p
    #[arg(
        long,
        help = "ADVANCED. Neveko will no longer handle i2p proxy tunnels or identity.",
        default_value = "false"
    )]
    pub i2p_advanced: bool,
    /// Manually configured tunnels.json directory
    #[arg(
        long,
        help = "ADVANCED. Location of the manually created destination tunnels.",
        default_value = "/home/user/neveko/i2p-manual"
    )]
    pub i2p_tunnels_json: String,
    /// Dummy flag for normal neveko i2p-zero config. Future use.
    #[arg(
        long,
        help = "Normal mode. Neveko will handle i2p proxy tunnels and identity.",
        default_value = "false"
    )]
    pub i2p_normal: bool,
    /// i2p anonymous inbound port
    #[arg(
        long,
        help = "Set i2p anon inbound connectivity",
        default_value = "38089"
    )]
    pub anon_inbound_port: u16,
}
