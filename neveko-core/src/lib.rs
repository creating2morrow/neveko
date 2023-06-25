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
pub const MSIG_MESSAGE_DB_KEY:          &str = "msig";
pub const FTS_JWP_DB_KEY:               &str = "fts-jwp";
// End LMDB Keys

/// Environment variable for injecting wallet password
pub const MONERO_WALLET_PASSWORD: &str = "MONERO_WALLET_PASSWORD";
/// Environment variable for I2P proxy host
pub const NEVEKO_I2P_PROXY_HOST: &str = "NEVEKO_I2P_PROXY_HOST";
/// Environment variable for I2P manual tunnels.json
pub const NEVEKO_I2P_TUNNELS_JSON: &str = "NEVEKO_I2P_TUNNELS_JSON";
/// Environment variable for I2P advanced mode
pub const NEVEKO_I2P_ADVANCED_MODE: &str = "NEVEKO_I2P_ADVANCED_MODE";

/// The latest monero release download
pub const MONERO_RELEASE_VERSION: &str = "monero-linux-x64-v0.18.2.2.tar.bz2";
pub const MONERO_RELEASE_HASH: &str =
    "186800de18f67cca8475ce392168aabeb5709a8f8058b0f7919d7c693786d56b";
/// The latest i2p-zero release version
pub const I2P_ZERO_RELEASE_VERSION: &str = "v1.21";
pub const I2P_ZERO_RELEASH_HASH: &str =
    "14f34052ad6abb0c24b048816b0ea86b696ae350dd139dd1e90a67ca88e1d07a";
// DO NOT EDIT BELOW THIS LINE
