//! Neveko app for egui
#![allow(clippy::missing_errors_doc)]

mod apps;
mod login;
mod wrap_app;

// LMDB keys
pub const GUI_JWP_DB_KEY:           &str = "gui-jwp";
pub const GUI_EXP_DB_KEY:           &str = "gui-exp";
pub const GUI_TX_PROOF_DB_KEY:      &str = "gui-txp";
pub const GUI_NICK_DB_KEY:          &str = "gui-nick";
/// Order-Vendor-Lookup for fetching .b32.i2p for order;
pub const GUI_OVL_DB_KEY:           &str = "gui-ovl";
pub const GUI_TX_SIGNATURE_DB_KEY:  &str = "gui-txp-sig";
pub const GUI_TX_HASH_DB_KEY:       &str = "gui-txp-hash";
pub const GUI_SIGNED_GPG_DB_KEY:    &str = "gui-signed-key";
pub const GUI_TX_SUBADDRESS_DB_KEY: &str = "gui-txp-subaddress";

pub const GUI_MSIG_KEX_ONE_DB_KEY:  &str = "gui-kex-1";
pub const GUI_MSIG_KEX_TWO_DB_KEY:  &str = "gui-kex-2";
pub const GUI_MSIG_EXPORT_DB_KEY:   &str = "gui-export";
pub const GUI_MSIG_IMPORT_DB_KEY:   &str = "gui-import";
pub const GUI_MSIG_MAKE_DB_KEY:     &str = "gui-make";
pub const GUI_MSIG_MEDIATOR_DB_KEY: &str = "gui-mediator";
pub const GUI_MSIG_PREPARE_DB_KEY:  &str = "gui-prepare";
pub const GUI_MSIG_TXSET_DB_KEY:    &str = "gui-txset";
// End LMDB keys

/// Designate a contact as verified and trusted
pub const SIGNED_GPG_KEY: &str = "1";

/// key for fetching the login credential hash
pub const CREDENTIAL_KEY: &str = "NEVEKO_GUI_KEY";
/// TODO(c2m): configurable lock screen timeout
pub const LOCK_SCREEN_TIMEOUT_SECS: u64 = 60 * 5;
/// interval to search for credential on initial gui load
pub const CRED_CHECK_INTERVAL: u64 = 5;
/// monero estimated block time in seconds
pub const BLOCK_TIME_IN_SECS_EST: u64 = 0x78;
/// monero estimated propagation time in seconds
pub const PROPAGATION_TIME_IN_SECS_EST: u64 = 5;
pub const I2P_PROPAGATION_TIME_IN_SECS_EST: u64 = PROPAGATION_TIME_IN_SECS_EST * 10;
/// time to wait before giving up on adding a contact
pub const ADD_CONTACT_TIMEOUT_SECS: u64 = 0x5A;
/// time to wait before giving up on neveko core
pub const START_CORE_TIMEOUT_SECS: u64 = 0x79;
/// bytes in a a GB for calculating space on home page
pub const BYTES_IN_GB: u64 = 1000000000;
/// Useful flag to keep services running in background
pub const NEVEKO_DEV_BACKGROUND: &str = "NEVEKO_DEV_BACKGROUND";
pub use wrap_app::WrapApp;
