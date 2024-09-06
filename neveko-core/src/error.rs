use kn0sys_lmdb_rs::MdbError;
use thiserror::Error;

/// Use for mapping errors in functions that can throw multiple errors.
#[derive(Debug, Error)]
#[error("Neveko error. See logs for more info.")]
pub enum NevekoError {
    ///J4I2PRS(J4RsError),
    Database(MdbError),
    Dispute,
    MoneroRpc,
    MoneroDaemon,
    Unknown,
}