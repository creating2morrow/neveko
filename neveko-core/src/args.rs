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
        default_value = "monero-x86_64-linux-gnu-v0.18.2.2"
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
    /// Absolute path to i2p zero
    #[arg(
        long,
        help = "Absolute path to i2p-zero directroy",
        default_value = "/home/user/i2p-zero-linux.v1.21"
    )]
    pub i2p_zero_dir: String,
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
    /// Auto trust contact gpg keys (DISABLED)
    #[arg(
        long,
        help = "FUTURE FEATURE. Auto trust contacts. DISABLED",
        default_value = "false"
    )]
    pub auto_trust: bool,
    /// Start with gui
    #[arg(
        long,
        help = "Start the graphical user interface",
        default_value = "false"
    )]
    pub gui: bool,
    /// i2p http proxy host
    #[arg(
        long,
        help = "i2p http proxy host",
        default_value = "http://localhost:4444"
    )]
    pub i2p_proxy_host: String,
    /// Connect wallet rpc for a remote-node, WARNING: may harm privacy
    #[arg(
        long,
        help = "connect to remote node, don't use locally running monerod",
        default_value = "false"
    )]
    pub remote_node: bool,
    /// Connect to micro servers
    #[arg(
        long,
        help = "allow remote access to mirco server functionality",
        default_value = "false"
    )]
    pub remote_access: bool,
    /// Remove all failed-to-send messages from db on app startup
    #[arg(
        long,
        help = "this will clear failed-to-send messages from the databse",
        default_value = "false"
    )]
    pub clear_fts: bool,
}
