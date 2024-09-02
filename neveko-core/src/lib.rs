pub mod args;
pub mod auth;
pub mod contact;
pub mod neveko25519;
pub mod dispute;
pub mod db;
pub mod i2p;
pub mod message;
pub mod models;
pub mod monero;
pub mod order;
pub mod product;
pub mod proof;
pub mod reqres;
pub mod user;
pub mod utils;

pub const APP_NAME: &str = "neveko";
pub const NEVEKO_JWP_SECRET_KEY: &str = "NEVEKO_JWP_SECRET_KEY";
pub const NEVEKO_JWT_SECRET_KEY: &str = "NEVEKO_JWT_SECRET_KEY";
pub const NEVEKO_NMPK: &str = "NEVEKO_NMPK";

// LMDB Keys
pub const AUTH_DB_KEY:                  &str = "a";
pub const CONTACT_DB_KEY:               &str = "c";
pub const DISPUTE_DB_KEY:               &str = "d";
pub const MESSAGE_DB_KEY:               &str = "m";
pub const ORDER_DB_KEY:                 &str = "o";
pub const PRODUCT_DB_KEY:               &str = "p";
pub const USER_DB_KEY:                  &str = "u";
pub const CONTACT_LIST_DB_KEY:          &str = "cl";
pub const DISPUTE_LIST_DB_KEY:          &str = "dl";
pub const MESSAGE_LIST_DB_KEY:          &str = "ml";
pub const ORDER_LIST_DB_KEY:            &str = "ol";
pub const PRODUCT_LIST_DB_KEY:          &str = "pl";
pub const RX_MESSAGE_DB_KEY:            &str = "rx";
pub const DISPUTE_LAST_CHECK_DB_KEY:    &str = "dlc";
pub const FTS_DB_KEY:                   &str = "fts"; 
pub const CUSTOMER_ORDER_LIST_DB_KEY:   &str = "olc";
pub const ADJUDICATOR_DB_KEY:              &str = "med8";
pub const MSIG_MESSAGE_DB_KEY:          &str = "msig";
pub const MSIG_MESSAGE_LIST_DB_KEY:     &str = "msigl";
pub const FTS_JWP_DB_KEY:               &str = "fts-jwp";
pub const DELIVERY_INFO_DB_KEY:         &str = "delivery";
// End LMDB Keys

/// Environment variable for injecting wallet password
pub const MONERO_WALLET_PASSWORD: &str = "MONERO_WALLET_PASSWORD";
/// Environment variable for I2P proxy host
pub const NEVEKO_I2P_PROXY_HOST: &str = "NEVEKO_I2P_PROXY_HOST";
/// Environment variable for I2P manual tunnels.json
pub const NEVEKO_I2P_TUNNELS_JSON: &str = "NEVEKO_I2P_TUNNELS_JSON";
/// Environment variable for I2P advanced mode
pub const NEVEKO_I2P_ADVANCED_MODE: &str = "NEVEKO_I2P_ADVANCED_MODE";
/// Environment variable for I2P advanced mode
pub const MONERO_DAEMON_HOST: &str = "MONERO_DAEMON_HOST";
/// Environment variable for I2P advanced mode
pub const MONERO_WALLET_RPC_HOST: &str = "MONERO_WALLET_RPC_HOST";
/// Reference to check if gui set remote node flag
pub const GUI_REMOTE_NODE: &str = "GUI_REMOTE_NODE";
pub const GUI_SET_REMOTE_NODE: &str = "1";

/// The latest monero release download
pub const MONERO_RELEASE_VERSION: &str = "monero-linux-x64-v0.18.3.2.tar.bz2";
pub const MONERO_RELEASE_HASH: &str =
    "9dafd70230a7b3a73101b624f3b5f439cc5b84a19b12c17c24e6aab94b678cbb";
/// The latest i2p-zero release version
pub const I2P_ZERO_RELEASE_VERSION: &str = "v1.21";
pub const I2P_ZERO_RELEASH_HASH: &str =
    "14f34052ad6abb0c24b048816b0ea86b696ae350dd139dd1e90a67ca88e1d07a";

pub const LMDB_MAPSIZE: u64 = 1024 * 1024 * 1024;
pub const I2P_CONNECTIVITY_CHECK_INTERVAL: u32 = 600000;
pub const FTS_RETRY_INTERVAL: u32 = 60000;
/// There is a one week grace period for manual intervention of disputes
pub const DISPUTE_AUTO_SETTLE: u32 = 1000 * 60 * 60 * 24 * 7;
/// Daily dispute auto-settle check interval
pub const DISPUTE_CHECK_INTERVAL: u32 = 1000 * 60 * 60 * 24;
// DO NOT EDIT BELOW THIS LINE
