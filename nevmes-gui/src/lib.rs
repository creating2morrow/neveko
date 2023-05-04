//! App app for egui
#![allow(clippy::missing_errors_doc)]

mod apps;
mod login;
mod wrap_app;

/// key for fetching the login credential hash
pub const CREDENTIAL_KEY: &str = "NEVMES_GUI_KEY";
/// TODO(c2m): configurable lock screen timeout
/// , also the screen shouldn't lock if your actively
/// using it.
pub const LOCK_SCREEN_TIMEOUT_SECS: u64 = 60*5;
/// interval to search for credential on initial gui load
pub const CRED_CHECK_INTERVAL: u64 = 5;
/// monero estimated block time in seconds
pub const BLOCK_TIME_IN_SECS_EST: u128 = 0x78;
/// time to wait before giving up on adding a contact
pub const ADD_CONTACT_TIMEOUT_SECS: u64 = 0x5A;
/// time to wait before giving up on nevmes core
pub const START_CORE_TIMEOUT_SECS: u64 = 0x79;
/// bytes in a a GB for calculating space on home page
pub const BYTES_IN_GB: u64 = 1000000000;

pub use wrap_app::WrapApp;
