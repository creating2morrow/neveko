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
pub mod user;       // misc.
pub mod utils;      // user rep/service layer

pub const NEVMES_JWP_SECRET_KEY: &str = "NEVMES_JWP_SECRET_KEY";
pub const NEVMES_JWT_SECRET_KEY: &str = "NEVMES_JWT_SECRET_KEY";

/// The latest monero release download
pub const MONERO_RELEASE_VERSION: &str = "monero-linux-x64-v0.18.2.2.tar.bz2";
pub const MONERO_RELEASE_HASH: &str =
    "186800de18f67cca8475ce392168aabeb5709a8f8058b0f7919d7c693786d56b";
/// The latest i2p-zero release version
pub const I2P_ZERO_RELEASE_VERSION: &str = "v1.21";
pub const I2P_ZERO_RELEASH_HASH: &str =
    "14f34052ad6abb0c24b048816b0ea86b696ae350dd139dd1e90a67ca88e1d07a";
/// The latest i2pd release version
pub const I2P_RELEASE_VERSION: &str = "2.2.1";
pub const I2P_RELEASE_HASH: &str =
    "c9879b8f69ea13c758672c2fa083dc2e0abb289e0fc9a55af98f9f1795f82659";
// DO NOT EDIT BELOW THIS LINE
