pub mod args;       // command line arguments
pub mod auth;       // internal auth repo/service layer
pub mod contact;    // contact repo/service layer
pub mod dispute;    // dispute repo/service layer
pub mod db;         // lmdb interface
pub mod gpg;        // gpgme interface
pub mod i2p;        // i2p repo/service layer
pub mod message;    // message repo/service layer
pub mod models;     // db structs
pub mod monero;     // monero-wallet-rpc interface
pub mod order;      // order repo/service layer
pub mod product;    // product repo/service layer
pub mod proof;      // external auth/payment proof module
pub mod reqres;     // http request/responses
pub mod user;       // user repo/service layer
pub mod utils;      // misc.

pub const APP_NAME: &str = "neveko";
pub const NEVEKO_JWP_SECRET_KEY: &str = "NEVEKO_JWP_SECRET_KEY";
pub const NEVEKO_JWT_SECRET_KEY: &str = "NEVEKO_JWT_SECRET_KEY";

// LMDB Keys
pub const AUTH_DB_KEY:                  &str = "a";
pub const CONTACT_DB_KEY:               &str = "c";
pub const DISPUTE_DB_KEY:               &str = "d";
pub const MESSAGE_DB_KEY:               &str = "m";
pub const ORDER_DB_KEY:                 &str = "o";
pub const PRODUCT_DB_KEY:               &str = "p";
pub const USER_DB_KEY:                  &str = "u";
pub const CONTACT_LIST_DB_KEY:          &str = "cl";
pub const MESSAGE_LIST_DB_KEY:          &str = "ml";
pub const ORDER_LIST_DB_KEY:            &str = "ol";
pub const PRODUCT_LIST_DB_KEY:          &str = "pl";
pub const RX_MESSAGE_DB_KEY:            &str = "rx";
pub const FTS_DB_KEY:                   &str = "fts"; 
pub const CUSTOMER_ORDER_LIST_DB_KEY:   &str = "olc";
pub const MEDIATOR_DB_KEY:              &str = "med8";
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
pub const MONERO_RELEASE_VERSION: &str = "monero-linux-x64-v0.18.3.1.tar.bz2";
pub const MONERO_RELEASE_HASH: &str =
    "23af572fdfe3459b9ab97e2e9aa7e3c11021c955d6064b801a27d7e8c21ae09d";
/// The latest i2p-zero release version
pub const I2P_ZERO_RELEASE_VERSION: &str = "v1.21";
pub const I2P_ZERO_RELEASH_HASH: &str =
    "14f34052ad6abb0c24b048816b0ea86b696ae350dd139dd1e90a67ca88e1d07a";
// DO NOT EDIT BELOW THIS LINE
