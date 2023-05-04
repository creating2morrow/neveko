pub mod args;       // command line arguments
pub mod auth;       // internal auth repo/service layer
pub mod contact;    // contact repo/service layer
pub mod db;         // lmdb interface
pub mod gpg;        // gpgme interface
pub mod i2p;        // i2p repo/service layer
pub mod message;    // message repo/service layer
pub mod models;     // db structs
pub mod monero;     // monero-wallet-rpc interface
pub mod proof;      // external auth/payment proof module
pub mod reqres;     // http request/responses
pub mod utils;      // misc.
pub mod user;       // user rep/service layer

pub const NEVMES_JWP_SECRET_KEY: &str = "NEVMES_JWP_SECRET_KEY";
pub const NEVMES_JWT_SECRET_KEY: &str = "NEVMES_JWT_SECRET_KEY";

// set the latest monero release download here
pub const MONERO_RELEASE_VERSION: &str = "monero-linux-x64-v0.18.2.2.tar.bz2";

// DO NOT EDIT BELOW THIS LINE
